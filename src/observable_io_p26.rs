use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use crate::capability::{Capability, CapabilitySet};
use crate::dynamic_input_v07::{
    lower_dynamic_plan_v07, DynamicInputError, DynamicValue, DynamicValueKind,
};
use crate::effect_audit::hash::sha256;
use crate::frontend::{
    analyze_type_effect_unit, lex, LexError, SourceError, SourceSpan, SourceText,
    SurfaceDeclarationKind, TokenKind, TypeEffectError,
};
use crate::input::{
    InputBatch, InputDeviceId, InputError, InputEvent, InputPayload, InputSequence, InputSource,
    InputTarget,
};
use crate::observable_io_p24::{
    compile_multi_segment_output_plan_p24, MultiSegmentObservableIoError,
    P24MultiSegmentOutputPlan, MAX_P24_OUTPUT_BYTES,
};
use crate::observable_io_p25::{
    compile_file_output_plan_p25, execute_file_output_p25, FileOutputError, P25FileOutputReceipt,
    P25_FILE_EFFECT_NAME,
};
use crate::runtime::{run_closed_observed, RuntimeError};
use crate::value::Value;

const P26_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.6-DYNAMIC-CAPABILITY-SECURED-FILE-OUTPUT-PLAN\0";
const P26_SEGMENT_DOMAIN: &[u8] = b"NORDOI-P2.6-DYNAMIC-FILE-OUTPUT-SEGMENT-PROOF\0";
const P26_RECEIPT_DOMAIN: &[u8] = b"NORDOI-P2.6-DYNAMIC-CAPABILITY-SECURED-FILE-OUTPUT-RECEIPT\0";

pub const P26_OUTPUT_SCHEMA: &str = "nordoi.dynamic-file-output.p2.6";
pub const MAX_P26_OUTPUT_BYTES: usize = MAX_P24_OUTPUT_BYTES;

#[derive(Debug)]
pub enum DynamicFileOutputError {
    TypeEffect(TypeEffectError),
    Lex(LexError),
    Source(SourceError),
    Input(InputError),
    Dynamic(DynamicInputError),
    Runtime(RuntimeError),
    Syntax {
        message: String,
        span: SourceSpan,
    },
    Policy {
        message: String,
        span: Option<SourceSpan>,
    },
    CapabilityDenied(Capability),
    Materialization {
        message: String,
    },
    Invariant {
        message: String,
    },
}

impl DynamicFileOutputError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) | Self::Input(_) | Self::Dynamic(_) | Self::Runtime(_) => None,
            Self::Syntax { span, .. } => Some(*span),
            Self::Policy { span, .. } => *span,
            Self::CapabilityDenied(_) | Self::Materialization { .. } | Self::Invariant { .. } => {
                None
            }
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        matches!(
            self,
            Self::TypeEffect(_)
                | Self::Lex(_)
                | Self::Source(_)
                | Self::Syntax { .. }
                | Self::Policy { .. }
        )
    }

    pub fn is_authority_failure(&self) -> bool {
        matches!(self, Self::CapabilityDenied(_))
    }

    pub fn is_materialization_failure(&self) -> bool {
        matches!(self, Self::Materialization { .. })
    }
}

impl Display for DynamicFileOutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Input(error) => Display::fmt(error, f),
            Self::Dynamic(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.6 dynamic file output syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.6 dynamic file output policy error: {message}")
            }
            Self::CapabilityDenied(capability) => write!(
                f,
                "P2.6 dynamic file output authority error: capability denied: {capability:?}"
            ),
            Self::Materialization { message } => {
                write!(
                    f,
                    "P2.6 dynamic file output materialization error: {message}"
                )
            }
            Self::Invariant { message } => {
                write!(f, "P2.6 dynamic file output invariant failed: {message}")
            }
        }
    }
}

impl Error for DynamicFileOutputError {}

impl From<TypeEffectError> for DynamicFileOutputError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<LexError> for DynamicFileOutputError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for DynamicFileOutputError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<InputError> for DynamicFileOutputError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

impl From<DynamicInputError> for DynamicFileOutputError {
    fn from(value: DynamicInputError) -> Self {
        Self::Dynamic(value)
    }
}

impl From<RuntimeError> for DynamicFileOutputError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

pub type DynamicFileOutputResult<T> = Result<T, DynamicFileOutputError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P26DynamicFileOutputPlan {
    file_name: String,
    render_plan: P24MultiSegmentOutputPlan,
    target_policy_sha256: [u8; 32],
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P26DynamicFileOutputPlan {
    pub fn module(&self) -> Option<&str> {
        self.render_plan.module()
    }

    pub fn entry_name(&self) -> &str {
        self.render_plan.entry_name()
    }

    pub fn input_name(&self) -> &str {
        self.render_plan.input_name()
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn runtime_segment_count(&self) -> usize {
        self.render_plan.runtime_segment_count()
    }

    pub fn result_kinds(&self) -> Vec<DynamicValueKind> {
        self.render_plan.result_kinds()
    }

    pub fn render_plan(&self) -> &P24MultiSegmentOutputPlan {
        &self.render_plan
    }

    pub fn target_policy_sha256_hex(&self) -> String {
        hex_bytes(&self.target_policy_sha256)
    }

    pub fn canonical_plan_bytes(&self) -> &[u8] {
        &self.canonical_plan
    }

    pub fn plan_sha256_hex(&self) -> String {
        hex_bytes(&self.plan_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P26DynamicFileOutputReceipt {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    file_name: String,
    key_code: u32,
    results: Vec<DynamicValue>,
    output: String,
    plan_sha256: [u8; 32],
    render_plan_sha256: [u8; 32],
    segment_proof_sha256: Vec<[u8; 32]>,
    file_receipt_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P26DynamicFileOutputReceipt {
    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn output_bytes(&self) -> usize {
        self.output.len()
    }

    pub fn results(&self) -> &[DynamicValue] {
        &self.results
    }

    pub fn receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.receipt_sha256)
    }

    pub fn canonical_receipt_bytes(&self) -> &[u8] {
        &self.canonical_receipt
    }

    pub fn render_text(&self) -> String {
        let results = self
            .results
            .iter()
            .map(|value| value.as_text())
            .collect::<Vec<_>>()
            .join(",");
        let segment_proofs = self
            .segment_proof_sha256
            .iter()
            .map(|hash| hex_bytes(hash))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "dynamic-file-write module=\"{}\" entry=\"{}\" input=\"{}\" key-code={} target=\"{}\" segments={} results=[{}] effect={} capability=FileWrite(\"{}\") output-bytes={} plan-sha256={} render-plan-sha256={} segment-proofs-sha256=[{}] file-receipt-sha256={} receipt-sha256={} status=WRITTEN authority=EXPLICIT grant=FileWrite(\"{}\") ambient-authority=NONE overwrite=DENIED\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            escape_text(&self.input_name),
            self.key_code,
            escape_text(&self.file_name),
            self.results.len(),
            results,
            P25_FILE_EFFECT_NAME,
            escape_text(&self.file_name),
            self.output_bytes(),
            hex_bytes(&self.plan_sha256),
            hex_bytes(&self.render_plan_sha256),
            segment_proofs,
            hex_bytes(&self.file_receipt_sha256),
            self.receipt_sha256_hex(),
            escape_text(&self.file_name),
        )
    }
}

pub fn compile_dynamic_file_output_plan_p26(
    source: &SourceText,
) -> DynamicFileOutputResult<P26DynamicFileOutputPlan> {
    let unit = analyze_type_effect_unit(source)?;
    let effect_name_span = validate_effect_prelude(source, &unit)?;

    let tokens = lex(source)?;
    let significant = tokens
        .iter()
        .filter(|token| {
            !token.is_trivia()
                && token.kind() != &TokenKind::Eof
                && token.span().start().get() >= unit.body_span().start().get()
        })
        .collect::<Vec<_>>();

    let entry_indexes = significant
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind() == &TokenKind::Identifier
                && source.slice(token.span()).ok() == Some("entry"))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if entry_indexes.len() != 1 {
        return Err(DynamicFileOutputError::Policy {
            message: "P2.6 requires exactly one entry declaration".to_owned(),
            span: entry_indexes
                .first()
                .and_then(|index| significant.get(*index))
                .map(|token| token.span())
                .or(Some(unit.body_span())),
        });
    }

    let entry_index = entry_indexes[0];
    let writes =
        significant
            .get(entry_index + 2)
            .ok_or_else(|| DynamicFileOutputError::Syntax {
                message: "expected: entry <name> writes \"<file>\" emits <P2.4 structured output>;"
                    .to_owned(),
                span: significant[entry_index].span(),
            })?;
    if writes.kind() != &TokenKind::Identifier || source.slice(writes.span())? != "writes" {
        return Err(DynamicFileOutputError::Syntax {
            message: "expected contextual 'writes' after the entry name".to_owned(),
            span: writes.span(),
        });
    }
    let file_token =
        significant
            .get(entry_index + 3)
            .ok_or_else(|| DynamicFileOutputError::Syntax {
                message: "expected one quoted output file name after contextual 'writes'"
                    .to_owned(),
                span: writes.span(),
            })?;
    if file_token.kind() != &TokenKind::QuotedText {
        return Err(DynamicFileOutputError::Syntax {
            message: "expected one quoted output file name after contextual 'writes'".to_owned(),
            span: file_token.span(),
        });
    }
    let emits = significant
        .get(entry_index + 4)
        .ok_or_else(|| DynamicFileOutputError::Syntax {
            message: "expected contextual 'emits' after the output file name".to_owned(),
            span: file_token.span(),
        })?;
    if emits.kind() != &TokenKind::Identifier || source.slice(emits.span())? != "emits" {
        return Err(DynamicFileOutputError::Syntax {
            message: "expected contextual 'emits' after the output file name".to_owned(),
            span: emits.span(),
        });
    }

    let entry_name_token =
        significant
            .get(entry_index + 1)
            .ok_or_else(|| DynamicFileOutputError::Syntax {
                message: "expected an entry name".to_owned(),
                span: significant[entry_index].span(),
            })?;
    if entry_name_token.kind() != &TokenKind::Identifier {
        return Err(DynamicFileOutputError::Syntax {
            message: "expected an entry name".to_owned(),
            span: entry_name_token.span(),
        });
    }
    let entry_name = source.slice(entry_name_token.span())?;
    let raw_file = source.slice(file_token.span())?;
    let module = unit
        .module_unit()
        .module()
        .map(|declaration| source.slice(declaration.path().span()).map(str::to_owned))
        .transpose()?;

    let p25_source = static_file_source(module.as_deref(), entry_name, raw_file, "");
    let p25_synthetic = SourceText::new(source.id(), "<p2.6-target-policy>", p25_source)?;
    let target_plan =
        compile_file_output_plan_p25(&p25_synthetic).map_err(map_p25_compile_error)?;

    let transformed = apply_replacements(
        source.text(),
        &[
            (
                effect_name_span.start().get() as usize,
                effect_name_span.end().get() as usize,
                "ConsoleWrite",
            ),
            (
                writes.span().start().get() as usize,
                emits.span().end().get() as usize,
                "emits",
            ),
        ],
    )?;
    let p24_synthetic = SourceText::new(source.id(), "<p2.6-render-plan>", transformed)?;
    let render_plan =
        compile_multi_segment_output_plan_p24(&p24_synthetic).map_err(map_p24_compile_error)?;

    if render_plan.entry_name() != entry_name {
        return Err(DynamicFileOutputError::Invariant {
            message: "P2.4 render plan changed the entry identity".to_owned(),
        });
    }
    if render_plan.module() != module.as_deref() {
        return Err(DynamicFileOutputError::Invariant {
            message: "P2.4 render plan changed the module identity".to_owned(),
        });
    }

    let target_policy_sha256 = sha256(target_plan.canonical_plan_bytes());
    let canonical_plan = canonical_plan_bytes(
        target_plan.file_name(),
        &target_policy_sha256,
        render_plan.canonical_plan_bytes(),
    );
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P26DynamicFileOutputPlan {
        file_name: target_plan.file_name().to_owned(),
        render_plan,
        target_policy_sha256,
        canonical_plan,
        plan_sha256,
    })
}

pub fn execute_dynamic_file_output_p26(
    plan: &P26DynamicFileOutputPlan,
    key_code: u32,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> DynamicFileOutputResult<P26DynamicFileOutputReceipt> {
    let required = Capability::FileWrite(plan.file_name.clone());
    if !authority.contains(&required) {
        return Err(DynamicFileOutputError::CapabilityDenied(required));
    }

    let input = key_batch(key_code);
    let input_bytes = input.canonical_bytes()?;
    let mut results = Vec::with_capacity(plan.render_plan.runtime_segment_count());
    let mut rendered = Vec::with_capacity(plan.render_plan.runtime_segment_count());
    let mut segment_proof_sha256 = Vec::with_capacity(plan.render_plan.runtime_segment_count());

    for (index, segment) in plan.render_plan.segment_plans().iter().enumerate() {
        let lowering = lower_dynamic_plan_v07(segment.dynamic_plan())?;
        let runtime = run_closed_observed(lowering.program(), &input)?;
        let value = runtime
            .register(lowering.result_register())
            .ok_or_else(|| DynamicFileOutputError::Invariant {
                message: format!("segment {} result register is missing", index + 1),
            })?;
        let result = match (segment.result_kind(), value) {
            (DynamicValueKind::Int, Value::Int(value)) => DynamicValue::Int(*value),
            (DynamicValueKind::Bool, Value::Bool(value)) => DynamicValue::Bool(*value),
            (expected, actual) => {
                return Err(DynamicFileOutputError::Invariant {
                    message: format!(
                        "segment {} result kind mismatch: expected {}, got {actual:?}",
                        index + 1,
                        expected.as_str()
                    ),
                });
            }
        };
        let proof = segment_proof_bytes(
            index,
            segment.canonical_plan_bytes(),
            lowering.canonical_v07_witness_bytes(),
            &input_bytes,
            runtime.runtime().replay_key.value(),
            result,
        );
        segment_proof_sha256.push(sha256(&proof));
        rendered.push(render_dynamic_value(result));
        results.push(result);
    }

    let mut output = String::new();
    for (static_segment, rendered_segment) in plan
        .render_plan
        .static_segments()
        .iter()
        .zip(rendered.iter())
    {
        output.push_str(static_segment);
        output.push_str(rendered_segment);
    }
    output.push_str(&plan.render_plan.static_segments()[rendered.len()]);
    if output.len() > MAX_P26_OUTPUT_BYTES {
        return Err(DynamicFileOutputError::Invariant {
            message: format!(
                "rendered dynamic file output has {} bytes, exceeding bound {MAX_P26_OUTPUT_BYTES}",
                output.len()
            ),
        });
    }

    let file_source = static_file_source(
        plan.module(),
        plan.entry_name(),
        &quote_noi_string(&plan.file_name),
        &output,
    );
    let synthetic = SourceText::new(
        crate::frontend::SourceId::new(0x2605),
        "<p2.6-file-materialization>",
        file_source,
    )?;
    let file_plan = compile_file_output_plan_p25(&synthetic).map_err(map_p25_runtime_error)?;
    if file_plan.file_name() != plan.file_name || file_plan.output() != output {
        return Err(DynamicFileOutputError::Invariant {
            message: "P2.5 materialization plan differs from the certified P2.6 output".to_owned(),
        });
    }
    let file_receipt = execute_file_output_p25(&file_plan, authority, output_dir)
        .map_err(map_p25_runtime_error)?;

    build_receipt(
        plan,
        key_code,
        results,
        output,
        segment_proof_sha256,
        &file_receipt,
    )
}

pub fn execute_dynamic_file_output_source_p26(
    source: &SourceText,
    key_code: u32,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> DynamicFileOutputResult<P26DynamicFileOutputReceipt> {
    let plan = compile_dynamic_file_output_plan_p26(source)?;
    execute_dynamic_file_output_p26(&plan, key_code, authority, output_dir)
}

fn build_receipt(
    plan: &P26DynamicFileOutputPlan,
    key_code: u32,
    results: Vec<DynamicValue>,
    output: String,
    segment_proof_sha256: Vec<[u8; 32]>,
    file_receipt: &P25FileOutputReceipt,
) -> DynamicFileOutputResult<P26DynamicFileOutputReceipt> {
    let file_receipt_sha256 = sha256(file_receipt.canonical_receipt_bytes());
    let canonical_receipt = canonical_receipt_bytes(
        plan,
        key_code,
        &results,
        &output,
        &segment_proof_sha256,
        &file_receipt_sha256,
    );
    let receipt_sha256 = sha256(&canonical_receipt);
    Ok(P26DynamicFileOutputReceipt {
        module: plan.module().map(str::to_owned),
        entry_name: plan.entry_name().to_owned(),
        input_name: plan.input_name().to_owned(),
        file_name: plan.file_name.clone(),
        key_code,
        results,
        output,
        plan_sha256: plan.plan_sha256,
        render_plan_sha256: *plan.render_plan.plan_sha256(),
        segment_proof_sha256,
        file_receipt_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

fn validate_effect_prelude(
    source: &SourceText,
    unit: &crate::frontend::TypeEffectUnit,
) -> DynamicFileOutputResult<SourceSpan> {
    let mut file_write = None;
    for declaration in unit.declarations() {
        if declaration.kind() != SurfaceDeclarationKind::Effect {
            continue;
        }
        let name = declaration.name().text(source)?;
        if name != P25_FILE_EFFECT_NAME {
            return Err(DynamicFileOutputError::Policy {
                message: format!("P2.6 permits only effect {P25_FILE_EFFECT_NAME}; found '{name}'"),
                span: Some(declaration.span()),
            });
        }
        if file_write.is_some() {
            return Err(DynamicFileOutputError::Policy {
                message: format!("duplicate effect {P25_FILE_EFFECT_NAME}"),
                span: Some(declaration.span()),
            });
        }
        file_write = Some(declaration.name().span());
    }
    file_write.ok_or_else(|| DynamicFileOutputError::Policy {
        message: format!("dynamic file output requires explicit 'effect {P25_FILE_EFFECT_NAME};'"),
        span: Some(unit.body_span()),
    })
}

fn map_p24_compile_error(error: MultiSegmentObservableIoError) -> DynamicFileOutputError {
    DynamicFileOutputError::Policy {
        message: format!("P2.4 structured renderer rejected the dynamic file body: {error}"),
        span: None,
    }
}

fn map_p25_compile_error(error: FileOutputError) -> DynamicFileOutputError {
    DynamicFileOutputError::Policy {
        message: format!("P2.5 target policy rejected the file target: {error}"),
        span: None,
    }
}

fn map_p25_runtime_error(error: FileOutputError) -> DynamicFileOutputError {
    match error {
        FileOutputError::CapabilityDenied(capability) => {
            DynamicFileOutputError::CapabilityDenied(capability)
        }
        FileOutputError::Materialization { message } => {
            DynamicFileOutputError::Materialization { message }
        }
        other if other.is_frontend_failure() => DynamicFileOutputError::Invariant {
            message: format!("certified P2.5 materialization plan failed validation: {other}"),
        },
        other => DynamicFileOutputError::Invariant {
            message: format!("certified P2.5 materialization failed unexpectedly: {other}"),
        },
    }
}

fn apply_replacements(
    text: &str,
    replacements: &[(usize, usize, &str)],
) -> DynamicFileOutputResult<String> {
    let mut ordered = replacements.to_vec();
    ordered.sort_by_key(|replacement| replacement.0);
    let mut cursor = 0usize;
    let mut output = String::with_capacity(text.len() + 16);
    for (start, end, replacement) in ordered {
        if start < cursor || start > end || end > text.len() {
            return Err(DynamicFileOutputError::Invariant {
                message: "P2.6 source transformation spans overlap or exceed source bounds"
                    .to_owned(),
            });
        }
        output.push_str(&text[cursor..start]);
        output.push_str(replacement);
        cursor = end;
    }
    output.push_str(&text[cursor..]);
    Ok(output)
}

fn static_file_source(
    module: Option<&str>,
    entry_name: &str,
    raw_file_literal: &str,
    output: &str,
) -> String {
    let mut source = String::new();
    if let Some(module) = module {
        source.push_str("module ");
        source.push_str(module);
        source.push_str(";\n");
    }
    source.push_str("effect FileWrite;\n");
    source.push_str("entry ");
    source.push_str(entry_name);
    source.push_str(" writes ");
    source.push_str(raw_file_literal);
    source.push_str(" emits ");
    source.push_str(&quote_noi_string(output));
    source.push_str(";\n");
    source
}

fn quote_noi_string(text: &str) -> String {
    let mut output = String::with_capacity(text.len() + 2);
    output.push('"');
    for ch in text.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn render_dynamic_value(value: DynamicValue) -> String {
    match value {
        DynamicValue::Int(value) => value.to_string(),
        DynamicValue::Bool(value) => {
            if value {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        }
    }
}

fn segment_proof_bytes(
    index: usize,
    segment_plan: &[u8],
    runtime_witness: &[u8],
    input_bytes: &[u8],
    replay_key: u64,
    result: DynamicValue,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P26_SEGMENT_DOMAIN);
    push_u64(&mut out, index as u64);
    push_component(&mut out, segment_plan);
    push_component(&mut out, runtime_witness);
    push_component(&mut out, input_bytes);
    out.extend_from_slice(&replay_key.to_be_bytes());
    encode_dynamic_value(result, &mut out);
    out
}

fn canonical_plan_bytes(
    file_name: &str,
    target_policy_sha256: &[u8; 32],
    render_plan: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P26_PLAN_DOMAIN);
    push_string(&mut out, file_name);
    out.extend_from_slice(target_policy_sha256);
    push_component(&mut out, render_plan);
    out
}

fn canonical_receipt_bytes(
    plan: &P26DynamicFileOutputPlan,
    key_code: u32,
    results: &[DynamicValue],
    output: &str,
    segment_proofs: &[[u8; 32]],
    file_receipt_sha256: &[u8; 32],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P26_RECEIPT_DOMAIN);
    push_component(&mut out, plan.canonical_plan_bytes());
    out.extend_from_slice(&key_code.to_be_bytes());
    push_u64(&mut out, results.len() as u64);
    for result in results {
        encode_dynamic_value(*result, &mut out);
    }
    push_string(&mut out, output);
    push_u64(&mut out, segment_proofs.len() as u64);
    for proof in segment_proofs {
        out.extend_from_slice(proof);
    }
    out.extend_from_slice(file_receipt_sha256);
    out
}

fn encode_dynamic_value(value: DynamicValue, out: &mut Vec<u8>) {
    match value {
        DynamicValue::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        DynamicValue::Bool(value) => {
            out.push(2);
            out.push(u8::from(value));
        }
    }
}

fn push_component(out: &mut Vec<u8>, bytes: &[u8]) {
    push_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn push_string(out: &mut Vec<u8>, text: &str) {
    push_component(out, text.as_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn escape_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            other => vec![other],
        })
        .collect()
}
