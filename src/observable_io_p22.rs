use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::capability::{Capability, CapabilitySet};
use crate::dynamic_input_v07::{
    compile_dynamic_plan_v07, execute_dynamic_source_v07, DynamicInputError, DynamicValue,
    DynamicValueKind, V07DynamicPlan,
};
use crate::effect_audit::hash::sha256;
use crate::frontend::{
    analyze_type_effect_unit, lex, LexError, SourceError, SourceSpan, SourceText,
    SurfaceDeclarationKind, TokenKind, TypeEffectError,
};
use crate::input::{
    InputBatch, InputDeviceId, InputEvent, InputPayload, InputSequence, InputSource, InputTarget,
};
use crate::observable_io_p21::{
    P21ObservableEffect, P21ObservableEffectGuard, P21_CONSOLE_EFFECT_NAME,
};

const P22_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.2-DYNAMIC-CAPABILITY-SECURED-OBSERVABLE-OUTPUT-PLAN\0";
const P22_RECEIPT_DOMAIN: &[u8] =
    b"NORDOI-P2.2-DYNAMIC-CAPABILITY-SECURED-OBSERVABLE-OUTPUT-RECEIPT\0";

pub const P22_OUTPUT_SCHEMA: &str = "nordoi.observable-io.p2.2";
pub const MAX_P22_OUTPUT_BYTES: usize = 64;

#[derive(Debug)]
pub enum DynamicObservableIoError {
    TypeEffect(TypeEffectError),
    Lex(LexError),
    Source(SourceError),
    Dynamic(DynamicInputError),
    Syntax {
        message: String,
        span: SourceSpan,
    },
    Policy {
        message: String,
        span: Option<SourceSpan>,
    },
    CapabilityDenied(Capability),
    Invariant {
        message: String,
    },
}

impl DynamicObservableIoError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) => None,
            Self::Dynamic(error) => error.primary_span(),
            Self::Syntax { span, .. } => Some(*span),
            Self::Policy { span, .. } => *span,
            Self::CapabilityDenied(_) | Self::Invariant { .. } => None,
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::CapabilityDenied(_) | Self::Invariant { .. })
    }
}

impl Display for DynamicObservableIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Dynamic(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.2 dynamic observable output syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.2 dynamic observable output policy error: {message}")
            }
            Self::CapabilityDenied(capability) => write!(
                f,
                "P2.2 dynamic observable output authority error: capability denied: {capability:?}"
            ),
            Self::Invariant { message } => {
                write!(
                    f,
                    "P2.2 dynamic observable output invariant failed: {message}"
                )
            }
        }
    }
}

impl Error for DynamicObservableIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Dynamic(error) => Some(error),
            Self::Syntax { .. }
            | Self::Policy { .. }
            | Self::CapabilityDenied(_)
            | Self::Invariant { .. } => None,
        }
    }
}

impl From<TypeEffectError> for DynamicObservableIoError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<LexError> for DynamicObservableIoError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for DynamicObservableIoError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<DynamicInputError> for DynamicObservableIoError {
    fn from(value: DynamicInputError) -> Self {
        Self::Dynamic(value)
    }
}

pub type DynamicObservableIoResult<T> = Result<T, DynamicObservableIoError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P22DynamicObservableOutputPlan {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    result_kind: DynamicValueKind,
    runtime_source: String,
    dynamic_plan: V07DynamicPlan,
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P22DynamicObservableOutputPlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn input_name(&self) -> &str {
        &self.input_name
    }

    pub fn result_kind(&self) -> DynamicValueKind {
        self.result_kind
    }

    pub fn canonical_plan_bytes(&self) -> &[u8] {
        &self.canonical_plan
    }

    pub fn plan_sha256(&self) -> &[u8; 32] {
        &self.plan_sha256
    }

    pub fn plan_sha256_hex(&self) -> String {
        hex_bytes(&self.plan_sha256)
    }

    pub fn dynamic_plan(&self) -> &V07DynamicPlan {
        &self.dynamic_plan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P22DynamicObservableOutputReceipt {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    key_code: u32,
    result: DynamicValue,
    output: String,
    plan_sha256: [u8; 32],
    runtime_receipt_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P22DynamicObservableOutputReceipt {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn input_name(&self) -> &str {
        &self.input_name
    }

    pub fn key_code(&self) -> u32 {
        self.key_code
    }

    pub fn result(&self) -> DynamicValue {
        self.result
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn output_bytes(&self) -> usize {
        self.output.len()
    }

    pub fn plan_sha256_hex(&self) -> String {
        hex_bytes(&self.plan_sha256)
    }

    pub fn runtime_receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.runtime_receipt_sha256)
    }

    pub fn receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.receipt_sha256)
    }

    pub fn canonical_receipt_bytes(&self) -> &[u8] {
        &self.canonical_receipt
    }

    pub fn render_text(&self) -> String {
        format!(
            "dynamic-console-run module=\"{}\" entry=\"{}\" input=\"{}\" key-code={} result={} effect={} capability={} output-bytes={} plan-sha256={} runtime-receipt-sha256={} receipt-sha256={} status=EMITTED authority=EXPLICIT grant=ConsoleWrite ambient-authority=NONE\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            escape_text(&self.input_name),
            self.key_code,
            self.result.as_text(),
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            self.output_bytes(),
            self.plan_sha256_hex(),
            self.runtime_receipt_sha256_hex(),
            self.receipt_sha256_hex(),
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"status\":\"emitted\",\"module\":\"{}\",\"entry\":\"{}\",\"input\":\"{}\",\"keyCode\":{},\"resultKind\":\"{}\",\"result\":\"{}\",\"effect\":\"{}\",\"capability\":\"{}\",\"output\":\"{}\",\"outputBytes\":{},\"planSha256\":\"{}\",\"runtimeReceiptSha256\":\"{}\",\"receiptSha256\":\"{}\",\"authority\":\"EXPLICIT\",\"ambientAuthority\":\"NONE\"}}",
            P22_OUTPUT_SCHEMA,
            escape_json(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_json(&self.entry_name),
            escape_json(&self.input_name),
            self.key_code,
            self.result.kind().as_str(),
            escape_json(&self.result.as_text()),
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            escape_json(&self.output),
            self.output_bytes(),
            self.plan_sha256_hex(),
            self.runtime_receipt_sha256_hex(),
            self.receipt_sha256_hex(),
        )
    }
}

pub fn compile_dynamic_observable_output_plan_p22(
    source: &SourceText,
) -> DynamicObservableIoResult<P22DynamicObservableOutputPlan> {
    let unit = analyze_type_effect_unit(source)?;
    validate_effect_prelude(source, &unit)?;

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
        return Err(DynamicObservableIoError::Policy {
            message: "P2.2 requires exactly one entry declaration".to_owned(),
            span: entry_indexes
                .first()
                .and_then(|index| significant.get(*index))
                .map(|token| token.span())
                .or(Some(unit.body_span())),
        });
    }

    let entry_index = entry_indexes[0];
    let emits_token =
        significant
            .get(entry_index + 2)
            .ok_or_else(|| DynamicObservableIoError::Syntax {
                message: "expected: entry <name> emits <dynamic-expression>;".to_owned(),
                span: significant[entry_index].span(),
            })?;
    if emits_token.kind() != &TokenKind::Identifier || source.slice(emits_token.span())? != "emits"
    {
        return Err(DynamicObservableIoError::Syntax {
            message: "expected contextual 'emits' after the entry name".to_owned(),
            span: emits_token.span(),
        });
    }

    if significant
        .iter()
        .skip(entry_index + 3)
        .any(|token| token.kind() == &TokenKind::QuotedText)
    {
        return Err(DynamicObservableIoError::Policy {
            message:
                "P2.2 dynamic output accepts runtime Int/Bool expressions; quoted text remains P2.1"
                    .to_owned(),
            span: significant
                .iter()
                .skip(entry_index + 3)
                .find(|token| token.kind() == &TokenKind::QuotedText)
                .map(|token| token.span()),
        });
    }

    let runtime_source = replace_span(source, emits_token.span(), "returns")?;
    let synthetic = SourceText::new(source.id(), source.name(), runtime_source.clone())?;
    let dynamic_plan = compile_dynamic_plan_v07(&synthetic)?;

    if !dynamic_plan.is_dynamic() {
        return Err(DynamicObservableIoError::Policy {
            message: "P2.2 requires output to depend on explicit runtime input".to_owned(),
            span: Some(emits_token.span()),
        });
    }
    let input_name = dynamic_plan
        .input_name()
        .ok_or_else(|| DynamicObservableIoError::Policy {
            message: "P2.2 requires exactly one explicit runtime input".to_owned(),
            span: Some(unit.body_span()),
        })?
        .to_owned();

    let module = dynamic_plan.module().map(str::to_owned);
    let entry_name = dynamic_plan.entry_name().to_owned();
    let result_kind = dynamic_plan.result_kind();
    let canonical_plan = canonical_plan_bytes(&dynamic_plan)?;
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P22DynamicObservableOutputPlan {
        module,
        entry_name,
        input_name,
        result_kind,
        runtime_source,
        dynamic_plan,
        canonical_plan,
        plan_sha256,
    })
}

pub fn execute_dynamic_observable_output_p22(
    plan: &P22DynamicObservableOutputPlan,
    key_code: u32,
    authority: &CapabilitySet,
) -> DynamicObservableIoResult<P22DynamicObservableOutputReceipt> {
    let guard = P21ObservableEffectGuard::new(true, authority.clone());
    guard
        .check(P21ObservableEffect::ConsoleWrite)
        .map_err(|error| match error {
            crate::observable_io_p21::ObservableIoError::CapabilityDenied(capability) => {
                DynamicObservableIoError::CapabilityDenied(capability)
            }
            other => DynamicObservableIoError::Invariant {
                message: format!("P2.1 authority guard failed unexpectedly: {other}"),
            },
        })?;

    let synthetic = SourceText::new(
        crate::frontend::SourceId::new(0x2202),
        "<p2.2-runtime>",
        plan.runtime_source.clone(),
    )?;
    let input = key_batch(key_code);
    let runtime = execute_dynamic_source_v07(&synthetic, &input)?;

    if runtime.plan().canonical_v07_semantic_bytes()
        != plan.dynamic_plan.canonical_v07_semantic_bytes()
    {
        return Err(DynamicObservableIoError::Invariant {
            message: "runtime dynamic semantics differ from the certified P2.2 plan".to_owned(),
        });
    }
    if runtime.plan().result_kind() != plan.result_kind {
        return Err(DynamicObservableIoError::Invariant {
            message: "runtime result kind differs from the certified P2.2 plan".to_owned(),
        });
    }

    let result = runtime.result();
    let output = render_dynamic_value(result);
    if output.len() > MAX_P22_OUTPUT_BYTES {
        return Err(DynamicObservableIoError::Invariant {
            message: format!(
                "rendered dynamic output has {} bytes, exceeding bound {MAX_P22_OUTPUT_BYTES}",
                output.len()
            ),
        });
    }

    let runtime_receipt_sha256 = sha256(runtime.canonical_v07_receipt_bytes());
    let canonical_receipt =
        canonical_receipt_bytes(plan, key_code, result, &output, &runtime_receipt_sha256)?;
    let receipt_sha256 = sha256(&canonical_receipt);

    Ok(P22DynamicObservableOutputReceipt {
        module: plan.module.clone(),
        entry_name: plan.entry_name.clone(),
        input_name: plan.input_name.clone(),
        key_code,
        result,
        output,
        plan_sha256: plan.plan_sha256,
        runtime_receipt_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

pub fn execute_dynamic_observable_output_source_p22(
    source: &SourceText,
    key_code: u32,
    authority: &CapabilitySet,
) -> DynamicObservableIoResult<P22DynamicObservableOutputReceipt> {
    let plan = compile_dynamic_observable_output_plan_p22(source)?;
    execute_dynamic_observable_output_p22(&plan, key_code, authority)
}

fn validate_effect_prelude(
    source: &SourceText,
    unit: &crate::frontend::TypeEffectUnit,
) -> DynamicObservableIoResult<()> {
    let mut console_write = None;
    for declaration in unit.declarations() {
        if declaration.kind() != SurfaceDeclarationKind::Effect {
            continue;
        }
        let name = declaration.name().text(source)?;
        if name != P21_CONSOLE_EFFECT_NAME {
            return Err(DynamicObservableIoError::Policy {
                message: format!(
                    "P2.2 permits only effect {P21_CONSOLE_EFFECT_NAME}; found '{name}'"
                ),
                span: Some(declaration.span()),
            });
        }
        if console_write.is_some() {
            return Err(DynamicObservableIoError::Policy {
                message: format!("duplicate effect {P21_CONSOLE_EFFECT_NAME}"),
                span: Some(declaration.span()),
            });
        }
        console_write = Some(declaration.span());
    }

    if console_write.is_none() {
        return Err(DynamicObservableIoError::Policy {
            message: format!(
                "dynamic console output requires explicit 'effect {P21_CONSOLE_EFFECT_NAME};'"
            ),
            span: Some(unit.body_span()),
        });
    }
    Ok(())
}

fn replace_span(
    source: &SourceText,
    span: SourceSpan,
    replacement: &str,
) -> DynamicObservableIoResult<String> {
    let start = span.start().get() as usize;
    let end = span.end().get() as usize;
    let text = source.text();
    if start > end || end > text.len() {
        return Err(DynamicObservableIoError::Invariant {
            message: "invalid contextual emits span".to_owned(),
        });
    }
    let mut transformed = String::with_capacity(text.len() + replacement.len());
    transformed.push_str(&text[..start]);
    transformed.push_str(replacement);
    transformed.push_str(&text[end..]);
    Ok(transformed)
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

fn canonical_plan_bytes(plan: &V07DynamicPlan) -> DynamicObservableIoResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P22_PLAN_DOMAIN);
    push_component(&mut bytes, plan.canonical_v07_semantic_bytes())?;
    push_string(&mut bytes, P21_CONSOLE_EFFECT_NAME)?;
    push_string(&mut bytes, plan.result_kind().as_str())?;
    bytes.extend_from_slice(&(MAX_P22_OUTPUT_BYTES as u32).to_be_bytes());
    Ok(bytes)
}

fn canonical_receipt_bytes(
    plan: &P22DynamicObservableOutputPlan,
    key_code: u32,
    result: DynamicValue,
    output: &str,
    runtime_receipt_sha256: &[u8; 32],
) -> DynamicObservableIoResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P22_RECEIPT_DOMAIN);
    bytes.extend_from_slice(&plan.plan_sha256);
    bytes.extend_from_slice(runtime_receipt_sha256);
    bytes.extend_from_slice(&key_code.to_be_bytes());
    push_string(&mut bytes, plan.module.as_deref().unwrap_or("<anonymous>"))?;
    push_string(&mut bytes, &plan.entry_name)?;
    push_string(&mut bytes, &plan.input_name)?;
    push_string(&mut bytes, result.kind().as_str())?;
    push_string(&mut bytes, &result.as_text())?;
    push_string(&mut bytes, output)?;
    push_string(&mut bytes, "EXPLICIT")?;
    push_string(&mut bytes, "NONE")?;
    Ok(bytes)
}

fn push_component(out: &mut Vec<u8>, value: &[u8]) -> DynamicObservableIoResult<()> {
    let len = u32::try_from(value.len()).map_err(|_| DynamicObservableIoError::Invariant {
        message: "P2.2 canonical component exceeded u32 length".to_owned(),
    })?;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn push_string(out: &mut Vec<u8>, value: &str) -> DynamicObservableIoResult<()> {
    push_component(out, value.as_bytes())
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}

fn escape_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(&mut escaped, "\\u{:04x}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}
