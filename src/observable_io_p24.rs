use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::capability::{Capability, CapabilitySet};
use crate::dynamic_input_v07::{DynamicValue, DynamicValueKind};
use crate::effect_audit::hash::sha256;
use crate::frontend::{
    analyze_type_effect_unit, lex, LexError, SourceError, SourceSpan, SourceText,
    SurfaceDeclarationKind, Token, TokenKind, TypeEffectError,
};
use crate::observable_io_p21::{
    P21ObservableEffect, P21ObservableEffectGuard, P21_CONSOLE_EFFECT_NAME,
};
use crate::observable_io_p23::{
    compile_structured_observable_output_plan_p23, execute_structured_observable_output_p23,
    P23StructuredObservableOutputPlan, StructuredObservableIoError,
};

const P24_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.4-MULTI-SEGMENT-STRUCTURED-OUTPUT-PLAN\0";
const P24_RECEIPT_DOMAIN: &[u8] = b"NORDOI-P2.4-MULTI-SEGMENT-STRUCTURED-OUTPUT-RECEIPT\0";

pub const P24_OUTPUT_SCHEMA: &str = "nordoi.observable-io.p2.4";
pub const MAX_P24_OUTPUT_BYTES: usize = 4096;
pub const MAX_P24_RUNTIME_SEGMENTS: usize = 8;
pub const MIN_P24_RUNTIME_SEGMENTS: usize = 2;

#[derive(Debug)]
pub enum MultiSegmentObservableIoError {
    TypeEffect(TypeEffectError),
    Lex(LexError),
    Source(SourceError),
    Syntax {
        message: String,
        span: SourceSpan,
    },
    Policy {
        message: String,
        span: Option<SourceSpan>,
    },
    Segment {
        index: usize,
        message: String,
        span: Option<SourceSpan>,
    },
    CapabilityDenied(Capability),
    Invariant {
        message: String,
    },
}

impl MultiSegmentObservableIoError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) => None,
            Self::Syntax { span, .. } => Some(*span),
            Self::Policy { span, .. } | Self::Segment { span, .. } => *span,
            Self::CapabilityDenied(_) | Self::Invariant { .. } => None,
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::CapabilityDenied(_) | Self::Invariant { .. })
    }
}

impl Display for MultiSegmentObservableIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.4 multi-segment output syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.4 multi-segment output policy error: {message}")
            }
            Self::Segment { index, message, .. } => {
                write!(f, "P2.4 runtime segment {} rejected: {message}", index + 1)
            }
            Self::CapabilityDenied(capability) => write!(
                f,
                "P2.4 multi-segment output authority error: capability denied: {capability:?}"
            ),
            Self::Invariant { message } => {
                write!(f, "P2.4 multi-segment output invariant failed: {message}")
            }
        }
    }
}

impl Error for MultiSegmentObservableIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Syntax { .. }
            | Self::Policy { .. }
            | Self::Segment { .. }
            | Self::CapabilityDenied(_)
            | Self::Invariant { .. } => None,
        }
    }
}

impl From<TypeEffectError> for MultiSegmentObservableIoError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<LexError> for MultiSegmentObservableIoError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for MultiSegmentObservableIoError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type MultiSegmentObservableIoResult<T> = Result<T, MultiSegmentObservableIoError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P24MultiSegmentOutputPlan {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    static_segments: Vec<String>,
    segment_plans: Vec<P23StructuredObservableOutputPlan>,
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P24MultiSegmentOutputPlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn input_name(&self) -> &str {
        &self.input_name
    }

    pub fn runtime_segment_count(&self) -> usize {
        self.segment_plans.len()
    }

    pub fn static_segments(&self) -> &[String] {
        &self.static_segments
    }

    pub fn segment_plans(&self) -> &[P23StructuredObservableOutputPlan] {
        &self.segment_plans
    }

    pub fn result_kinds(&self) -> Vec<DynamicValueKind> {
        self.segment_plans
            .iter()
            .map(P23StructuredObservableOutputPlan::result_kind)
            .collect()
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P24MultiSegmentOutputReceipt {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    key_code: u32,
    results: Vec<DynamicValue>,
    output: String,
    plan_sha256: [u8; 32],
    segment_receipt_sha256: Vec<[u8; 32]>,
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P24MultiSegmentOutputReceipt {
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

    pub fn results(&self) -> &[DynamicValue] {
        &self.results
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

    pub fn segment_receipt_sha256_hex(&self) -> Vec<String> {
        self.segment_receipt_sha256
            .iter()
            .map(|hash| hex_bytes(hash))
            .collect()
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
        let segment_receipts = self
            .segment_receipt_sha256
            .iter()
            .map(|hash| hex_bytes(hash))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "multi-console-run module=\"{}\" entry=\"{}\" input=\"{}\" key-code={} segments={} results=[{}] effect={} capability={} output-bytes={} plan-sha256={} segment-receipts-sha256=[{}] receipt-sha256={} status=EMITTED authority=EXPLICIT grant=ConsoleWrite ambient-authority=NONE\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            escape_text(&self.input_name),
            self.key_code,
            self.results.len(),
            results,
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            self.output_bytes(),
            self.plan_sha256_hex(),
            segment_receipts,
            self.receipt_sha256_hex(),
        )
    }

    pub fn render_json(&self) -> String {
        let results = self
            .results
            .iter()
            .map(|value| {
                format!(
                    "{{\"kind\":\"{}\",\"value\":\"{}\"}}",
                    value.kind().as_str(),
                    escape_json(&value.as_text())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let segment_receipts = self
            .segment_receipt_sha256
            .iter()
            .map(|hash| format!("\"{}\"", hex_bytes(hash)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"{}\",\"status\":\"emitted\",\"module\":\"{}\",\"entry\":\"{}\",\"input\":\"{}\",\"keyCode\":{},\"segments\":{},\"results\":[{}],\"effect\":\"{}\",\"capability\":\"{}\",\"output\":\"{}\",\"outputBytes\":{},\"planSha256\":\"{}\",\"segmentReceiptSha256\":[{}],\"receiptSha256\":\"{}\",\"authority\":\"EXPLICIT\",\"ambientAuthority\":\"NONE\"}}",
            P24_OUTPUT_SCHEMA,
            escape_json(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_json(&self.entry_name),
            escape_json(&self.input_name),
            self.key_code,
            self.results.len(),
            results,
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            escape_json(&self.output),
            self.output_bytes(),
            self.plan_sha256_hex(),
            segment_receipts,
            self.receipt_sha256_hex(),
        )
    }
}

pub fn compile_multi_segment_output_plan_p24(
    source: &SourceText,
) -> MultiSegmentObservableIoResult<P24MultiSegmentOutputPlan> {
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
        return Err(MultiSegmentObservableIoError::Policy {
            message: "P2.4 requires exactly one entry declaration".to_owned(),
            span: entry_indexes
                .first()
                .and_then(|index| significant.get(*index))
                .map(|token| token.span())
                .or(Some(unit.body_span())),
        });
    }

    let entry_index = entry_indexes[0];
    let emits_token = match significant.get(entry_index + 2) {
        Some(token) => token,
        None => {
            return Err(MultiSegmentObservableIoError::Syntax {
                message:
                    "expected: entry <name> emits \"prefix\" + <dynamic> + \"text\" + <dynamic> ...;"
                        .to_owned(),
                span: significant[entry_index].span(),
            });
        }
    };
    if emits_token.kind() != &TokenKind::Identifier || source.slice(emits_token.span())? != "emits"
    {
        return Err(MultiSegmentObservableIoError::Syntax {
            message: "expected contextual 'emits' after the entry name".to_owned(),
            span: emits_token.span(),
        });
    }

    let semicolon_index = significant
        .iter()
        .enumerate()
        .skip(entry_index + 3)
        .find_map(|(index, token)| (token.kind() == &TokenKind::Punctuation(';')).then_some(index))
        .ok_or_else(|| MultiSegmentObservableIoError::Syntax {
            message: "multi-segment output entry must end with ';'".to_owned(),
            span: emits_token.span(),
        })?;

    let expression_tokens = &significant[entry_index + 3..semicolon_index];
    if expression_tokens.len() < 7 {
        return Err(MultiSegmentObservableIoError::Policy {
            message:
                "P2.4 requires at least two runtime segments; use P2.3 for one runtime segment"
                    .to_owned(),
            span: expression_tokens
                .first()
                .map(|token| token.span())
                .or(Some(emits_token.span())),
        });
    }
    if expression_tokens[0].kind() != &TokenKind::QuotedText {
        return Err(MultiSegmentObservableIoError::Policy {
            message: "P2.4 output must begin with one quoted UTF-8 static segment".to_owned(),
            span: Some(expression_tokens[0].span()),
        });
    }
    if expression_tokens[1].kind() != &TokenKind::Punctuation('+') {
        return Err(MultiSegmentObservableIoError::Syntax {
            message: "expected '+' after the first quoted static segment".to_owned(),
            span: expression_tokens[1].span(),
        });
    }

    let mut static_segments = vec![decode_quoted_output(source, expression_tokens[0])?];
    let mut segment_ranges = Vec::new();
    let mut cursor = 2usize;

    while cursor < expression_tokens.len() {
        let next_quote = expression_tokens[cursor..]
            .iter()
            .position(|token| token.kind() == &TokenKind::QuotedText)
            .map(|offset| cursor + offset);

        match next_quote {
            Some(quote_index) => {
                if quote_index == cursor {
                    return Err(MultiSegmentObservableIoError::Syntax {
                        message: "runtime segment must not be empty".to_owned(),
                        span: expression_tokens[quote_index].span(),
                    });
                }
                let separator = quote_index - 1;
                if expression_tokens[separator].kind() != &TokenKind::Punctuation('+') {
                    return Err(MultiSegmentObservableIoError::Syntax {
                        message: "quoted static segments must be separated from runtime expressions by '+'"
                            .to_owned(),
                        span: expression_tokens[quote_index].span(),
                    });
                }
                if separator == cursor {
                    return Err(MultiSegmentObservableIoError::Syntax {
                        message: "runtime segment must not be empty".to_owned(),
                        span: expression_tokens[separator].span(),
                    });
                }
                segment_ranges.push((cursor, separator));
                static_segments.push(decode_quoted_output(
                    source,
                    expression_tokens[quote_index],
                )?);
                cursor = quote_index + 1;
                if cursor == expression_tokens.len() {
                    break;
                }
                if expression_tokens[cursor].kind() != &TokenKind::Punctuation('+') {
                    return Err(MultiSegmentObservableIoError::Syntax {
                        message: "expected '+' after quoted static segment".to_owned(),
                        span: expression_tokens[cursor].span(),
                    });
                }
                cursor += 1;
                if cursor == expression_tokens.len() {
                    return Err(MultiSegmentObservableIoError::Syntax {
                        message: "trailing '+' requires another runtime segment".to_owned(),
                        span: expression_tokens[cursor - 1].span(),
                    });
                }
            }
            None => {
                segment_ranges.push((cursor, expression_tokens.len()));
                static_segments.push(String::new());
                cursor = expression_tokens.len();
            }
        }
    }

    if segment_ranges.len() < MIN_P24_RUNTIME_SEGMENTS {
        return Err(MultiSegmentObservableIoError::Policy {
            message:
                "P2.4 requires at least two runtime segments; use P2.3 for one runtime segment"
                    .to_owned(),
            span: Some(expression_tokens[0].span()),
        });
    }
    if segment_ranges.len() > MAX_P24_RUNTIME_SEGMENTS {
        return Err(MultiSegmentObservableIoError::Policy {
            message: format!(
                "P2.4 permits at most {MAX_P24_RUNTIME_SEGMENTS} runtime segments; found {}",
                segment_ranges.len()
            ),
            span: Some(expression_tokens[0].span()),
        });
    }
    if static_segments.len() != segment_ranges.len() + 1 {
        return Err(MultiSegmentObservableIoError::Invariant {
            message: "static/runtime segment arity is not canonical".to_owned(),
        });
    }

    let semicolon_span = significant[semicolon_index].span();
    let mut segment_plans = Vec::with_capacity(segment_ranges.len());
    for (index, (start, end)) in segment_ranges.iter().copied().enumerate() {
        if start >= end {
            return Err(MultiSegmentObservableIoError::Syntax {
                message: "runtime segment must not be empty".to_owned(),
                span: expression_tokens[start.min(expression_tokens.len() - 1)].span(),
            });
        }
        let runtime_start = expression_tokens[start].span().start().get() as usize;
        let runtime_end = expression_tokens[end - 1].span().end().get() as usize;
        let segment_source = build_segment_source(
            source,
            emits_token.span(),
            runtime_start,
            runtime_end,
            semicolon_span,
        )?;
        let synthetic = SourceText::new(
            source.id(),
            format!("<p2.4-segment-{}>", index + 1),
            segment_source,
        )?;
        let plan = compile_structured_observable_output_plan_p23(&synthetic).map_err(|error| {
            MultiSegmentObservableIoError::Segment {
                index,
                message: error.to_string(),
                span: Some(expression_tokens[start].span()),
            }
        })?;
        if !plan.prefix().is_empty() || !plan.suffix().is_empty() {
            return Err(MultiSegmentObservableIoError::Invariant {
                message: format!(
                    "segment {} did not canonicalize to empty P2.3 static text",
                    index + 1
                ),
            });
        }
        segment_plans.push(plan);
    }

    let first = &segment_plans[0];
    let module = first.module().map(str::to_owned);
    let entry_name = first.entry_name().to_owned();
    let input_name = first.input_name().to_owned();
    for (index, plan) in segment_plans.iter().enumerate().skip(1) {
        if plan.module() != module.as_deref()
            || plan.entry_name() != entry_name
            || plan.input_name() != input_name
        {
            return Err(MultiSegmentObservableIoError::Invariant {
                message: format!(
                    "segment {} resolved a different module/entry/input identity",
                    index + 1
                ),
            });
        }
    }

    let static_bytes = static_segments
        .iter()
        .fold(0usize, |total, text| total.saturating_add(text.len()));
    let runtime_bytes = segment_plans.iter().fold(0usize, |total, plan| {
        total.saturating_add(maximum_rendered_bytes(plan.result_kind()))
    });
    let maximum_output = static_bytes.saturating_add(runtime_bytes);
    if maximum_output > MAX_P24_OUTPUT_BYTES {
        return Err(MultiSegmentObservableIoError::Policy {
            message: format!(
                "multi-segment output can require {maximum_output} UTF-8 bytes, exceeding bound {MAX_P24_OUTPUT_BYTES}"
            ),
            span: Some(expression_tokens[0].span()),
        });
    }

    let canonical_plan = canonical_plan_bytes(&static_segments, &segment_plans)?;
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P24MultiSegmentOutputPlan {
        module,
        entry_name,
        input_name,
        static_segments,
        segment_plans,
        canonical_plan,
        plan_sha256,
    })
}

pub fn execute_multi_segment_output_p24(
    plan: &P24MultiSegmentOutputPlan,
    key_code: u32,
    authority: &CapabilitySet,
) -> MultiSegmentObservableIoResult<P24MultiSegmentOutputReceipt> {
    let guard = P21ObservableEffectGuard::new(true, authority.clone());
    guard
        .check(P21ObservableEffect::ConsoleWrite)
        .map_err(|error| match error {
            crate::observable_io_p21::ObservableIoError::CapabilityDenied(capability) => {
                MultiSegmentObservableIoError::CapabilityDenied(capability)
            }
            other => MultiSegmentObservableIoError::Invariant {
                message: format!("P2.1 authority guard failed unexpectedly: {other}"),
            },
        })?;

    let mut results = Vec::with_capacity(plan.segment_plans.len());
    let mut segment_receipt_sha256 = Vec::with_capacity(plan.segment_plans.len());
    let mut rendered = Vec::with_capacity(plan.segment_plans.len());

    for (index, segment_plan) in plan.segment_plans.iter().enumerate() {
        let receipt = execute_structured_observable_output_p23(segment_plan, key_code, authority)
            .map_err(|error| match error {
            StructuredObservableIoError::CapabilityDenied(capability) => {
                MultiSegmentObservableIoError::CapabilityDenied(capability)
            }
            other => MultiSegmentObservableIoError::Invariant {
                message: format!(
                    "certified P2.3 segment {} execution failed: {other}",
                    index + 1
                ),
            },
        })?;
        let result = receipt.result();
        let expected = render_dynamic_value(result);
        if receipt.output() != expected {
            return Err(MultiSegmentObservableIoError::Invariant {
                message: format!(
                    "segment {} P2.3 output is not canonical runtime text",
                    index + 1
                ),
            });
        }
        results.push(result);
        rendered.push(expected);
        segment_receipt_sha256.push(sha256(receipt.canonical_receipt_bytes()));
    }

    let mut output = String::new();
    for (static_segment, rendered_segment) in plan.static_segments.iter().zip(rendered.iter()) {
        output.push_str(static_segment);
        output.push_str(rendered_segment);
    }
    output.push_str(&plan.static_segments[rendered.len()]);
    if output.len() > MAX_P24_OUTPUT_BYTES {
        return Err(MultiSegmentObservableIoError::Invariant {
            message: format!(
                "rendered multi-segment output has {} bytes, exceeding bound {MAX_P24_OUTPUT_BYTES}",
                output.len()
            ),
        });
    }

    let canonical_receipt =
        canonical_receipt_bytes(plan, key_code, &results, &output, &segment_receipt_sha256)?;
    let receipt_sha256 = sha256(&canonical_receipt);

    Ok(P24MultiSegmentOutputReceipt {
        module: plan.module.clone(),
        entry_name: plan.entry_name.clone(),
        input_name: plan.input_name.clone(),
        key_code,
        results,
        output,
        plan_sha256: plan.plan_sha256,
        segment_receipt_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

pub fn execute_multi_segment_output_source_p24(
    source: &SourceText,
    key_code: u32,
    authority: &CapabilitySet,
) -> MultiSegmentObservableIoResult<P24MultiSegmentOutputReceipt> {
    let plan = compile_multi_segment_output_plan_p24(source)?;
    execute_multi_segment_output_p24(&plan, key_code, authority)
}

fn validate_effect_prelude(
    source: &SourceText,
    unit: &crate::frontend::TypeEffectUnit,
) -> MultiSegmentObservableIoResult<()> {
    let mut console_write = None;
    for declaration in unit.declarations() {
        if declaration.kind() != SurfaceDeclarationKind::Effect {
            continue;
        }
        let name = declaration.name().text(source)?;
        if name != P21_CONSOLE_EFFECT_NAME {
            return Err(MultiSegmentObservableIoError::Policy {
                message: format!(
                    "P2.4 permits only effect {P21_CONSOLE_EFFECT_NAME}; found '{name}'"
                ),
                span: Some(declaration.span()),
            });
        }
        if console_write.is_some() {
            return Err(MultiSegmentObservableIoError::Policy {
                message: format!("duplicate effect {P21_CONSOLE_EFFECT_NAME}"),
                span: Some(declaration.span()),
            });
        }
        console_write = Some(declaration.span());
    }

    if console_write.is_none() {
        return Err(MultiSegmentObservableIoError::Policy {
            message: format!(
                "multi-segment console output requires explicit 'effect {P21_CONSOLE_EFFECT_NAME};'"
            ),
            span: Some(unit.body_span()),
        });
    }
    Ok(())
}

fn decode_quoted_output(
    source: &SourceText,
    token: &Token,
) -> MultiSegmentObservableIoResult<String> {
    let span = token.span();
    let raw = source.slice(span)?;
    let bytes = raw.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(MultiSegmentObservableIoError::Syntax {
            message: "invalid quoted multi-segment output token".to_owned(),
            span,
        });
    }

    let content = &raw[1..raw.len() - 1];
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| MultiSegmentObservableIoError::Syntax {
                message: "trailing escape in multi-segment output".to_owned(),
                span,
            })?;
        match escaped {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            _ => {
                return Err(MultiSegmentObservableIoError::Syntax {
                    message: format!(
                        "unsupported output escape '\\{escaped}'; allowed escapes are \\\\, \\\" , \\n, \\r and \\t"
                    ),
                    span,
                })
            }
        }
    }
    Ok(output)
}

fn build_segment_source(
    source: &SourceText,
    emits_span: SourceSpan,
    runtime_start: usize,
    runtime_end: usize,
    semicolon_span: SourceSpan,
) -> MultiSegmentObservableIoResult<String> {
    let text = source.text();
    let emits_start = emits_span.start().get() as usize;
    let semicolon_start = semicolon_span.start().get() as usize;
    if emits_start > runtime_start
        || runtime_start >= runtime_end
        || runtime_end > semicolon_start
        || semicolon_start > text.len()
    {
        return Err(MultiSegmentObservableIoError::Invariant {
            message: "invalid multi-segment source spans".to_owned(),
        });
    }

    let mut transformed = String::with_capacity(text.len());
    transformed.push_str(&text[..emits_start]);
    transformed.push_str("emits \"\" + ");
    transformed.push_str(&text[runtime_start..runtime_end]);
    transformed.push_str(&text[semicolon_start..]);
    Ok(transformed)
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

const fn maximum_rendered_bytes(kind: DynamicValueKind) -> usize {
    match kind {
        DynamicValueKind::Int => 20,
        DynamicValueKind::Bool => 5,
    }
}

fn canonical_plan_bytes(
    static_segments: &[String],
    segment_plans: &[P23StructuredObservableOutputPlan],
) -> MultiSegmentObservableIoResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P24_PLAN_DOMAIN);
    push_string(&mut bytes, P21_CONSOLE_EFFECT_NAME)?;
    push_u32(&mut bytes, segment_plans.len(), "runtime segment count")?;
    for (index, plan) in segment_plans.iter().enumerate() {
        push_string(&mut bytes, &static_segments[index])?;
        push_component(&mut bytes, plan.canonical_plan_bytes())?;
        push_string(&mut bytes, plan.result_kind().as_str())?;
    }
    push_string(&mut bytes, &static_segments[segment_plans.len()])?;
    bytes.extend_from_slice(&(MAX_P24_OUTPUT_BYTES as u32).to_be_bytes());
    bytes.extend_from_slice(&(MAX_P24_RUNTIME_SEGMENTS as u32).to_be_bytes());
    Ok(bytes)
}

fn canonical_receipt_bytes(
    plan: &P24MultiSegmentOutputPlan,
    key_code: u32,
    results: &[DynamicValue],
    output: &str,
    segment_receipt_sha256: &[[u8; 32]],
) -> MultiSegmentObservableIoResult<Vec<u8>> {
    if results.len() != plan.segment_plans.len()
        || segment_receipt_sha256.len() != plan.segment_plans.len()
    {
        return Err(MultiSegmentObservableIoError::Invariant {
            message: "receipt segment arity differs from certified P2.4 plan".to_owned(),
        });
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(P24_RECEIPT_DOMAIN);
    bytes.extend_from_slice(&plan.plan_sha256);
    bytes.extend_from_slice(&key_code.to_be_bytes());
    push_string(&mut bytes, plan.module.as_deref().unwrap_or("<anonymous>"))?;
    push_string(&mut bytes, &plan.entry_name)?;
    push_string(&mut bytes, &plan.input_name)?;
    push_u32(&mut bytes, results.len(), "receipt segment count")?;
    for (result, hash) in results.iter().zip(segment_receipt_sha256.iter()) {
        bytes.extend_from_slice(hash);
        push_string(&mut bytes, result.kind().as_str())?;
        push_string(&mut bytes, &result.as_text())?;
    }
    push_string(&mut bytes, output)?;
    push_string(&mut bytes, "EXPLICIT")?;
    push_string(&mut bytes, "NONE")?;
    Ok(bytes)
}

fn push_u32(out: &mut Vec<u8>, value: usize, label: &str) -> MultiSegmentObservableIoResult<()> {
    let value = u32::try_from(value).map_err(|_| MultiSegmentObservableIoError::Invariant {
        message: format!("P2.4 {label} exceeded u32"),
    })?;
    out.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn push_component(out: &mut Vec<u8>, value: &[u8]) -> MultiSegmentObservableIoResult<()> {
    let len = u32::try_from(value.len()).map_err(|_| MultiSegmentObservableIoError::Invariant {
        message: "P2.4 canonical component exceeded u32 length".to_owned(),
    })?;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn push_string(out: &mut Vec<u8>, value: &str) -> MultiSegmentObservableIoResult<()> {
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
