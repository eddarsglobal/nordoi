use crate::frontend::{ModuleError, SourceError, SourceSpan, TypeEffectError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    Frontend(ModuleError),
    TypeEffectFrontend(TypeEffectError),
    Source(SourceError),
    EmptySemanticName,
    SemanticNameTooLong {
        bytes: u32,
        maximum: u32,
    },
    InvalidSemanticName {
        name: String,
    },
    EmptySemanticPath,
    TooManySemanticSegments {
        count: u32,
        maximum: u32,
    },
    TooManySemanticDeclarations {
        count: u32,
        maximum: u32,
    },
    DuplicateSemanticDeclaration {
        kind: &'static str,
        name: String,
        first: SourceSpan,
        duplicate: SourceSpan,
    },
    TooManyEffectRequirements {
        count: u32,
        maximum: u32,
    },
    DuplicateEffectRequirement {
        name: String,
    },
    UnknownEffectRequirement {
        name: String,
    },
    SourceMismatch {
        file: SourceSpan,
        other: SourceSpan,
    },
    ModuleSpanOutsideFile {
        file: SourceSpan,
        module: SourceSpan,
    },
    DeclarationSpanOutsideFile {
        file: SourceSpan,
        declaration: SourceSpan,
    },
    DeclarationStartsBeforePrelude {
        prelude_start: SourceSpan,
        declaration: SourceSpan,
    },
    DeclarationOrderViolation {
        previous: SourceSpan,
        declaration: SourceSpan,
    },
    DeclarationOverlapsBody {
        declaration: SourceSpan,
        body: SourceSpan,
    },
    BodySpanOutsideFile {
        file: SourceSpan,
        body: SourceSpan,
    },
    BodyStartsBeforeModuleEnds {
        module: SourceSpan,
        body: SourceSpan,
    },
}

impl CompilerError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Frontend(error) => error.primary_span(),
            Self::TypeEffectFrontend(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::SourceMismatch { other, .. } => Some(*other),
            Self::ModuleSpanOutsideFile { module, .. } => Some(*module),
            Self::DeclarationSpanOutsideFile { declaration, .. }
            | Self::DeclarationStartsBeforePrelude { declaration, .. }
            | Self::DeclarationOrderViolation { declaration, .. }
            | Self::DeclarationOverlapsBody { declaration, .. } => Some(*declaration),
            Self::BodySpanOutsideFile { body, .. }
            | Self::BodyStartsBeforeModuleEnds { body, .. } => Some(*body),
            Self::DuplicateSemanticDeclaration { duplicate, .. } => Some(*duplicate),
            Self::EmptySemanticName
            | Self::SemanticNameTooLong { .. }
            | Self::InvalidSemanticName { .. }
            | Self::EmptySemanticPath
            | Self::TooManySemanticSegments { .. }
            | Self::TooManySemanticDeclarations { .. }
            | Self::TooManyEffectRequirements { .. }
            | Self::DuplicateEffectRequirement { .. }
            | Self::UnknownEffectRequirement { .. } => None,
        }
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frontend(error) => Display::fmt(error, f),
            Self::TypeEffectFrontend(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::EmptySemanticName => write!(f, "semantic name must not be empty"),
            Self::SemanticNameTooLong { bytes, maximum } => write!(
                f,
                "semantic name has {bytes} bytes; the compiler maximum is {maximum} bytes"
            ),
            Self::InvalidSemanticName { name } => write!(
                f,
                "semantic name '{}' does not satisfy the current ASCII identifier profile",
                name.chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            Self::EmptySemanticPath => {
                write!(
                    f,
                    "named semantic module path must contain at least one segment"
                )
            }
            Self::TooManySemanticSegments { count, maximum } => write!(
                f,
                "semantic module path has {count} segments; the compiler maximum is {maximum}"
            ),
            Self::TooManySemanticDeclarations { count, maximum } => write!(
                f,
                "semantic prelude has {count} declarations; the L0.4 maximum is {maximum}"
            ),
            Self::DuplicateSemanticDeclaration { kind, name, .. } => write!(
                f,
                "duplicate {kind} declaration '{}' in the same module",
                name.chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            Self::TooManyEffectRequirements { count, maximum } => write!(
                f,
                "semantic effect set has {count} requirements; the L0.4 maximum is {maximum}"
            ),
            Self::DuplicateEffectRequirement { name } => write!(
                f,
                "semantic effect set contains duplicate requirement '{}'",
                name.chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            Self::UnknownEffectRequirement { name } => write!(
                f,
                "semantic effect requirement '{}' is not declared in this module",
                name.chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            Self::SourceMismatch { .. } => {
                write!(
                    f,
                    "HIR spans from different source units cannot be combined"
                )
            }
            Self::ModuleSpanOutsideFile { .. } => {
                write!(f, "HIR module span must be contained by the file span")
            }
            Self::DeclarationSpanOutsideFile { .. } => {
                write!(f, "HIR declaration span must be contained by the file span")
            }
            Self::DeclarationStartsBeforePrelude { .. } => write!(
                f,
                "HIR declaration cannot begin before the module declaration/prelude boundary"
            ),
            Self::DeclarationOrderViolation { .. } => {
                write!(f, "HIR declarations must be ordered and non-overlapping")
            }
            Self::DeclarationOverlapsBody { .. } => {
                write!(
                    f,
                    "HIR declaration cannot overlap the residual unlowered body"
                )
            }
            Self::BodySpanOutsideFile { .. } => {
                write!(f, "HIR body span must be contained by the file span")
            }
            Self::BodyStartsBeforeModuleEnds { .. } => {
                write!(
                    f,
                    "HIR body cannot begin before the module declaration ends"
                )
            }
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frontend(error) => Some(error),
            Self::TypeEffectFrontend(error) => Some(error),
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ModuleError> for CompilerError {
    fn from(value: ModuleError) -> Self {
        Self::Frontend(value)
    }
}

impl From<TypeEffectError> for CompilerError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffectFrontend(value)
    }
}

impl From<SourceError> for CompilerError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type CompilerResult<T> = Result<T, CompilerError>;
