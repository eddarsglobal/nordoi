use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::capability::{Capability, CapabilitySet};
use crate::dynamic_input_v07::{
    lower_dynamic_plan_v07, DynamicInputError, DynamicValue, DynamicValueKind,
};
use crate::effect_audit::hash::sha256;
use crate::frontend::{lex, LexError, SourceError, SourceSpan, SourceText, Token, TokenKind};
use crate::input::{
    InputBatch, InputDeviceId, InputError, InputEvent, InputPayload, InputSequence, InputSource,
    InputTarget,
};
use crate::observable_io_p25::P25_FILE_EFFECT_NAME;
use crate::observable_io_p26::{
    compile_dynamic_file_output_plan_p26, DynamicFileOutputError, P26DynamicFileOutputPlan,
    MAX_P26_OUTPUT_BYTES,
};
use crate::runtime::{run_closed_observed, RuntimeError};
use crate::value::Value;

const P27_PLAN_DOMAIN: &[u8] = b"NORDOI-P2.7-ATOMIC-MULTI-FILE-BUNDLE-PLAN\0";
const P27_FILE_PROOF_DOMAIN: &[u8] = b"NORDOI-P2.7-ATOMIC-MULTI-FILE-BUNDLE-FILE-PROOF\0";
const P27_SEGMENT_PROOF_DOMAIN: &[u8] = b"NORDOI-P2.7-ATOMIC-MULTI-FILE-BUNDLE-SEGMENT-PROOF\0";
const P27_RECEIPT_DOMAIN: &[u8] = b"NORDOI-P2.7-ATOMIC-MULTI-FILE-BUNDLE-RECEIPT\0";

pub const P27_OUTPUT_SCHEMA: &str = "nordoi.atomic-multi-file-bundle.p2.7";
pub const MIN_P27_FILES: usize = 2;
pub const MAX_P27_FILES: usize = 8;
pub const MAX_P27_BUNDLE_NAME_BYTES: usize = 128;
pub const MAX_P27_TOTAL_OUTPUT_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub enum AtomicBundleOutputError {
    Lex(LexError),
    Source(SourceError),
    Input(InputError),
    Dynamic(DynamicInputError),
    Runtime(RuntimeError),
    DynamicFile(DynamicFileOutputError),
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

impl AtomicBundleOutputError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Lex(error) => error.span(),
            Self::Source(_) | Self::Input(_) | Self::Dynamic(_) | Self::Runtime(_) => None,
            Self::DynamicFile(error) => error.primary_span(),
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
            Self::Lex(_) | Self::Source(_) | Self::Syntax { .. } | Self::Policy { .. }
        ) || matches!(self, Self::DynamicFile(error) if error.is_frontend_failure())
    }

    pub fn is_authority_failure(&self) -> bool {
        matches!(self, Self::CapabilityDenied(_))
    }

    pub fn is_materialization_failure(&self) -> bool {
        matches!(self, Self::Materialization { .. })
    }
}

impl Display for AtomicBundleOutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Input(error) => Display::fmt(error, f),
            Self::Dynamic(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::DynamicFile(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => {
                write!(f, "P2.7 atomic bundle output syntax error: {message}")
            }
            Self::Policy { message, .. } => {
                write!(f, "P2.7 atomic bundle output policy error: {message}")
            }
            Self::CapabilityDenied(capability) => write!(
                f,
                "P2.7 atomic bundle output authority error: capability denied: {capability:?}"
            ),
            Self::Materialization { message } => {
                write!(
                    f,
                    "P2.7 atomic bundle output materialization error: {message}"
                )
            }
            Self::Invariant { message } => {
                write!(f, "P2.7 atomic bundle output invariant failed: {message}")
            }
        }
    }
}

impl Error for AtomicBundleOutputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Input(error) => Some(error),
            Self::Dynamic(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::DynamicFile(error) => Some(error),
            Self::Syntax { .. }
            | Self::Policy { .. }
            | Self::CapabilityDenied(_)
            | Self::Materialization { .. }
            | Self::Invariant { .. } => None,
        }
    }
}

impl From<LexError> for AtomicBundleOutputError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for AtomicBundleOutputError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<InputError> for AtomicBundleOutputError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

impl From<DynamicInputError> for AtomicBundleOutputError {
    fn from(value: DynamicInputError) -> Self {
        Self::Dynamic(value)
    }
}

impl From<RuntimeError> for AtomicBundleOutputError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<DynamicFileOutputError> for AtomicBundleOutputError {
    fn from(value: DynamicFileOutputError) -> Self {
        Self::DynamicFile(value)
    }
}

pub type AtomicBundleOutputResult<T> = Result<T, AtomicBundleOutputError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P27BundleFilePlan {
    file_name: String,
    dynamic_plan: P26DynamicFileOutputPlan,
    maximum_output_bytes: usize,
}

impl P27BundleFilePlan {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn dynamic_plan(&self) -> &P26DynamicFileOutputPlan {
        &self.dynamic_plan
    }

    pub fn maximum_output_bytes(&self) -> usize {
        self.maximum_output_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P27AtomicBundlePlan {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    bundle_name: String,
    files: Vec<P27BundleFilePlan>,
    maximum_total_output_bytes: usize,
    canonical_plan: Vec<u8>,
    plan_sha256: [u8; 32],
}

impl P27AtomicBundlePlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn input_name(&self) -> &str {
        &self.input_name
    }

    pub fn bundle_name(&self) -> &str {
        &self.bundle_name
    }

    pub fn files(&self) -> &[P27BundleFilePlan] {
        &self.files
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn maximum_total_output_bytes(&self) -> usize {
        self.maximum_total_output_bytes
    }

    pub fn required_capabilities(&self) -> Vec<Capability> {
        self.files
            .iter()
            .map(|file| Capability::FileWrite(self.capability_target(file.file_name())))
            .collect()
    }

    pub fn canonical_plan_bytes(&self) -> &[u8] {
        &self.canonical_plan
    }

    pub fn plan_sha256_hex(&self) -> String {
        hex_bytes(&self.plan_sha256)
    }

    fn capability_target(&self, file_name: &str) -> String {
        format!("{}/{}", self.bundle_name, file_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P27PublishedFileReceipt {
    file_name: String,
    results: Vec<DynamicValue>,
    output: String,
    proof_sha256: [u8; 32],
}

impl P27PublishedFileReceipt {
    pub fn file_name(&self) -> &str {
        &self.file_name
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

    pub fn proof_sha256_hex(&self) -> String {
        hex_bytes(&self.proof_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P27AtomicBundleReceipt {
    module: Option<String>,
    entry_name: String,
    input_name: String,
    bundle_name: String,
    key_code: u32,
    files: Vec<P27PublishedFileReceipt>,
    total_output_bytes: usize,
    plan_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    canonical_receipt: Vec<u8>,
}

impl P27AtomicBundleReceipt {
    pub fn bundle_name(&self) -> &str {
        &self.bundle_name
    }

    pub fn files(&self) -> &[P27PublishedFileReceipt] {
        &self.files
    }

    pub fn total_output_bytes(&self) -> usize {
        self.total_output_bytes
    }

    pub fn receipt_sha256_hex(&self) -> String {
        hex_bytes(&self.receipt_sha256)
    }

    pub fn canonical_receipt_bytes(&self) -> &[u8] {
        &self.canonical_receipt
    }

    pub fn render_text(&self) -> String {
        let targets = self
            .files
            .iter()
            .map(|file| format!("\"{}\"", escape_text(file.file_name())))
            .collect::<Vec<_>>()
            .join(",");
        let capabilities = self
            .files
            .iter()
            .map(|file| {
                format!(
                    "FileWrite(\"{}/{}\")",
                    escape_text(&self.bundle_name),
                    escape_text(file.file_name())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let results = self
            .files
            .iter()
            .map(|file| {
                let values = file
                    .results()
                    .iter()
                    .map(|value| value.as_text())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("\"{}\":[{}]", escape_text(file.file_name()), values)
            })
            .collect::<Vec<_>>()
            .join(",");
        let proofs = self
            .files
            .iter()
            .map(P27PublishedFileReceipt::proof_sha256_hex)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "bundle-write module=\"{}\" entry=\"{}\" input=\"{}\" key-code={} bundle=\"{}\" files={} targets=[{}] results=[{}] effect={} capabilities=[{}] output-bytes={} plan-sha256={} file-proofs-sha256=[{}] receipt-sha256={} status=COMMITTED authority=EXPLICIT grant-count={} ambient-authority=NONE overwrite=DENIED partial-final-state=FORBIDDEN publication=DIRECTORY-RENAME crash-durability=UNCLAIMED\n",
            escape_text(self.module.as_deref().unwrap_or("<anonymous>")),
            escape_text(&self.entry_name),
            escape_text(&self.input_name),
            self.key_code,
            escape_text(&self.bundle_name),
            self.files.len(),
            targets,
            results,
            P25_FILE_EFFECT_NAME,
            capabilities,
            self.total_output_bytes,
            hex_bytes(&self.plan_sha256),
            proofs,
            self.receipt_sha256_hex(),
            self.files.len(),
        )
    }
}

pub fn compile_atomic_bundle_plan_p27(
    source: &SourceText,
) -> AtomicBundleOutputResult<P27AtomicBundlePlan> {
    let tokens = lex(source)?;
    let significant = tokens
        .iter()
        .filter(|token| !token.is_trivia() && token.kind() != &TokenKind::Eof)
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
        return Err(AtomicBundleOutputError::Policy {
            message: "P2.7 requires exactly one entry declaration".to_owned(),
            span: entry_indexes
                .first()
                .and_then(|index| significant.get(*index))
                .map(|token| token.span()),
        });
    }

    let entry_index = entry_indexes[0];
    let entry_token = significant[entry_index];
    let entry_name_token =
        significant
            .get(entry_index + 1)
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
                message: "expected an entry name".to_owned(),
                span: entry_token.span(),
            })?;
    if entry_name_token.kind() != &TokenKind::Identifier {
        return Err(AtomicBundleOutputError::Syntax {
            message: "expected an entry name".to_owned(),
            span: entry_name_token.span(),
        });
    }
    let entry_name = source.slice(entry_name_token.span())?.to_owned();

    let writes =
        significant
            .get(entry_index + 2)
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
                message: "expected contextual 'writes' after the entry name".to_owned(),
                span: entry_name_token.span(),
            })?;
    expect_identifier(
        source,
        writes,
        "writes",
        "expected contextual 'writes' after the entry name",
    )?;

    let bundle_token =
        significant
            .get(entry_index + 3)
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
                message: "expected one quoted bundle name after contextual 'writes'".to_owned(),
                span: writes.span(),
            })?;
    if bundle_token.kind() != &TokenKind::QuotedText {
        return Err(AtomicBundleOutputError::Syntax {
            message: "expected one quoted bundle name after contextual 'writes'".to_owned(),
            span: bundle_token.span(),
        });
    }
    let bundle_name = decode_quoted_text(source.slice(bundle_token.span())?, bundle_token.span())?;
    validate_simple_name(
        &bundle_name,
        MAX_P27_BUNDLE_NAME_BYTES,
        "bundle name",
        bundle_token.span(),
    )?;

    let open = significant
        .get(entry_index + 4)
        .ok_or_else(|| AtomicBundleOutputError::Syntax {
            message: "expected '{' after the bundle name".to_owned(),
            span: bundle_token.span(),
        })?;
    if open.kind() != &TokenKind::Punctuation('{') {
        return Err(AtomicBundleOutputError::Syntax {
            message: "expected '{' after the bundle name".to_owned(),
            span: open.span(),
        });
    }

    let close_index = find_matching_bundle_close(&significant, entry_index + 4)?;
    let close = significant[close_index];
    let semicolon =
        significant
            .get(close_index + 1)
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
                message: "expected ';' after the bundle block".to_owned(),
                span: close.span(),
            })?;
    if semicolon.kind() != &TokenKind::Punctuation(';') {
        return Err(AtomicBundleOutputError::Syntax {
            message: "expected ';' after the bundle block".to_owned(),
            span: semicolon.span(),
        });
    }
    if close_index + 2 != significant.len() {
        return Err(AtomicBundleOutputError::Policy {
            message: "P2.7 permits no significant source after the atomic bundle entry".to_owned(),
            span: significant.get(close_index + 2).map(|token| token.span()),
        });
    }

    let prelude_end = entry_token.span().start().get() as usize;
    let prelude = &source.text()[..prelude_end];
    let mut files = parse_bundle_files(
        source,
        &significant[(entry_index + 5)..close_index],
        prelude,
        &entry_name,
    )?;

    if files.len() < MIN_P27_FILES {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("P2.7 requires at least {MIN_P27_FILES} files in one atomic bundle"),
            span: Some(open.span()),
        });
    }
    if files.len() > MAX_P27_FILES {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("P2.7 permits at most {MAX_P27_FILES} files in one atomic bundle"),
            span: Some(open.span()),
        });
    }

    files.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    for pair in files.windows(2) {
        if pair[0].file_name == pair[1].file_name {
            return Err(AtomicBundleOutputError::Policy {
                message: format!("duplicate bundle target '{}'", pair[0].file_name),
                span: Some(bundle_token.span()),
            });
        }
    }

    let first = files
        .first()
        .ok_or_else(|| AtomicBundleOutputError::Invariant {
            message: "P2.7 file count validation admitted an empty bundle".to_owned(),
        })?;
    let module = first.dynamic_plan.module().map(str::to_owned);
    let input_name = first.dynamic_plan.input_name().to_owned();
    for file in &files {
        if file.dynamic_plan.module() != module.as_deref()
            || file.dynamic_plan.entry_name() != entry_name.as_str()
            || file.dynamic_plan.input_name() != input_name.as_str()
        {
            return Err(AtomicBundleOutputError::Invariant {
                message: "bundle file plans disagree on module, entry or input identity".to_owned(),
            });
        }
    }

    let maximum_total_output_bytes = files
        .iter()
        .try_fold(0usize, |total, file| {
            total.checked_add(file.maximum_output_bytes)
        })
        .ok_or_else(|| AtomicBundleOutputError::Invariant {
            message: "bundle maximum output byte count overflowed usize".to_owned(),
        })?;
    if maximum_total_output_bytes > MAX_P27_TOTAL_OUTPUT_BYTES {
        return Err(AtomicBundleOutputError::Policy {
            message: format!(
                "atomic bundle can require {maximum_total_output_bytes} UTF-8 bytes, exceeding global bound {MAX_P27_TOTAL_OUTPUT_BYTES}"
            ),
            span: Some(open.span()),
        });
    }

    let canonical_plan = canonical_plan_bytes(
        module.as_deref(),
        &entry_name,
        &input_name,
        &bundle_name,
        &files,
        maximum_total_output_bytes,
    );
    let plan_sha256 = sha256(&canonical_plan);

    Ok(P27AtomicBundlePlan {
        module,
        entry_name,
        input_name,
        bundle_name,
        files,
        maximum_total_output_bytes,
        canonical_plan,
        plan_sha256,
    })
}

pub fn execute_atomic_bundle_p27(
    plan: &P27AtomicBundlePlan,
    key_code: u32,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> AtomicBundleOutputResult<P27AtomicBundleReceipt> {
    for required in plan.required_capabilities() {
        if !authority.contains(&required) {
            return Err(AtomicBundleOutputError::CapabilityDenied(required));
        }
    }

    let input = key_batch(key_code);
    let input_bytes = input.canonical_bytes()?;
    let mut prepared = Vec::with_capacity(plan.files.len());
    let mut total_output_bytes = 0usize;

    for file in &plan.files {
        let published = evaluate_bundle_file(file, key_code, &input, &input_bytes)?;
        total_output_bytes = total_output_bytes
            .checked_add(published.output_bytes())
            .ok_or_else(|| AtomicBundleOutputError::Invariant {
                message: "rendered bundle byte count overflowed usize".to_owned(),
            })?;
        prepared.push(published);
    }

    if total_output_bytes > MAX_P27_TOTAL_OUTPUT_BYTES {
        return Err(AtomicBundleOutputError::Invariant {
            message: format!(
                "rendered bundle has {total_output_bytes} UTF-8 bytes, exceeding bound {MAX_P27_TOTAL_OUTPUT_BYTES}"
            ),
        });
    }

    let canonical_root =
        fs::canonicalize(output_dir).map_err(|error| AtomicBundleOutputError::Materialization {
            message: format!("cannot resolve granted output directory: {error}"),
        })?;
    if !canonical_root.is_dir() {
        return Err(AtomicBundleOutputError::Materialization {
            message: "granted output path is not a directory".to_owned(),
        });
    }

    let final_bundle = canonical_root.join(&plan.bundle_name);
    if path_entry_exists(&final_bundle)? {
        return Err(AtomicBundleOutputError::Materialization {
            message: format!(
                "cannot create new atomic bundle '{}': target already exists",
                plan.bundle_name
            ),
        });
    }

    let staging = create_staging_dir(&canonical_root, plan)?;
    if let Err(error) = write_staging_files(&staging, &prepared) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    match path_entry_exists(&final_bundle) {
        Ok(false) => {}
        Ok(true) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(AtomicBundleOutputError::Materialization {
                message: format!(
                    "cannot commit atomic bundle '{}': target appeared before publication",
                    plan.bundle_name
                ),
            });
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    }

    if let Err(error) = fs::rename(&staging, &final_bundle) {
        let _ = fs::remove_dir_all(&staging);
        return Err(AtomicBundleOutputError::Materialization {
            message: format!(
                "cannot publish atomic bundle '{}' by same-root directory rename: {error}",
                plan.bundle_name
            ),
        });
    }

    build_receipt(plan, key_code, prepared, total_output_bytes)
}

pub fn execute_atomic_bundle_source_p27(
    source: &SourceText,
    key_code: u32,
    authority: &CapabilitySet,
    output_dir: &Path,
) -> AtomicBundleOutputResult<P27AtomicBundleReceipt> {
    let plan = compile_atomic_bundle_plan_p27(source)?;
    execute_atomic_bundle_p27(&plan, key_code, authority, output_dir)
}

fn parse_bundle_files(
    source: &SourceText,
    tokens: &[&Token],
    prelude: &str,
    entry_name: &str,
) -> AtomicBundleOutputResult<Vec<P27BundleFilePlan>> {
    let mut files = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let file_token = tokens[index];
        if file_token.kind() != &TokenKind::QuotedText {
            return Err(AtomicBundleOutputError::Syntax {
                message: "expected quoted file name inside the atomic bundle".to_owned(),
                span: file_token.span(),
            });
        }
        let emits = tokens
            .get(index + 1)
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
                message: "expected contextual 'emits' after bundle file name".to_owned(),
                span: file_token.span(),
            })?;
        expect_identifier(
            source,
            emits,
            "emits",
            "expected contextual 'emits' after bundle file name",
        )?;

        let expression_start = index + 2;
        if expression_start >= tokens.len() {
            return Err(AtomicBundleOutputError::Syntax {
                message: "expected structured dynamic output expression after 'emits'".to_owned(),
                span: emits.span(),
            });
        }
        let semicolon_index = find_item_semicolon(tokens, expression_start)?;
        if semicolon_index == expression_start {
            return Err(AtomicBundleOutputError::Syntax {
                message: "bundle file output expression must not be empty".to_owned(),
                span: tokens[semicolon_index].span(),
            });
        }

        let raw_file = source.slice(file_token.span())?;
        let expression_first = tokens[expression_start].span().start().get() as usize;
        let expression_last = tokens[semicolon_index - 1].span().end().get() as usize;
        let expression = &source.text()[expression_first..expression_last];
        let synthetic_text =
            format!("{prelude}entry {entry_name} writes {raw_file} emits {expression};");
        let synthetic = SourceText::new(source.id(), source.name(), synthetic_text)?;
        let dynamic_plan = compile_dynamic_file_output_plan_p26(&synthetic)?;
        let maximum_output_bytes = maximum_dynamic_output_bytes(&dynamic_plan);
        if maximum_output_bytes > MAX_P26_OUTPUT_BYTES {
            return Err(AtomicBundleOutputError::Invariant {
                message: format!(
                    "P2.6 admitted file '{}' above its certified output bound",
                    dynamic_plan.file_name()
                ),
            });
        }
        files.push(P27BundleFilePlan {
            file_name: dynamic_plan.file_name().to_owned(),
            dynamic_plan,
            maximum_output_bytes,
        });

        index = semicolon_index + 1;
    }

    Ok(files)
}

fn find_matching_bundle_close(
    tokens: &[&Token],
    open_index: usize,
) -> AtomicBundleOutputResult<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        match token.kind() {
            TokenKind::Punctuation('{') => depth += 1,
            TokenKind::Punctuation('}') => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| AtomicBundleOutputError::Syntax {
                        message: "unexpected '}' in bundle block".to_owned(),
                        span: token.span(),
                    })?;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err(AtomicBundleOutputError::Syntax {
        message: "unterminated atomic bundle block".to_owned(),
        span: tokens[open_index].span(),
    })
}

fn find_item_semicolon(tokens: &[&Token], start: usize) -> AtomicBundleOutputResult<usize> {
    let mut paren_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::Punctuation('(') => paren_depth += 1,
            TokenKind::Punctuation(')') => {
                paren_depth =
                    paren_depth
                        .checked_sub(1)
                        .ok_or_else(|| AtomicBundleOutputError::Syntax {
                            message: "unexpected ')' in bundle output expression".to_owned(),
                            span: token.span(),
                        })?;
            }
            TokenKind::Punctuation(';') if paren_depth == 0 => return Ok(index),
            TokenKind::Punctuation('{') | TokenKind::Punctuation('}') => {
                return Err(AtomicBundleOutputError::Policy {
                    message: "nested braces are not part of P2.7 bundle file expressions"
                        .to_owned(),
                    span: Some(token.span()),
                });
            }
            _ => {}
        }
    }
    Err(AtomicBundleOutputError::Syntax {
        message: "expected ';' after bundle file output expression".to_owned(),
        span: tokens[start].span(),
    })
}

fn maximum_dynamic_output_bytes(plan: &P26DynamicFileOutputPlan) -> usize {
    let static_bytes = plan
        .render_plan()
        .static_segments()
        .iter()
        .map(String::len)
        .sum::<usize>();
    let dynamic_bytes = plan
        .result_kinds()
        .into_iter()
        .map(maximum_rendered_bytes)
        .sum::<usize>();
    static_bytes.saturating_add(dynamic_bytes)
}

const fn maximum_rendered_bytes(kind: DynamicValueKind) -> usize {
    match kind {
        DynamicValueKind::Int => 20,
        DynamicValueKind::Bool => 5,
    }
}

fn evaluate_bundle_file(
    file: &P27BundleFilePlan,
    key_code: u32,
    input: &InputBatch,
    input_bytes: &[u8],
) -> AtomicBundleOutputResult<P27PublishedFileReceipt> {
    let mut results = Vec::with_capacity(file.dynamic_plan.render_plan().runtime_segment_count());
    let mut rendered = Vec::with_capacity(file.dynamic_plan.render_plan().runtime_segment_count());
    let mut segment_proofs =
        Vec::with_capacity(file.dynamic_plan.render_plan().runtime_segment_count());

    for (index, segment) in file
        .dynamic_plan
        .render_plan()
        .segment_plans()
        .iter()
        .enumerate()
    {
        let lowering = lower_dynamic_plan_v07(segment.dynamic_plan())?;
        let runtime = run_closed_observed(lowering.program(), input)?;
        let value = runtime
            .register(lowering.result_register())
            .ok_or_else(|| AtomicBundleOutputError::Invariant {
                message: format!(
                    "file '{}' segment {} result register is missing",
                    file.file_name,
                    index + 1
                ),
            })?;
        let result = match (segment.result_kind(), value) {
            (DynamicValueKind::Int, Value::Int(value)) => DynamicValue::Int(*value),
            (DynamicValueKind::Bool, Value::Bool(value)) => DynamicValue::Bool(*value),
            (expected, actual) => {
                return Err(AtomicBundleOutputError::Invariant {
                    message: format!(
                        "file '{}' segment {} result kind mismatch: expected {}, got {actual:?}",
                        file.file_name,
                        index + 1,
                        expected.as_str()
                    ),
                });
            }
        };
        let proof = segment_proof_bytes(
            &file.file_name,
            index,
            segment.canonical_plan_bytes(),
            lowering.canonical_v07_witness_bytes(),
            input_bytes,
            runtime.runtime().replay_key.value(),
            result,
        );
        segment_proofs.push(sha256(&proof));
        rendered.push(render_dynamic_value(result));
        results.push(result);
    }

    let mut output = String::new();
    for (static_segment, rendered_segment) in file
        .dynamic_plan
        .render_plan()
        .static_segments()
        .iter()
        .zip(rendered.iter())
    {
        output.push_str(static_segment);
        output.push_str(rendered_segment);
    }
    output.push_str(&file.dynamic_plan.render_plan().static_segments()[rendered.len()]);
    if output.len() > MAX_P26_OUTPUT_BYTES {
        return Err(AtomicBundleOutputError::Invariant {
            message: format!(
                "rendered file '{}' has {} bytes, exceeding P2.6 bound {MAX_P26_OUTPUT_BYTES}",
                file.file_name,
                output.len()
            ),
        });
    }

    let proof = file_proof_bytes(file, key_code, &results, &output, &segment_proofs);
    Ok(P27PublishedFileReceipt {
        file_name: file.file_name.clone(),
        results,
        output,
        proof_sha256: sha256(&proof),
    })
}

fn create_staging_dir(
    root: &Path,
    plan: &P27AtomicBundlePlan,
) -> AtomicBundleOutputResult<PathBuf> {
    let hash = plan.plan_sha256_hex();
    let prefix = &hash[..16];
    for attempt in 0..64u8 {
        let path = root.join(format!(".nordoi-p27-stage-{prefix}-{attempt:02x}"));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(AtomicBundleOutputError::Materialization {
                    message: format!("cannot create private P2.7 staging directory: {error}"),
                });
            }
        }
    }
    Err(AtomicBundleOutputError::Materialization {
        message: "cannot reserve a private P2.7 staging directory after 64 attempts".to_owned(),
    })
}

fn write_staging_files(
    staging: &Path,
    files: &[P27PublishedFileReceipt],
) -> AtomicBundleOutputResult<()> {
    for file in files {
        let target = staging.join(file.file_name());
        let mut handle = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|error| AtomicBundleOutputError::Materialization {
                message: format!(
                    "cannot create staged target '{}': {error}",
                    file.file_name()
                ),
            })?;
        handle
            .write_all(file.output().as_bytes())
            .and_then(|_| handle.flush())
            .map_err(|error| AtomicBundleOutputError::Materialization {
                message: format!("cannot write staged target '{}': {error}", file.file_name()),
            })?;
    }
    Ok(())
}

fn path_entry_exists(path: &Path) -> AtomicBundleOutputResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(AtomicBundleOutputError::Materialization {
            message: format!("cannot inspect output target '{}': {error}", path.display()),
        }),
    }
}

fn build_receipt(
    plan: &P27AtomicBundlePlan,
    key_code: u32,
    files: Vec<P27PublishedFileReceipt>,
    total_output_bytes: usize,
) -> AtomicBundleOutputResult<P27AtomicBundleReceipt> {
    let canonical_receipt = canonical_receipt_bytes(plan, key_code, &files, total_output_bytes);
    let receipt_sha256 = sha256(&canonical_receipt);
    Ok(P27AtomicBundleReceipt {
        module: plan.module.clone(),
        entry_name: plan.entry_name.clone(),
        input_name: plan.input_name.clone(),
        bundle_name: plan.bundle_name.clone(),
        key_code,
        files,
        total_output_bytes,
        plan_sha256: plan.plan_sha256,
        receipt_sha256,
        canonical_receipt,
    })
}

fn canonical_plan_bytes(
    module: Option<&str>,
    entry_name: &str,
    input_name: &str,
    bundle_name: &str,
    files: &[P27BundleFilePlan],
    maximum_total_output_bytes: usize,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P27_PLAN_DOMAIN);
    push_string(&mut out, module.unwrap_or("<anonymous>"));
    push_string(&mut out, entry_name);
    push_string(&mut out, input_name);
    push_string(&mut out, P25_FILE_EFFECT_NAME);
    push_string(&mut out, bundle_name);
    push_u64(&mut out, files.len() as u64);
    for file in files {
        push_string(&mut out, &file.file_name);
        push_component(&mut out, file.dynamic_plan.canonical_plan_bytes());
        push_u64(&mut out, file.maximum_output_bytes as u64);
    }
    push_u64(&mut out, maximum_total_output_bytes as u64);
    push_u64(&mut out, MIN_P27_FILES as u64);
    push_u64(&mut out, MAX_P27_FILES as u64);
    push_u64(&mut out, MAX_P27_TOTAL_OUTPUT_BYTES as u64);
    out
}

fn canonical_receipt_bytes(
    plan: &P27AtomicBundlePlan,
    key_code: u32,
    files: &[P27PublishedFileReceipt],
    total_output_bytes: usize,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P27_RECEIPT_DOMAIN);
    push_component(&mut out, plan.canonical_plan_bytes());
    out.extend_from_slice(&key_code.to_be_bytes());
    push_u64(&mut out, files.len() as u64);
    for file in files {
        push_string(&mut out, file.file_name());
        out.extend_from_slice(&file.proof_sha256);
    }
    push_u64(&mut out, total_output_bytes as u64);
    push_string(&mut out, "EXPLICIT");
    push_string(&mut out, "NONE");
    push_string(&mut out, "DENIED");
    push_string(&mut out, "DIRECTORY-RENAME");
    push_string(&mut out, "UNCLAIMED");
    out
}

fn file_proof_bytes(
    file: &P27BundleFilePlan,
    key_code: u32,
    results: &[DynamicValue],
    output: &str,
    segment_proofs: &[[u8; 32]],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P27_FILE_PROOF_DOMAIN);
    push_string(&mut out, &file.file_name);
    push_component(&mut out, file.dynamic_plan.canonical_plan_bytes());
    out.extend_from_slice(&key_code.to_be_bytes());
    push_u64(&mut out, results.len() as u64);
    for result in results {
        encode_dynamic_value(*result, &mut out);
    }
    push_string(&mut out, output);
    out.extend_from_slice(&sha256(output.as_bytes()));
    push_u64(&mut out, segment_proofs.len() as u64);
    for proof in segment_proofs {
        out.extend_from_slice(proof);
    }
    out
}

fn segment_proof_bytes(
    file_name: &str,
    index: usize,
    segment_plan: &[u8],
    runtime_witness: &[u8],
    input_bytes: &[u8],
    replay_key: u64,
    result: DynamicValue,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(P27_SEGMENT_PROOF_DOMAIN);
    push_string(&mut out, file_name);
    push_u64(&mut out, index as u64);
    push_component(&mut out, segment_plan);
    push_component(&mut out, runtime_witness);
    push_component(&mut out, input_bytes);
    out.extend_from_slice(&replay_key.to_be_bytes());
    encode_dynamic_value(result, &mut out);
    out
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

fn validate_simple_name(
    name: &str,
    maximum: usize,
    label: &str,
    span: SourceSpan,
) -> AtomicBundleOutputResult<()> {
    if name.is_empty() {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("{label} must not be empty"),
            span: Some(span),
        });
    }
    if name.len() > maximum {
        return Err(AtomicBundleOutputError::Policy {
            message: format!(
                "{label} has {} bytes, exceeding bound {maximum}",
                name.len()
            ),
            span: Some(span),
        });
    }
    if name == "." || name == ".." {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("{label} must be one ordinary relative name"),
            span: Some(span),
        });
    }
    if name.contains('/') || name.contains('\\') {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("{label} must not contain path separators"),
            span: Some(span),
        });
    }
    if name.chars().any(char::is_control) {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("{label} must not contain control characters"),
            span: Some(span),
        });
    }
    if Path::new(name).is_absolute() {
        return Err(AtomicBundleOutputError::Policy {
            message: format!("absolute {label} paths are forbidden"),
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
) -> AtomicBundleOutputResult<()> {
    if token.kind() != &TokenKind::Identifier || source.slice(token.span())? != expected {
        return Err(AtomicBundleOutputError::Syntax {
            message: message.to_owned(),
            span: token.span(),
        });
    }
    Ok(())
}

fn decode_quoted_text(raw: &str, span: SourceSpan) -> AtomicBundleOutputResult<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(AtomicBundleOutputError::Syntax {
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
        let escaped = chars
            .next()
            .ok_or_else(|| AtomicBundleOutputError::Syntax {
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
                return Err(AtomicBundleOutputError::Syntax {
                    message: format!(
                        "unsupported text escape '\\{escaped}'; allowed escapes are \\\\, \\\" , \\n, \\r and \\t"
                    ),
                    span,
                });
            }
        }
    }
    Ok(output)
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
