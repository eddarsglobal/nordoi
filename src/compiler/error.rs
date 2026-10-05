use crate::frontend::{
    BodyError, ModuleError, PureExpressionError, PureResultError, SourceError, SourceSpan,
    TypeEffectError,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    Frontend(ModuleError),
    TypeEffectFrontend(TypeEffectError),
    BodyFrontend(BodyError),
    PureExpressionFrontend(PureExpressionError),
    PureResultFrontend(PureResultError),
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
    BodyLayerSpanMismatch {
        semantic_body: SourceSpan,
        body: SourceSpan,
    },
    EntrySpanOutsideBody {
        body: SourceSpan,
        entry: SourceSpan,
    },
    PureResultBodySpanMismatch {
        semantic_body: SourceSpan,
        body: SourceSpan,
    },
    PureResultEntrySpanOutsideBody {
        body: SourceSpan,
        entry: SourceSpan,
    },
    PureResultValueSpanOutsideEntry {
        entry: SourceSpan,
        result: SourceSpan,
    },
    PureExpressionBodySpanMismatch {
        semantic_body: SourceSpan,
        body: SourceSpan,
    },
    PureExpressionEntrySpanOutsideBody {
        body: SourceSpan,
        entry: SourceSpan,
    },
    PureExpressionSpanOutsideEntry {
        entry: SourceSpan,
        expression: SourceSpan,
    },
    PureExpressionNodeSpanOutsideExpression {
        expression: SourceSpan,
        node: SourceSpan,
    },
    TooManyPureExpressionNodes {
        count: u32,
        maximum: u32,
        span: SourceSpan,
    },
    InvalidPureExpressionPostfix {
        span: SourceSpan,
    },
    NegativePureExpressionLiteral {
        span: SourceSpan,
    },
    PureExpressionIntegerOverflow {
        span: SourceSpan,
    },
    ExecutablePlanEntryMustBePure {
        name: String,
    },
    PureResultPlanEntryMustBePure {
        name: String,
    },
    PureExpressionPlanEntryMustBePure {
        name: String,
    },
    NairLoweringRequiresZeroWork {
        count: u32,
    },
    NairLoweringRequiresPurePlan {
        count: u32,
    },
    NairLoweringRequiresNoAuthority,
    NairLoweringValidationFailed {
        message: String,
    },
    PureResultNairLoweringRequiresZeroWork {
        count: u32,
    },
    PureResultNairLoweringRequiresPurePlan {
        count: u32,
    },
    PureResultNairLoweringRequiresNoAuthority,
    PureResultNairLoweringValidationFailed {
        message: String,
    },
    PureExpressionNairLoweringRequiresZeroWork {
        count: u32,
    },
    PureExpressionNairLoweringRequiresPurePlan {
        count: u32,
    },
    PureExpressionNairLoweringRequiresNoAuthority,
    PureExpressionNairRegisterSpaceExhausted,
    PureExpressionNairInvalidPostfix {
        depth: u32,
    },
    PureExpressionNairLoweringValidationFailed {
        message: String,
    },
}

impl CompilerError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Frontend(error) => error.primary_span(),
            Self::TypeEffectFrontend(error) => error.primary_span(),
            Self::BodyFrontend(error) => error.primary_span(),
            Self::PureExpressionFrontend(error) => error.primary_span(),
            Self::PureResultFrontend(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::SourceMismatch { other, .. } => Some(*other),
            Self::ModuleSpanOutsideFile { module, .. } => Some(*module),
            Self::DeclarationSpanOutsideFile { declaration, .. }
            | Self::DeclarationStartsBeforePrelude { declaration, .. }
            | Self::DeclarationOrderViolation { declaration, .. }
            | Self::DeclarationOverlapsBody { declaration, .. } => Some(*declaration),
            Self::BodySpanOutsideFile { body, .. }
            | Self::BodyStartsBeforeModuleEnds { body, .. }
            | Self::BodyLayerSpanMismatch { body, .. } => Some(*body),
            Self::EntrySpanOutsideBody { entry, .. } => Some(*entry),
            Self::PureResultBodySpanMismatch { body, .. } => Some(*body),
            Self::PureResultEntrySpanOutsideBody { entry, .. } => Some(*entry),
            Self::PureResultValueSpanOutsideEntry { result, .. } => Some(*result),
            Self::PureExpressionBodySpanMismatch { body, .. } => Some(*body),
            Self::PureExpressionEntrySpanOutsideBody { entry, .. } => Some(*entry),
            Self::PureExpressionSpanOutsideEntry { expression, .. } => Some(*expression),
            Self::PureExpressionNodeSpanOutsideExpression { node, .. } => Some(*node),
            Self::TooManyPureExpressionNodes { span, .. }
            | Self::InvalidPureExpressionPostfix { span }
            | Self::NegativePureExpressionLiteral { span }
            | Self::PureExpressionIntegerOverflow { span } => Some(*span),
            Self::DuplicateSemanticDeclaration { duplicate, .. } => Some(*duplicate),
            Self::EmptySemanticName
            | Self::SemanticNameTooLong { .. }
            | Self::InvalidSemanticName { .. }
            | Self::EmptySemanticPath
            | Self::TooManySemanticSegments { .. }
            | Self::TooManySemanticDeclarations { .. }
            | Self::TooManyEffectRequirements { .. }
            | Self::DuplicateEffectRequirement { .. }
            | Self::UnknownEffectRequirement { .. }
            | Self::ExecutablePlanEntryMustBePure { .. }
            | Self::PureResultPlanEntryMustBePure { .. }
            | Self::PureExpressionPlanEntryMustBePure { .. }
            | Self::NairLoweringRequiresZeroWork { .. }
            | Self::NairLoweringRequiresPurePlan { .. }
            | Self::NairLoweringRequiresNoAuthority
            | Self::NairLoweringValidationFailed { .. }
            | Self::PureResultNairLoweringRequiresZeroWork { .. }
            | Self::PureResultNairLoweringRequiresPurePlan { .. }
            | Self::PureResultNairLoweringRequiresNoAuthority
            | Self::PureResultNairLoweringValidationFailed { .. }
            | Self::PureExpressionNairLoweringRequiresZeroWork { .. }
            | Self::PureExpressionNairLoweringRequiresPurePlan { .. }
            | Self::PureExpressionNairLoweringRequiresNoAuthority
            | Self::PureExpressionNairRegisterSpaceExhausted
            | Self::PureExpressionNairInvalidPostfix { .. }
            | Self::PureExpressionNairLoweringValidationFailed { .. } => None,
        }
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frontend(error) => Display::fmt(error, f),
            Self::TypeEffectFrontend(error) => Display::fmt(error, f),
            Self::BodyFrontend(error) => Display::fmt(error, f),
            Self::PureExpressionFrontend(error) => Display::fmt(error, f),
            Self::PureResultFrontend(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::EmptySemanticName => write!(f, "semantic name must not be empty"),
            Self::SemanticNameTooLong { bytes, maximum } => write!(
                f,
                "semantic name has {bytes} bytes; the compiler maximum is {maximum} bytes"
            ),
            Self::InvalidSemanticName { name } => write!(
                f,
                "semantic name '{}' does not satisfy the current ASCII identifier profile",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::EmptySemanticPath => {
                write!(f, "named semantic module path must contain at least one segment")
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
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::TooManyEffectRequirements { count, maximum } => write!(
                f,
                "semantic effect set has {count} requirements; the L0.4 maximum is {maximum}"
            ),
            Self::DuplicateEffectRequirement { name } => write!(
                f,
                "semantic effect set contains duplicate requirement '{}'",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::UnknownEffectRequirement { name } => write!(
                f,
                "semantic effect requirement '{}' is not declared in this module",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::SourceMismatch { .. } => {
                write!(f, "HIR spans from different source units cannot be combined")
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
                write!(f, "HIR declaration cannot overlap the residual unlowered body")
            }
            Self::BodySpanOutsideFile { .. } => {
                write!(f, "HIR body span must be contained by the file span")
            }
            Self::BodyStartsBeforeModuleEnds { .. } => {
                write!(f, "HIR body cannot begin before the module declaration ends")
            }
            Self::BodyLayerSpanMismatch { .. } => write!(
                f,
                "L0.5 body layer must cover exactly the residual body span published by L0.4"
            ),
            Self::EntrySpanOutsideBody { .. } => {
                write!(f, "L0.5 entry span must be contained by the residual body span")
            }
            Self::PureResultBodySpanMismatch { .. } => write!(
                f,
                "L0.6 pure-result layer must cover exactly the residual body span published by L0.4"
            ),
            Self::PureResultEntrySpanOutsideBody { .. } => write!(
                f,
                "L0.6 entry span must be contained by the residual body span"
            ),
            Self::PureResultValueSpanOutsideEntry { .. } => write!(
                f,
                "L0.6 result literal span must be contained by its entry span"
            ),
            Self::PureExpressionBodySpanMismatch { .. } => write!(
                f,
                "L0.7 pure-expression layer must cover exactly the residual body span published by L0.4"
            ),
            Self::PureExpressionEntrySpanOutsideBody { .. } => write!(
                f,
                "L0.7 entry span must be contained by the residual body span"
            ),
            Self::PureExpressionSpanOutsideEntry { .. } => write!(
                f,
                "L0.7 pure-expression span must be contained by its entry span"
            ),
            Self::PureExpressionNodeSpanOutsideExpression { .. } => write!(
                f,
                "L0.7 expression node span must be contained by the expression span"
            ),
            Self::TooManyPureExpressionNodes { count, maximum, .. } => write!(
                f,
                "L0.7 pure expression has {count} semantic nodes; the maximum is {maximum}"
            ),
            Self::InvalidPureExpressionPostfix { .. } => write!(
                f,
                "L0.7 pure expression postfix form is not a valid single-result expression"
            ),
            Self::NegativePureExpressionLiteral { .. } => write!(
                f,
                "L0.7 pure expression literals must remain non-negative"
            ),
            Self::PureExpressionIntegerOverflow { .. } => write!(
                f,
                "L0.7 pure integer addition overflowed the supported i64 range"
            ),
            Self::ExecutablePlanEntryMustBePure { name } => write!(
                f,
                "C0.3 executable plan entry '{}' must require zero semantic effects",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::PureResultPlanEntryMustBePure { name } => write!(
                f,
                "C0.5 pure-result execution plan entry '{}' must require zero semantic effects",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::PureExpressionPlanEntryMustBePure { name } => write!(
                f,
                "C0.7 pure-expression execution plan entry '{}' must require zero semantic effects",
                name.chars().flat_map(char::escape_default).collect::<String>()
            ),
            Self::NairLoweringRequiresZeroWork { count } => write!(
                f,
                "C0.4 NAIR lowering accepts only zero-work semantic plans; found {count} work items"
            ),
            Self::NairLoweringRequiresPurePlan { count } => write!(
                f,
                "C0.4 NAIR lowering accepts only pure semantic plans; found {count} required effects"
            ),
            Self::NairLoweringRequiresNoAuthority => write!(
                f,
                "C0.4 NAIR lowering accepts only plans requiring no host authority"
            ),
            Self::NairLoweringValidationFailed { message } => write!(
                f,
                "C0.4 produced invalid NAIR: {message}"
            ),
            Self::PureResultNairLoweringRequiresZeroWork { count } => write!(
                f,
                "C0.6 pure-result NAIR lowering accepts only zero-work plans; found {count} work items"
            ),
            Self::PureResultNairLoweringRequiresPurePlan { count } => write!(
                f,
                "C0.6 pure-result NAIR lowering accepts only pure plans; found {count} required effects"
            ),
            Self::PureResultNairLoweringRequiresNoAuthority => write!(
                f,
                "C0.6 pure-result NAIR lowering accepts only plans requiring no host authority"
            ),
            Self::PureResultNairLoweringValidationFailed { message } => write!(
                f,
                "C0.6 produced invalid NAIR: {message}"
            ),
            Self::PureExpressionNairLoweringRequiresZeroWork { count } => write!(
                f,
                "C0.8 pure-expression NAIR lowering accepts only zero-work plans; found {count} work items"
            ),
            Self::PureExpressionNairLoweringRequiresPurePlan { count } => write!(
                f,
                "C0.8 pure-expression NAIR lowering accepts only pure plans; found {count} required effects"
            ),
            Self::PureExpressionNairLoweringRequiresNoAuthority => write!(
                f,
                "C0.8 pure-expression NAIR lowering accepts only plans requiring no host authority"
            ),
            Self::PureExpressionNairRegisterSpaceExhausted => write!(
                f,
                "C0.8 pure-expression NAIR lowering exhausted the u32 SSA register space"
            ),
            Self::PureExpressionNairInvalidPostfix { depth } => write!(
                f,
                "C0.8 pure-expression NAIR lowering received an invalid postfix register stack with depth {depth}"
            ),
            Self::PureExpressionNairLoweringValidationFailed { message } => write!(
                f,
                "C0.8 produced invalid NAIR: {message}"
            ),
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frontend(error) => Some(error),
            Self::TypeEffectFrontend(error) => Some(error),
            Self::BodyFrontend(error) => Some(error),
            Self::PureExpressionFrontend(error) => Some(error),
            Self::PureResultFrontend(error) => Some(error),
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

impl From<BodyError> for CompilerError {
    fn from(value: BodyError) -> Self {
        Self::BodyFrontend(value)
    }
}

impl From<PureExpressionError> for CompilerError {
    fn from(value: PureExpressionError) -> Self {
        Self::PureExpressionFrontend(value)
    }
}

impl From<PureResultError> for CompilerError {
    fn from(value: PureResultError) -> Self {
        Self::PureResultFrontend(value)
    }
}

impl From<SourceError> for CompilerError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type CompilerResult<T> = Result<T, CompilerError>;
