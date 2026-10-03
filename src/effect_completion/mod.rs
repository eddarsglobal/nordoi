mod authority;
mod core;
mod error;
mod id;
mod model;

pub use authority::EffectCompletionAuthority;
pub(crate) use core::canonical_report_bytes;
pub use core::AtomicEffectCompletionCore;
pub use error::{EffectCompletionError, EffectCompletionResult};
pub use id::{EffectCompletionSequence, EffectCompletionSourceId};
pub use model::{
    EffectCompletion, EffectCompletionApplicationReport, EffectCompletionBatch,
    EffectCompletionBatchReport, EffectCompletionOutcome, EffectCompletionProjection,
    EffectCompletionProjectionValue, EffectCompletionWriteReport, MAX_EFFECT_COMPLETION_BATCH,
    MAX_EFFECT_COMPLETION_TEXT_BYTES,
};
