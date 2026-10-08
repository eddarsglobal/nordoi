use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::capability::{Capability, CapabilitySet};
use crate::effect_audit::hash::sha256;
use crate::frontend::{
    analyze_type_effect_unit, lex, LexError, SourceError, SourceSpan, SourceText,
    SurfaceDeclarationKind, Token, TokenKind, TypeEffectError,
};

const P21_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.1-CAPABILITY-SECURED-OBSERVABLE-IO-PLAN\0";
const P21_RECEIPT_DOMAIN: &[u8] = b"NORDOI-P2.1-CAPABILITY-SECURED-OBSERVABLE-IO-RECEIPT\0";

pub const P21_CONSOLE_EFFECT_NAME: &str = "ConsoleWrite";
pub const P21_OUTPUT_SCHEMA: &str = "nordoi.observable-io.p2.1";
pub const MAX_P21_OUTPUT_BYTES: usize = 4096;
pub const MAX_P21_ENTRY_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum P21ObservableEffect {
    ConsoleWrite,
}

impl P21ObservableEffect {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ConsoleWrite => P21_CONSOLE_EFFECT_NAME,
        }
    }
}

pub fn required_observable_capability_p21(effect: P21ObservableEffect) -> Capability {
    match effect {
        P21ObservableEffect::ConsoleWrite => Capability::ConsoleWrite,
    }
}

#[derive(Debug, Clone)]
pub struct P21ObservableEffectGuard {
    declared: bool,
    authority: CapabilitySet,
}

impl P21ObservableEffectGuard {
    pub fn new(declared: bool, authority: CapabilitySet) -> Self {
        Self {
            declared,
            authority,
        }
    }

    pub fn check(&self, effect: P21ObservableEffect) -> ObservableIoResult<()> {
        if !self.declared {
            return Err(ObservableIoError::Policy {
                message: format!("effect {} was not declared", effect.name()),
                span: None,
            });
        }
        let capability = required_observable_capability_p21(effect);
        if self.authority.contains(&capability) {
            Ok(())
        } else {
            Err(ObservableIoError::CapabilityDenied(capability))
        }
    }
}

#[derive(Debug)]
pub enum ObservableIoError {
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
    CapabilityDenied(Capability),
    Invariant {
        message: String,
    },
}

impl ObservableIoError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) => None,
            Self::Syntax { span, .. } => Some(*span),
            Self::Policy { span, .. } => *span,
            Self::CapabilityDenied(_) | Self::Invariant { .. } => None,
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::CapabilityDenied(_) | Self::Invariant { .. })
    }
}

impl Display for ObservableIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.1 observable I/O syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.1 observable I/O policy error: {message}")
            }
            Self::CapabilityDenied(capability) => {
                write!(
                    f,
                    "P2.1 observable I/O authority error: capability denied: {capability:?}"
                )
            }
            Self::Invariant { message } => {
                write!(f, "P2.1 observable I/O invariant failed: {message}")
            }
        }
    }
}

impl Error for ObservableIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Syntax { .. }
            | Self::Policy { .. }
            | Self::CapabilityDenied(_)
            | Self::Invariant { .. } => None,
        }
    }
}

impl From<TypeEffectError> for ObservableIoError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<LexError> for ObservableIoError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for ObservableIoError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type ObservableIoResult<T> = Result<T, ObservableIoError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P21ObservableOutputPlan {
    module: Option<String>,
    entry_name: String,
    output: String,
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P21ObservableOutputPlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn output_bytes(&self) -> usize {
        self.output.len()
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
pub struct P21ObservableOutputReceipt {
    module: Option<String>,
    entry_name: String,
    output: String,
    plan_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P21ObservableOutputReceipt {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn output_bytes(&self) -> usize {
        self.output.len()
    }

    pub fn plan_sha256(&self) -> &[u8; 32] {
        &self.plan_sha256
    }

    pub fn plan_sha256_hex(&self) -> String {
        hex_bytes(&self.plan_sha256)
    }

    pub fn receipt_sha256(&self) -> &[u8; 32] {
        &self.receipt_sha256
    }

    pub fn receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.receipt_sha256)
    }

    pub fn canonical_receipt_bytes(&self) -> &[u8] {
        &self.canonical_receipt
    }

    pub fn render_text(&self) -> String {
        format!(
            "console-run module=\"{}\" entry=\"{}\" effect={} capability={} output-bytes={} plan-sha256={} receipt-sha256={} status=EMITTED authority=EXPLICIT grant=ConsoleWrite ambient-authority=NONE\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            self.output_bytes(),
            self.plan_sha256_hex(),
            self.receipt_sha256_hex(),
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"status\":\"emitted\",\"module\":\"{}\",\"entry\":\"{}\",\"effect\":\"{}\",\"capability\":\"{}\",\"output\":\"{}\",\"outputBytes\":{},\"planSha256\":\"{}\",\"receiptSha256\":\"{}\",\"authority\":\"EXPLICIT\",\"ambientAuthority\":\"NONE\"}}",
            P21_OUTPUT_SCHEMA,
            escape_json(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_json(&self.entry_name),
            P21_CONSOLE_EFFECT_NAME,
            P21_CONSOLE_EFFECT_NAME,
            escape_json(&self.output),
            self.output_bytes(),
            self.plan_sha256_hex(),
            self.receipt_sha256_hex(),
        )
    }
}

pub fn compile_observable_output_plan_p21(
    source: &SourceText,
) -> ObservableIoResult<P21ObservableOutputPlan> {
    let unit = analyze_type_effect_unit(source)?;
    validate_effect_prelude(source, &unit)?;

    let module = unit
        .module_unit()
        .module()
        .map(|declaration| source.slice(declaration.path().span()).map(str::to_owned))
        .transpose()?;

    let tokens = lex(source)?;
    let significant: Vec<&Token> = tokens
        .iter()
        .filter(|token| {
            !token.is_trivia()
                && token.kind() != &TokenKind::Eof
                && token.span().start().get() >= unit.body_span().start().get()
        })
        .collect();

    if significant.len() != 5 {
        let span = significant
            .first()
            .map(|token| token.span())
            .unwrap_or(unit.body_span());
        return Err(ObservableIoError::Syntax {
            message: "body must be exactly: entry <name> emits \"<text>\";".to_owned(),
            span,
        });
    }

    expect_identifier(
        source,
        significant[0],
        "entry",
        "expected contextual 'entry'",
    )?;
    let entry_name = expect_any_identifier(source, significant[1], "expected an entry name")?;
    if entry_name.len() > MAX_P21_ENTRY_NAME_BYTES {
        return Err(ObservableIoError::Policy {
            message: format!(
                "entry name has {} bytes, exceeding bound {MAX_P21_ENTRY_NAME_BYTES}",
                entry_name.len()
            ),
            span: Some(significant[1].span()),
        });
    }
    expect_identifier(
        source,
        significant[2],
        "emits",
        "expected contextual 'emits' after the entry name",
    )?;
    if significant[3].kind() != &TokenKind::QuotedText {
        return Err(ObservableIoError::Syntax {
            message: "expected one quoted UTF-8 output literal after contextual 'emits'".to_owned(),
            span: significant[3].span(),
        });
    }
    if significant[4].kind() != &TokenKind::Punctuation(';') {
        return Err(ObservableIoError::Syntax {
            message: "observable output entry must end with ';'".to_owned(),
            span: significant[4].span(),
        });
    }

    let raw_output = source.slice(significant[3].span())?;
    let output = decode_quoted_output(raw_output, significant[3].span())?;
    if output.len() > MAX_P21_OUTPUT_BYTES {
        return Err(ObservableIoError::Policy {
            message: format!(
                "decoded console output has {} bytes, exceeding bound {MAX_P21_OUTPUT_BYTES}",
                output.len()
            ),
            span: Some(significant[3].span()),
        });
    }

    let canonical_plan = canonical_plan_bytes(module.as_deref(), &entry_name, &output)?;
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P21ObservableOutputPlan {
        module,
        entry_name,
        output,
        canonical_plan,
        plan_sha256,
    })
}

pub fn execute_observable_output_p21(
    plan: &P21ObservableOutputPlan,
    authority: &CapabilitySet,
) -> ObservableIoResult<P21ObservableOutputReceipt> {
    let guard = P21ObservableEffectGuard::new(true, authority.clone());
    guard.check(P21ObservableEffect::ConsoleWrite)?;

    if plan.output.len() > MAX_P21_OUTPUT_BYTES {
        return Err(ObservableIoError::Invariant {
            message: "plan output exceeded the certified P2.1 bound after validation".to_owned(),
        });
    }

    let canonical_receipt = canonical_receipt_bytes(plan)?;
    let receipt_sha256 = sha256(&canonical_receipt);

    Ok(P21ObservableOutputReceipt {
        module: plan.module.clone(),
        entry_name: plan.entry_name.clone(),
        output: plan.output.clone(),
        plan_sha256: plan.plan_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

pub fn execute_observable_output_source_p21(
    source: &SourceText,
    authority: &CapabilitySet,
) -> ObservableIoResult<P21ObservableOutputReceipt> {
    let plan = compile_observable_output_plan_p21(source)?;
    execute_observable_output_p21(&plan, authority)
}

fn validate_effect_prelude(
    source: &SourceText,
    unit: &crate::frontend::TypeEffectUnit,
) -> ObservableIoResult<()> {
    let mut console_write = None;
    for declaration in unit.declarations() {
        if declaration.kind() != SurfaceDeclarationKind::Effect {
            continue;
        }
        let name = declaration.name().text(source)?;
        if name != P21_CONSOLE_EFFECT_NAME {
            return Err(ObservableIoError::Policy {
                message: format!(
                    "P2.1 permits only effect {P21_CONSOLE_EFFECT_NAME}; found '{name}'"
                ),
                span: Some(declaration.span()),
            });
        }
        if console_write.is_some() {
            return Err(ObservableIoError::Policy {
                message: format!("duplicate effect {P21_CONSOLE_EFFECT_NAME}"),
                span: Some(declaration.span()),
            });
        }
        console_write = Some(declaration.span());
    }

    if console_write.is_none() {
        return Err(ObservableIoError::Policy {
            message: format!(
                "observable console output requires explicit 'effect {P21_CONSOLE_EFFECT_NAME};'"
            ),
            span: Some(unit.body_span()),
        });
    }
    Ok(())
}

fn expect_identifier(
    source: &SourceText,
    token: &Token,
    expected: &str,
    message: &str,
) -> ObservableIoResult<()> {
    if token.kind() != &TokenKind::Identifier || source.slice(token.span())? != expected {
        return Err(ObservableIoError::Syntax {
            message: message.to_owned(),
            span: token.span(),
        });
    }
    Ok(())
}

fn expect_any_identifier(
    source: &SourceText,
    token: &Token,
    message: &str,
) -> ObservableIoResult<String> {
    if token.kind() != &TokenKind::Identifier {
        return Err(ObservableIoError::Syntax {
            message: message.to_owned(),
            span: token.span(),
        });
    }
    Ok(source.slice(token.span())?.to_owned())
}

fn decode_quoted_output(raw: &str, span: SourceSpan) -> ObservableIoResult<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(ObservableIoError::Syntax {
            message: "invalid quoted output token".to_owned(),
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
        let escaped = chars.next().ok_or_else(|| ObservableIoError::Syntax {
            message: "trailing escape in quoted output".to_owned(),
            span,
        })?;
        match escaped {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            _ => {
                return Err(ObservableIoError::Syntax {
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

fn canonical_plan_bytes(
    module: Option<&str>,
    entry_name: &str,
    output: &str,
) -> ObservableIoResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P21_PLAN_DOMAIN);
    push_string(module.unwrap_or("<anonymous>"), &mut bytes)?;
    push_string(entry_name, &mut bytes)?;
    push_string(P21_CONSOLE_EFFECT_NAME, &mut bytes)?;
    push_string(P21_CONSOLE_EFFECT_NAME, &mut bytes)?;
    push_string(output, &mut bytes)?;
    bytes.extend_from_slice(&(MAX_P21_OUTPUT_BYTES as u32).to_be_bytes());
    Ok(bytes)
}

fn canonical_receipt_bytes(plan: &P21ObservableOutputPlan) -> ObservableIoResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P21_RECEIPT_DOMAIN);
    bytes.extend_from_slice(&plan.plan_sha256);
    push_string(plan.module.as_deref().unwrap_or("<anonymous>"), &mut bytes)?;
    push_string(&plan.entry_name, &mut bytes)?;
    push_string(P21_CONSOLE_EFFECT_NAME, &mut bytes)?;
    push_string("EXPLICIT", &mut bytes)?;
    push_string("NONE", &mut bytes)?;
    push_string(&plan.output, &mut bytes)?;
    Ok(bytes)
}

fn push_string(value: &str, out: &mut Vec<u8>) -> ObservableIoResult<()> {
    let length = u32::try_from(value.len()).map_err(|_| ObservableIoError::Invariant {
        message: "canonical string length does not fit u32".to_owned(),
    })?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn escape_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output
}

fn escape_json(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", ch as u32);
            }
            other => output.push(other),
        }
    }
    output
}
