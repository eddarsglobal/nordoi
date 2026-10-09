use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::capability::{Capability, CapabilitySet};
use crate::effect_audit::hash::sha256;
use crate::frontend::{
    analyze_type_effect_unit, lex, LexError, SourceError, SourceSpan, SourceText,
    SurfaceDeclarationKind, Token, TokenKind, TypeEffectError,
};

const P25_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.5-CAPABILITY-SECURED-FILE-OUTPUT-PLAN\0";
const P25_RECEIPT_DOMAIN: &[u8] = b"NORDOI-P2.5-CAPABILITY-SECURED-FILE-OUTPUT-RECEIPT\0";

pub const P25_FILE_EFFECT_NAME: &str = "FileWrite";
pub const P25_OUTPUT_SCHEMA: &str = "nordoi.file-output.p2.5";
pub const MAX_P25_OUTPUT_BYTES: usize = 4096;
pub const MAX_P25_FILE_NAME_BYTES: usize = 128;
pub const MAX_P25_ENTRY_NAME_BYTES: usize = 128;

#[derive(Debug)]
pub enum FileOutputError {
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
    Materialization {
        message: String,
    },
    Invariant {
        message: String,
    },
}

impl FileOutputError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) => None,
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
}

impl Display for FileOutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.5 file output syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.5 file output policy error: {message}")
            }
            Self::CapabilityDenied(capability) => {
                write!(
                    f,
                    "P2.5 file output authority error: capability denied: {capability:?}"
                )
            }
            Self::Materialization { message } => {
                write!(f, "P2.5 file output materialization error: {message}")
            }
            Self::Invariant { message } => {
                write!(f, "P2.5 file output invariant failed: {message}")
            }
        }
    }
}

impl Error for FileOutputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Syntax { .. }
            | Self::Policy { .. }
            | Self::CapabilityDenied(_)
            | Self::Materialization { .. }
            | Self::Invariant { .. } => None,
        }
    }
}

impl From<TypeEffectError> for FileOutputError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<LexError> for FileOutputError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for FileOutputError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type FileOutputResult<T> = Result<T, FileOutputError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P25FileOutputPlan {
    module: Option<String>,
    entry_name: String,
    file_name: String,
    output: String,
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P25FileOutputPlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
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
pub struct P25AuthorizedFileOutput {
    plan: P25FileOutputPlan,
}

impl P25AuthorizedFileOutput {
    pub fn plan(&self) -> &P25FileOutputPlan {
        &self.plan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P25FileOutputReceipt {
    module: Option<String>,
    entry_name: String,
    file_name: String,
    output: String,
    plan_sha256: [u8; 32],
    content_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P25FileOutputReceipt {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
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

    pub fn content_sha256_hex(&self) -> String {
        hex_bytes(&self.content_sha256)
    }

    pub fn receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.receipt_sha256)
    }

    pub fn canonical_receipt_bytes(&self) -> &[u8] {
        &self.canonical_receipt
    }

    pub fn render_text(&self) -> String {
        format!(
            "file-write module=\"{}\" entry=\"{}\" target=\"{}\" effect={} capability=FileWrite(\"{}\") output-bytes={} plan-sha256={} content-sha256={} receipt-sha256={} status=WRITTEN authority=EXPLICIT grant=FileWrite(\"{}\") ambient-authority=NONE overwrite=DENIED\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            escape_text(&self.file_name),
            P25_FILE_EFFECT_NAME,
            escape_text(&self.file_name),
            self.output_bytes(),
            self.plan_sha256_hex(),
            self.content_sha256_hex(),
            self.receipt_sha256_hex(),
            escape_text(&self.file_name),
        )
    }
}

pub fn compile_file_output_plan_p25(source: &SourceText) -> FileOutputResult<P25FileOutputPlan> {
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

    if significant.len() != 7 {
        let span = significant
            .first()
            .map(|token| token.span())
            .unwrap_or(unit.body_span());
        return Err(FileOutputError::Syntax {
            message: "body must be exactly: entry <name> writes \"<file>\" emits \"<text>\";"
                .to_owned(),
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
    if entry_name.len() > MAX_P25_ENTRY_NAME_BYTES {
        return Err(FileOutputError::Policy {
            message: format!(
                "entry name has {} bytes, exceeding bound {MAX_P25_ENTRY_NAME_BYTES}",
                entry_name.len()
            ),
            span: Some(significant[1].span()),
        });
    }

    expect_identifier(
        source,
        significant[2],
        "writes",
        "expected contextual 'writes' after the entry name",
    )?;
    if significant[3].kind() != &TokenKind::QuotedText {
        return Err(FileOutputError::Syntax {
            message: "expected one quoted output file name after contextual 'writes'".to_owned(),
            span: significant[3].span(),
        });
    }
    expect_identifier(
        source,
        significant[4],
        "emits",
        "expected contextual 'emits' after the output file name",
    )?;
    if significant[5].kind() != &TokenKind::QuotedText {
        return Err(FileOutputError::Syntax {
            message: "expected one quoted UTF-8 output literal after contextual 'emits'".to_owned(),
            span: significant[5].span(),
        });
    }
    if significant[6].kind() != &TokenKind::Punctuation(';') {
        return Err(FileOutputError::Syntax {
            message: "file output entry must end with ';'".to_owned(),
            span: significant[6].span(),
        });
    }

    let file_name =
        decode_quoted_text(source.slice(significant[3].span())?, significant[3].span())?;
    validate_file_name(&file_name, significant[3].span())?;

    let output = decode_quoted_text(source.slice(significant[5].span())?, significant[5].span())?;
    if output.len() > MAX_P25_OUTPUT_BYTES {
        return Err(FileOutputError::Policy {
            message: format!(
                "decoded file output has {} bytes, exceeding bound {MAX_P25_OUTPUT_BYTES}",
                output.len()
            ),
            span: Some(significant[5].span()),
        });
    }

    let canonical_plan = canonical_plan_bytes(module.as_deref(), &entry_name, &file_name, &output)?;
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P25FileOutputPlan {
        module,
        entry_name,
        file_name,
        output,
        canonical_plan,
        plan_sha256,
    })
}

pub fn authorize_file_output_p25(
    plan: &P25FileOutputPlan,
    authority: &CapabilitySet,
) -> FileOutputResult<P25AuthorizedFileOutput> {
    let required = Capability::FileWrite(plan.file_name.clone());
    if !authority.contains(&required) {
        return Err(FileOutputError::CapabilityDenied(required));
    }
    if plan.output.len() > MAX_P25_OUTPUT_BYTES {
        return Err(FileOutputError::Invariant {
            message: "plan output exceeded the certified P2.5 bound after validation".to_owned(),
        });
    }
    Ok(P25AuthorizedFileOutput { plan: plan.clone() })
}

pub fn materialize_file_output_p25(
    authorized: &P25AuthorizedFileOutput,
    output_dir: &Path,
) -> FileOutputResult<P25FileOutputReceipt> {
    let plan = authorized.plan();
    let canonical_root =
        fs::canonicalize(output_dir).map_err(|error| FileOutputError::Materialization {
            message: format!("cannot resolve granted output directory: {error}"),
        })?;
    if !canonical_root.is_dir() {
        return Err(FileOutputError::Materialization {
            message: "granted output path is not a directory".to_owned(),
        });
    }

    let target = canonical_root.join(&plan.file_name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|error| FileOutputError::Materialization {
            message: format!("cannot create new target '{}': {error}", plan.file_name),
        })?;

    if let Err(error) = file
        .write_all(plan.output.as_bytes())
        .and_then(|_| file.flush())
    {
        drop(file);
        let _ = fs::remove_file(&target);
        return Err(FileOutputError::Materialization {
            message: format!("cannot write target '{}': {error}", plan.file_name),
        });
    }

    build_receipt(plan)
}

pub fn execute_file_output_p25(
    plan: &P25FileOutputPlan,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> FileOutputResult<P25FileOutputReceipt> {
    let authorized = authorize_file_output_p25(plan, authority)?;
    materialize_file_output_p25(&authorized, output_dir)
}

pub fn execute_file_output_source_p25(
    source: &SourceText,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> FileOutputResult<P25FileOutputReceipt> {
    let plan = compile_file_output_plan_p25(source)?;
    execute_file_output_p25(&plan, authority, output_dir)
}

fn build_receipt(plan: &P25FileOutputPlan) -> FileOutputResult<P25FileOutputReceipt> {
    let content_sha256 = sha256(plan.output.as_bytes());
    let canonical_receipt = canonical_receipt_bytes(plan, &content_sha256)?;
    let receipt_sha256 = sha256(&canonical_receipt);
    Ok(P25FileOutputReceipt {
        module: plan.module.clone(),
        entry_name: plan.entry_name.clone(),
        file_name: plan.file_name.clone(),
        output: plan.output.clone(),
        plan_sha256: plan.plan_sha256,
        content_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

fn validate_effect_prelude(
    source: &SourceText,
    unit: &crate::frontend::TypeEffectUnit,
) -> FileOutputResult<()> {
    let mut file_write = None;
    for declaration in unit.declarations() {
        if declaration.kind() != SurfaceDeclarationKind::Effect {
            continue;
        }
        let name = declaration.name().text(source)?;
        if name != P25_FILE_EFFECT_NAME {
            return Err(FileOutputError::Policy {
                message: format!("P2.5 permits only effect {P25_FILE_EFFECT_NAME}; found '{name}'"),
                span: Some(declaration.span()),
            });
        }
        if file_write.is_some() {
            return Err(FileOutputError::Policy {
                message: format!("duplicate effect {P25_FILE_EFFECT_NAME}"),
                span: Some(declaration.span()),
            });
        }
        file_write = Some(declaration.span());
    }

    if file_write.is_none() {
        return Err(FileOutputError::Policy {
            message: format!("file output requires explicit 'effect {P25_FILE_EFFECT_NAME};'"),
            span: Some(unit.body_span()),
        });
    }
    Ok(())
}

fn validate_file_name(file_name: &str, span: SourceSpan) -> FileOutputResult<()> {
    if file_name.is_empty() {
        return Err(FileOutputError::Policy {
            message: "output file name must not be empty".to_owned(),
            span: Some(span),
        });
    }
    if file_name.len() > MAX_P25_FILE_NAME_BYTES {
        return Err(FileOutputError::Policy {
            message: format!(
                "output file name has {} bytes, exceeding bound {MAX_P25_FILE_NAME_BYTES}",
                file_name.len()
            ),
            span: Some(span),
        });
    }
    if file_name == "." || file_name == ".." {
        return Err(FileOutputError::Policy {
            message: "output file name must be one ordinary relative file name".to_owned(),
            span: Some(span),
        });
    }
    if file_name.contains('/') || file_name.contains('\\') {
        return Err(FileOutputError::Policy {
            message: "output file name must not contain path separators".to_owned(),
            span: Some(span),
        });
    }
    if file_name.chars().any(char::is_control) {
        return Err(FileOutputError::Policy {
            message: "output file name must not contain control characters".to_owned(),
            span: Some(span),
        });
    }
    if Path::new(file_name).is_absolute() {
        return Err(FileOutputError::Policy {
            message: "absolute output paths are forbidden".to_owned(),
            span: Some(span),
        });
    }
    Ok(())
}

fn expect_identifier(
    source: &SourceText,
    token: &Token,
    expected: &str,
    message: &str,
) -> FileOutputResult<()> {
    if token.kind() != &TokenKind::Identifier || source.slice(token.span())? != expected {
        return Err(FileOutputError::Syntax {
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
) -> FileOutputResult<String> {
    if token.kind() != &TokenKind::Identifier {
        return Err(FileOutputError::Syntax {
            message: message.to_owned(),
            span: token.span(),
        });
    }
    Ok(source.slice(token.span())?.to_owned())
}

fn decode_quoted_text(raw: &str, span: SourceSpan) -> FileOutputResult<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(FileOutputError::Syntax {
            message: "invalid quoted text token".to_owned(),
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
        let escaped = chars.next().ok_or_else(|| FileOutputError::Syntax {
            message: "trailing escape in quoted text".to_owned(),
            span,
        })?;
        match escaped {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            _ => {
                return Err(FileOutputError::Syntax {
                    message: format!(
                        "unsupported text escape '\\{escaped}'; allowed escapes are \\\\, \\\" , \\n, \\r and \\t"
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
    file_name: &str,
    output: &str,
) -> FileOutputResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P25_PLAN_DOMAIN);
    push_string(module.unwrap_or("<anonymous>"), &mut bytes)?;
    push_string(entry_name, &mut bytes)?;
    push_string(P25_FILE_EFFECT_NAME, &mut bytes)?;
    push_string(file_name, &mut bytes)?;
    push_string(output, &mut bytes)?;
    bytes.extend_from_slice(&(MAX_P25_FILE_NAME_BYTES as u32).to_be_bytes());
    bytes.extend_from_slice(&(MAX_P25_OUTPUT_BYTES as u32).to_be_bytes());
    Ok(bytes)
}

fn canonical_receipt_bytes(
    plan: &P25FileOutputPlan,
    content_sha256: &[u8; 32],
) -> FileOutputResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(P25_RECEIPT_DOMAIN);
    bytes.extend_from_slice(&plan.plan_sha256);
    push_string(plan.module.as_deref().unwrap_or("<anonymous>"), &mut bytes)?;
    push_string(&plan.entry_name, &mut bytes)?;
    push_string(P25_FILE_EFFECT_NAME, &mut bytes)?;
    push_string(&plan.file_name, &mut bytes)?;
    bytes.extend_from_slice(content_sha256);
    push_string("EXPLICIT", &mut bytes)?;
    push_string("NONE", &mut bytes)?;
    push_string("DENIED", &mut bytes)?;
    Ok(bytes)
}

fn push_string(value: &str, out: &mut Vec<u8>) -> FileOutputResult<()> {
    let length = u32::try_from(value.len()).map_err(|_| FileOutputError::Invariant {
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
