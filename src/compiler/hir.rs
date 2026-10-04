use super::error::{CompilerError, CompilerResult};
use crate::frontend::SourceSpan;

pub const MAX_SEMANTIC_NAME_BYTES: u32 = 128;
pub const MAX_SEMANTIC_PATH_SEGMENTS: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticName(String);

impl SemanticName {
    pub fn new(value: impl Into<String>) -> CompilerResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(CompilerError::EmptySemanticName);
        }
        let bytes = value.len() as u32;
        if bytes > MAX_SEMANTIC_NAME_BYTES {
            return Err(CompilerError::SemanticNameTooLong {
                bytes,
                maximum: MAX_SEMANTIC_NAME_BYTES,
            });
        }
        if !is_ascii_identifier(&value) {
            return Err(CompilerError::InvalidSemanticName { name: value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticPath {
    segments: Vec<SemanticName>,
}

impl SemanticPath {
    pub fn new(segments: Vec<SemanticName>) -> CompilerResult<Self> {
        if segments.is_empty() {
            return Err(CompilerError::EmptySemanticPath);
        }
        let count = segments.len() as u32;
        if count > MAX_SEMANTIC_PATH_SEGMENTS {
            return Err(CompilerError::TooManySemanticSegments {
                count,
                maximum: MAX_SEMANTIC_PATH_SEGMENTS,
            });
        }
        Ok(Self { segments })
    }

    pub fn segments(&self) -> &[SemanticName] {
        &self.segments
    }

    pub fn canonical_text(&self) -> String {
        self.segments
            .iter()
            .map(SemanticName::as_str)
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticModuleIdentity {
    Anonymous,
    Named(SemanticPath),
}

impl SemanticModuleIdentity {
    pub fn canonical_text(&self) -> Option<String> {
        match self {
            Self::Anonymous => None,
            Self::Named(path) => Some(path.canonical_text()),
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.1-MODULE-ID\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);
        match self {
            Self::Anonymous => bytes.push(0),
            Self::Named(path) => {
                bytes.push(1);
                bytes.extend_from_slice(&(path.segments.len() as u32).to_be_bytes());
                for segment in &path.segments {
                    let raw = segment.as_str().as_bytes();
                    bytes.extend_from_slice(&(raw.len() as u32).to_be_bytes());
                    bytes.extend_from_slice(raw);
                }
            }
        }
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HirBodyState {
    Unlowered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirUnit {
    module: SemanticModuleIdentity,
    file_span: SourceSpan,
    module_span: Option<SourceSpan>,
    body_span: SourceSpan,
    body_state: HirBodyState,
}

impl HirUnit {
    pub fn new(
        module: SemanticModuleIdentity,
        file_span: SourceSpan,
        module_span: Option<SourceSpan>,
        body_span: SourceSpan,
        body_state: HirBodyState,
    ) -> Self {
        Self {
            module,
            file_span,
            module_span,
            body_span,
            body_state,
        }
    }

    pub const fn module(&self) -> &SemanticModuleIdentity {
        &self.module
    }

    pub const fn file_span(&self) -> SourceSpan {
        self.file_span
    }

    pub const fn module_span(&self) -> Option<SourceSpan> {
        self.module_span
    }

    pub const fn body_span(&self) -> SourceSpan {
        self.body_span
    }

    pub const fn body_state(&self) -> HirBodyState {
        self.body_state
    }
}

fn is_ascii_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
