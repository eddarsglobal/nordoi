use nordoi_kernel::{
    kernel_semantic_stability_manifest, kernel_semantic_stability_manifest_bytes,
    kernel_semantic_stability_manifest_hash, ChangePolicy, SemanticKind, StabilityClass,
    KERNEL_STABILITY_MANIFEST_MAJOR, KERNEL_STABILITY_MANIFEST_MINOR, NAIR_FORMAT_MAJOR,
    NAIR_FORMAT_MINOR,
};
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn manifest_version_is_1_0() {
    assert_eq!(KERNEL_STABILITY_MANIFEST_MAJOR, 1);
    assert_eq!(KERNEL_STABILITY_MANIFEST_MINOR, 0);
}

#[test]
fn k118_does_not_increment_nair() {
    assert_eq!((NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR), (0, 6));
}

#[test]
fn manifest_has_no_duplicate_ids() {
    let mut ids = BTreeSet::new();
    for surface in kernel_semantic_stability_manifest() {
        assert!(ids.insert(surface.id()));
    }
}

#[test]
fn every_stable_surface_has_a_specification_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for surface in kernel_semantic_stability_manifest() {
        if surface.stability() == StabilityClass::Stable {
            assert!(
                root.join(surface.spec_path()).is_file(),
                "missing spec for {} at {}",
                surface.id(),
                surface.spec_path()
            );
        }
    }
}

#[test]
fn every_manifest_surface_has_non_empty_version() {
    for surface in kernel_semantic_stability_manifest() {
        assert!(!surface.version().is_empty(), "{}", surface.id());
    }
}

#[test]
fn stable_program_semantics_are_replay_relevant() {
    for surface in kernel_semantic_stability_manifest() {
        if surface.stability() == StabilityClass::Stable
            && surface.kind() == SemanticKind::ProgramSemantic
        {
            assert!(surface.replay_relevant(), "{}", surface.id());
        }
    }
}

#[test]
fn host_authority_never_changes_replay_identity_by_itself() {
    for surface in kernel_semantic_stability_manifest() {
        if surface.kind() == SemanticKind::HostAuthority {
            assert!(!surface.replay_relevant(), "{}", surface.id());
        }
    }
}

#[test]
fn stable_surface_never_claims_internal_only_change_policy() {
    for surface in kernel_semantic_stability_manifest() {
        if surface.stability() == StabilityClass::Stable {
            assert_ne!(surface.change_policy(), ChangePolicy::InternalOnly);
        }
    }
}

#[test]
fn program_serialized_authority_is_forbidden_everywhere() {
    for surface in kernel_semantic_stability_manifest() {
        assert!(!surface.program_serializes_authority(), "{}", surface.id());
    }
}

#[test]
fn rust_public_api_is_explicitly_not_frozen() {
    let surface = find("host.rust_public_api");
    assert_eq!(surface.stability(), StabilityClass::Experimental);
    assert_eq!(surface.kind(), SemanticKind::DeveloperSurface);
    assert_eq!(surface.change_policy(), ChangePolicy::VersionedEvolution);
}

#[test]
fn noi_surface_is_explicitly_not_frozen() {
    let surface = find("language.noi_surface");
    assert_eq!(surface.stability(), StabilityClass::Experimental);
    assert_eq!(surface.kind(), SemanticKind::DeveloperSurface);
    assert_eq!(surface.version(), "unfrozen");
}

#[test]
fn nair_0_6_is_stable_and_versioned() {
    let surface = find("nair.format");
    assert_eq!(surface.stability(), StabilityClass::Stable);
    assert_eq!(surface.kind(), SemanticKind::ProgramSemantic);
    assert_eq!(surface.change_policy(), ChangePolicy::VersionedEvolution);
    assert_eq!(surface.version(), "0.6");
}

#[test]
fn runtime_checkpoint_1_1_is_stable_and_versioned() {
    let surface = find("runtime.checkpoint.format");
    assert_eq!(surface.stability(), StabilityClass::Stable);
    assert_eq!(surface.kind(), SemanticKind::DurableEncoding);
    assert_eq!(surface.change_policy(), ChangePolicy::VersionedEvolution);
    assert_eq!(surface.version(), "NDRTSM01/1.1");
}

#[test]
fn program_upgrade_semantics_are_stable() {
    let surface = find("runtime.program_upgrade");
    assert_eq!(surface.stability(), StabilityClass::Stable);
    assert_eq!(surface.kind(), SemanticKind::ProgramSemantic);
    assert!(surface.replay_relevant());
}

#[test]
fn dynamic_timer_upgrade_semantics_are_stable() {
    let surface = find("runtime.timer_upgrade.dynamic");
    assert_eq!(surface.stability(), StabilityClass::Stable);
    assert_eq!(surface.version(), "K1.17");
}

#[test]
fn effect_dispatch_authority_is_host_only_semantics() {
    let surface = find("effect.dispatch.authority");
    assert_eq!(surface.kind(), SemanticKind::HostAuthority);
    assert!(!surface.program_serializes_authority());
    assert!(!surface.replay_relevant());
}

#[test]
fn audit_is_operational_not_program_replay_meaning() {
    let surface = find("effect.audit");
    assert_eq!(surface.kind(), SemanticKind::OperationalMetadata);
    assert!(!surface.replay_relevant());
}

#[test]
fn manifest_bytes_use_a_domain_and_are_nontrivial() {
    let bytes = kernel_semantic_stability_manifest_bytes();
    assert!(bytes.len() > 256);
    assert!(bytes.starts_with(b"NORDOI-KERNEL-SEMANTIC-STABILITY-MAP-1.0"));
}

#[test]
fn manifest_hash_is_not_zero() {
    assert_ne!(kernel_semantic_stability_manifest_hash().bytes(), [0; 32]);
}

#[test]
fn manifest_hash_matches_k118_golden_identity() {
    assert_eq!(
        kernel_semantic_stability_manifest_hash().bytes(),
        [
            0x0c, 0xab, 0x1d, 0xca, 0xf9, 0x3b, 0x82, 0xd1, 0x97, 0x47, 0x01, 0x05, 0x87, 0x41,
            0xb2, 0x71, 0xdc, 0xb5, 0x50, 0x37, 0x4e, 0xff, 0xa4, 0xd0, 0x47, 0xc2, 0x7a, 0xdb,
            0x92, 0xef, 0xdb, 0x29,
        ]
    );
}

#[test]
fn all_required_kernel_domains_are_classified() {
    for id in [
        "atomic.transaction",
        "atomic.ownership",
        "atomic.capability",
        "input.core",
        "time.logical",
        "reaction.core",
        "render.semantic",
        "effect.intent",
        "effect.completion",
        "runtime.checkpoint.format",
        "runtime.program_upgrade",
        "nair.format",
    ] {
        let _ = find(id);
    }
}

#[test]
fn developer_surfaces_are_not_mislabeled_as_stable_kernel_semantics() {
    for surface in kernel_semantic_stability_manifest() {
        if surface.kind() == SemanticKind::DeveloperSurface {
            assert_ne!(
                surface.stability(),
                StabilityClass::Stable,
                "{}",
                surface.id()
            );
        }
    }
}

fn find(id: &str) -> &'static nordoi_kernel::SemanticSurface {
    kernel_semantic_stability_manifest()
        .iter()
        .find(|surface| surface.id() == id)
        .unwrap_or_else(|| panic!("missing semantic surface {id}"))
}
