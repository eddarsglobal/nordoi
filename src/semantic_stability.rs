use crate::effect_audit::hash::sha256;

pub const KERNEL_STABILITY_MANIFEST_MAJOR: u16 = 1;
pub const KERNEL_STABILITY_MANIFEST_MINOR: u16 = 0;
pub const KERNEL_STABILITY_MANIFEST_DOMAIN: &[u8] = b"NORDOI-KERNEL-SEMANTIC-STABILITY-MAP-1.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StabilityClass {
    Stable,
    Experimental,
    Internal,
}

impl StabilityClass {
    const fn tag(self) -> u8 {
        match self {
            Self::Stable => 1,
            Self::Experimental => 2,
            Self::Internal => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticKind {
    ProgramSemantic,
    HostAuthority,
    OperationalMetadata,
    DurableEncoding,
    DeveloperSurface,
}

impl SemanticKind {
    const fn tag(self) -> u8 {
        match self {
            Self::ProgramSemantic => 1,
            Self::HostAuthority => 2,
            Self::OperationalMetadata => 3,
            Self::DurableEncoding => 4,
            Self::DeveloperSurface => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChangePolicy {
    AdditiveOnly,
    VersionedEvolution,
    InternalOnly,
}

impl ChangePolicy {
    const fn tag(self) -> u8 {
        match self {
            Self::AdditiveOnly => 1,
            Self::VersionedEvolution => 2,
            Self::InternalOnly => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticSurface {
    id: &'static str,
    stability: StabilityClass,
    kind: SemanticKind,
    change_policy: ChangePolicy,
    replay_relevant: bool,
    program_serializes_authority: bool,
    version: &'static str,
    spec_path: &'static str,
}

impl SemanticSurface {
    pub const fn id(&self) -> &'static str {
        self.id
    }

    pub const fn stability(&self) -> StabilityClass {
        self.stability
    }

    pub const fn kind(&self) -> SemanticKind {
        self.kind
    }

    pub const fn change_policy(&self) -> ChangePolicy {
        self.change_policy
    }

    pub const fn replay_relevant(&self) -> bool {
        self.replay_relevant
    }

    pub const fn program_serializes_authority(&self) -> bool {
        self.program_serializes_authority
    }

    pub const fn version(&self) -> &'static str {
        self.version
    }

    pub const fn spec_path(&self) -> &'static str {
        self.spec_path
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SemanticStabilityManifestHash([u8; 32]);

impl SemanticStabilityManifestHash {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

const S: StabilityClass = StabilityClass::Stable;
const E: StabilityClass = StabilityClass::Experimental;
const I: StabilityClass = StabilityClass::Internal;
const P: SemanticKind = SemanticKind::ProgramSemantic;
const H: SemanticKind = SemanticKind::HostAuthority;
const O: SemanticKind = SemanticKind::OperationalMetadata;
const D: SemanticKind = SemanticKind::DurableEncoding;
const V: SemanticKind = SemanticKind::DeveloperSurface;
const A: ChangePolicy = ChangePolicy::AdditiveOnly;
const X: ChangePolicy = ChangePolicy::VersionedEvolution;
const N: ChangePolicy = ChangePolicy::InternalOnly;

macro_rules! surface {
    (
        $id:expr,
        $stability:expr,
        $kind:expr,
        $change_policy:expr,
        $replay_relevant:expr,
        $program_serializes_authority:expr,
        $version:expr,
        $spec_path:expr $(,)?
    ) => {
        SemanticSurface {
            id: $id,
            stability: $stability,
            kind: $kind,
            change_policy: $change_policy,
            replay_relevant: $replay_relevant,
            program_serializes_authority: $program_serializes_authority,
            version: $version,
            spec_path: $spec_path,
        }
    };
}

pub static KERNEL_SEMANTIC_STABILITY_MANIFEST: &[SemanticSurface] = &[
    surface!(
        "atomic.capability",
        S,
        P,
        A,
        true,
        false,
        "K1.0+",
        "laws/LAW_0001_NORDOI_MASTER_LAW.md"
    ),
    surface!(
        "atomic.ownership",
        S,
        P,
        A,
        true,
        false,
        "K1.0+",
        "laws/LAW_0001_NORDOI_MASTER_LAW.md"
    ),
    surface!(
        "atomic.transaction",
        S,
        P,
        A,
        true,
        false,
        "K1.0+",
        "docs/ATOMIC_RUNTIME_SPEC_1_0.md"
    ),
    surface!(
        "effect.attestation",
        S,
        O,
        X,
        false,
        false,
        "K1.11",
        "docs/EFFECT_AUDIT_ATTESTATION_SPEC_1_0.md"
    ),
    surface!(
        "effect.audit",
        S,
        O,
        X,
        false,
        false,
        "K1.10",
        "docs/EFFECT_ATTEMPT_AUDIT_IN_DOUBT_SPEC_1_0.md"
    ),
    surface!(
        "effect.completion",
        S,
        P,
        A,
        true,
        false,
        "K1.12/K1.13",
        "docs/EFFECT_COMPLETION_REENTRY_SPEC_1_0.md"
    ),
    surface!(
        "effect.dispatch.authority",
        S,
        H,
        A,
        false,
        false,
        "K1.6",
        "docs/EFFECT_OUTBOX_DISPATCH_SPEC_1_0.md"
    ),
    surface!(
        "effect.fencing",
        S,
        O,
        X,
        false,
        false,
        "K1.8",
        "docs/EFFECT_FENCING_PROTOCOL_SPEC_1_0.md"
    ),
    surface!(
        "effect.intent",
        S,
        P,
        A,
        true,
        false,
        "K1.6",
        "docs/EFFECT_OUTBOX_DISPATCH_SPEC_1_0.md"
    ),
    surface!(
        "effect.persistence.format",
        S,
        D,
        X,
        false,
        false,
        "NDEFXJ01+",
        "docs/EFFECT_JOURNAL_RECOVERY_SPEC_1_0.md"
    ),
    surface!(
        "effect.retry",
        S,
        O,
        X,
        false,
        false,
        "K1.9",
        "docs/EFFECT_RETRY_DEAD_LETTER_SPEC_1_0.md"
    ),
    surface!(
        "host.effect_backend",
        S,
        H,
        A,
        false,
        false,
        "K1.6+",
        "docs/EFFECT_OUTBOX_DISPATCH_SPEC_1_0.md"
    ),
    surface!(
        "host.rust_public_api",
        E,
        V,
        X,
        false,
        false,
        "1.x",
        "docs/KERNEL_SEMANTIC_STABILITY_MAP_1_0.md"
    ),
    surface!(
        "input.core",
        S,
        P,
        A,
        true,
        false,
        "K1.2+",
        "docs/INPUT_CORE_SPEC_0_1.md"
    ),
    surface!(
        "kernel.semantic_stability_manifest",
        S,
        O,
        X,
        false,
        false,
        "1.0",
        "docs/KERNEL_SEMANTIC_STABILITY_MAP_1_0.md"
    ),
    surface!(
        "language.noi_surface",
        E,
        V,
        X,
        true,
        false,
        "unfrozen",
        "research/LANGUAGE_INTELLIGENCE_CHARTER.md"
    ),
    surface!(
        "nair.format",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_SPEC_0_6.md"
    ),
    surface!(
        "nair.native_completion",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_NATIVE_EFFECT_COMPLETION_SPEC_0_1.md"
    ),
    surface!(
        "nair.native_input",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_NATIVE_INPUT_SPEC_0_1.md"
    ),
    surface!(
        "nair.native_reaction",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_NATIVE_REACTION_SPEC_0_1.md"
    ),
    surface!(
        "nair.native_render",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_NATIVE_RENDER_SPEC_0_1.md"
    ),
    surface!(
        "nair.native_time",
        S,
        P,
        X,
        true,
        false,
        "0.6",
        "docs/NAIR_NATIVE_TIME_SPEC_0_1.md"
    ),
    surface!(
        "reaction.core",
        S,
        P,
        A,
        true,
        false,
        "K1.4+",
        "docs/REACTION_ACTION_CORE_SPEC_1_0.md"
    ),
    surface!(
        "render.backend",
        S,
        H,
        A,
        false,
        false,
        "K1.x",
        "docs/RENDER_CORE_SPEC_0_1.md"
    ),
    surface!(
        "render.semantic",
        S,
        P,
        A,
        true,
        false,
        "K1.x",
        "docs/RENDER_CORE_SPEC_0_1.md"
    ),
    surface!(
        "runtime.checkpoint.format",
        S,
        D,
        X,
        false,
        false,
        "NDRTSM01/1.1",
        "docs/RUNTIME_SEMANTIC_CHECKPOINT_RECOVERY_SPEC_1_0.md"
    ),
    surface!(
        "runtime.closed_execution",
        S,
        P,
        A,
        true,
        false,
        "K1.x",
        "docs/ATOMIC_RUNTIME_SPEC_1_0.md"
    ),
    surface!(
        "runtime.event_loop",
        S,
        P,
        A,
        true,
        false,
        "K1.17",
        "docs/TIME_EVENT_LOOP_SPEC_1_2.md"
    ),
    surface!(
        "runtime.program_upgrade",
        S,
        P,
        A,
        true,
        false,
        "K1.15+",
        "docs/RUNTIME_PROGRAM_UPGRADE_MIGRATION_SPEC_1_0.md"
    ),
    surface!(
        "runtime.timer_upgrade.dynamic",
        S,
        P,
        A,
        true,
        false,
        "K1.17",
        "docs/RUNTIME_DYNAMIC_TIMER_UPGRADE_CONTINUITY_SPEC_1_0.md"
    ),
    surface!(
        "runtime.timer_upgrade.native",
        S,
        P,
        A,
        true,
        false,
        "K1.16",
        "docs/RUNTIME_TIMER_UPGRADE_CONTINUITY_SPEC_1_0.md"
    ),
    surface!(
        "time.logical",
        S,
        P,
        A,
        true,
        false,
        "K1.3+",
        "docs/NAIR_NATIVE_TIME_SPEC_0_1.md"
    ),
    surface!(
        "tooling.compiler_frontend",
        I,
        V,
        N,
        false,
        false,
        "not-yet-certified",
        "docs/KERNEL_SEMANTIC_STABILITY_MAP_1_0.md"
    ),
];

pub fn kernel_semantic_stability_manifest() -> &'static [SemanticSurface] {
    KERNEL_SEMANTIC_STABILITY_MANIFEST
}

pub fn kernel_semantic_stability_manifest_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(KERNEL_STABILITY_MANIFEST_DOMAIN);
    bytes.extend_from_slice(&KERNEL_STABILITY_MANIFEST_MAJOR.to_be_bytes());
    bytes.extend_from_slice(&KERNEL_STABILITY_MANIFEST_MINOR.to_be_bytes());
    bytes.extend_from_slice(&(KERNEL_SEMANTIC_STABILITY_MANIFEST.len() as u64).to_be_bytes());

    for surface in KERNEL_SEMANTIC_STABILITY_MANIFEST {
        push_text(&mut bytes, surface.id);
        bytes.push(surface.stability.tag());
        bytes.push(surface.kind.tag());
        bytes.push(surface.change_policy.tag());
        bytes.push(u8::from(surface.replay_relevant));
        bytes.push(u8::from(surface.program_serializes_authority));
        push_text(&mut bytes, surface.version);
        push_text(&mut bytes, surface.spec_path);
    }

    bytes
}

pub fn kernel_semantic_stability_manifest_hash() -> SemanticStabilityManifestHash {
    SemanticStabilityManifestHash::from_bytes(sha256(&kernel_semantic_stability_manifest_bytes()))
}

fn push_text(bytes: &mut Vec<u8>, text: &str) {
    let len = u64::try_from(text.len()).expect("semantic stability manifest text length fits u64");
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(text.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_non_empty() {
        assert!(!kernel_semantic_stability_manifest().is_empty());
    }

    #[test]
    fn manifest_ids_are_sorted_and_unique() {
        for pair in KERNEL_SEMANTIC_STABILITY_MANIFEST.windows(2) {
            assert!(pair[0].id() < pair[1].id());
        }
    }

    #[test]
    fn stable_surface_never_uses_internal_change_policy() {
        for surface in KERNEL_SEMANTIC_STABILITY_MANIFEST {
            if surface.stability() == StabilityClass::Stable {
                assert_ne!(surface.change_policy(), ChangePolicy::InternalOnly);
            }
        }
    }

    #[test]
    fn host_authority_is_never_program_serialized() {
        for surface in KERNEL_SEMANTIC_STABILITY_MANIFEST {
            if surface.kind() == SemanticKind::HostAuthority {
                assert!(!surface.program_serializes_authority());
            }
        }
    }

    #[test]
    fn operational_metadata_is_not_replay_semantic() {
        for surface in KERNEL_SEMANTIC_STABILITY_MANIFEST {
            if surface.kind() == SemanticKind::OperationalMetadata {
                assert!(!surface.replay_relevant());
            }
        }
    }

    #[test]
    fn durable_encodings_are_not_replay_events() {
        for surface in KERNEL_SEMANTIC_STABILITY_MANIFEST {
            if surface.kind() == SemanticKind::DurableEncoding {
                assert!(!surface.replay_relevant());
            }
        }
    }

    #[test]
    fn noi_surface_remains_experimental() {
        let noi = KERNEL_SEMANTIC_STABILITY_MANIFEST
            .iter()
            .find(|surface| surface.id() == "language.noi_surface")
            .unwrap();
        assert_eq!(noi.stability(), StabilityClass::Experimental);
        assert_eq!(noi.version(), "unfrozen");
    }

    #[test]
    fn compiler_frontend_is_not_certified_kernel_semantics() {
        let compiler = KERNEL_SEMANTIC_STABILITY_MANIFEST
            .iter()
            .find(|surface| surface.id() == "tooling.compiler_frontend")
            .unwrap();
        assert_eq!(compiler.stability(), StabilityClass::Internal);
        assert_eq!(compiler.change_policy(), ChangePolicy::InternalOnly);
    }

    #[test]
    fn canonical_bytes_are_stable_for_equal_manifest() {
        assert_eq!(
            kernel_semantic_stability_manifest_bytes(),
            kernel_semantic_stability_manifest_bytes()
        );
    }

    #[test]
    fn manifest_hash_is_stable_for_equal_manifest() {
        assert_eq!(
            kernel_semantic_stability_manifest_hash(),
            kernel_semantic_stability_manifest_hash()
        );
    }
}
