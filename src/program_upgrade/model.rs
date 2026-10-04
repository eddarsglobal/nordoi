use std::collections::{BTreeMap, BTreeSet};

use crate::{effect_audit::hash::sha256, nair::AtomSlot, time::LogicalTime};

use super::{RuntimeUpgradeError, RuntimeUpgradeResult};

const PLAN_HASH_DOMAIN: &[u8] = b"NORDOI-RUNTIME-UPGRADE-PLAN-1.0";
const LINEAGE_HASH_DOMAIN: &[u8] = b"NORDOI-RUNTIME-UPGRADE-LINEAGE-1.0";

pub const MAX_RUNTIME_UPGRADE_RULES: usize = 524_288;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ProgramEpoch(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeUpgradeHash(pub [u8; 32]);

impl RuntimeUpgradeHash {
    pub const ZERO: Self = Self([0; 32]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomUpgradeRule {
    Copy { source: AtomSlot, target: AtomSlot },
    DropSource { source: AtomSlot },
    KeepTargetDefault { target: AtomSlot },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUpgradePlan {
    source_program_hash: [u8; 32],
    target_program_hash: [u8; 32],
    source_epoch: ProgramEpoch,
    source_dispositions: BTreeMap<AtomSlot, Option<AtomSlot>>,
    target_defaults: BTreeSet<AtomSlot>,
    plan_hash: RuntimeUpgradeHash,
}

impl RuntimeUpgradePlan {
    pub fn new(
        source_program_hash: [u8; 32],
        target_program_hash: [u8; 32],
        source_epoch: ProgramEpoch,
        rules: impl IntoIterator<Item = AtomUpgradeRule>,
    ) -> RuntimeUpgradeResult<Self> {
        if source_program_hash == target_program_hash {
            return Err(RuntimeUpgradeError::SameProgram);
        }

        let mut source_dispositions = BTreeMap::new();
        let mut target_claims = BTreeSet::new();
        let mut target_defaults = BTreeSet::new();
        let mut rule_count = 0_usize;

        for rule in rules {
            rule_count = rule_count.saturating_add(1);
            if rule_count > MAX_RUNTIME_UPGRADE_RULES {
                return Err(RuntimeUpgradeError::PlanRuleLimitExceeded {
                    rules: rule_count,
                    limit: MAX_RUNTIME_UPGRADE_RULES,
                });
            }
            match rule {
                AtomUpgradeRule::Copy { source, target } => {
                    if source_dispositions.insert(source, Some(target)).is_some() {
                        return Err(RuntimeUpgradeError::DuplicateSourceAtomDisposition(source));
                    }
                    if !target_claims.insert(target) {
                        return Err(RuntimeUpgradeError::DuplicateTargetAtomDisposition(target));
                    }
                }
                AtomUpgradeRule::DropSource { source } => {
                    if source_dispositions.insert(source, None).is_some() {
                        return Err(RuntimeUpgradeError::DuplicateSourceAtomDisposition(source));
                    }
                }
                AtomUpgradeRule::KeepTargetDefault { target } => {
                    if !target_claims.insert(target) {
                        return Err(RuntimeUpgradeError::DuplicateTargetAtomDisposition(target));
                    }
                    target_defaults.insert(target);
                }
            }
        }

        let plan_hash = RuntimeUpgradeHash(hash_plan(
            source_program_hash,
            target_program_hash,
            source_epoch,
            &source_dispositions,
            &target_defaults,
        ));

        Ok(Self {
            source_program_hash,
            target_program_hash,
            source_epoch,
            source_dispositions,
            target_defaults,
            plan_hash,
        })
    }

    pub const fn source_program_hash(&self) -> [u8; 32] {
        self.source_program_hash
    }
    pub const fn target_program_hash(&self) -> [u8; 32] {
        self.target_program_hash
    }
    pub const fn source_epoch(&self) -> ProgramEpoch {
        self.source_epoch
    }
    pub const fn plan_hash(&self) -> RuntimeUpgradeHash {
        self.plan_hash
    }

    pub(crate) fn source_dispositions(&self) -> &BTreeMap<AtomSlot, Option<AtomSlot>> {
        &self.source_dispositions
    }

    pub(crate) fn target_defaults(&self) -> &BTreeSet<AtomSlot> {
        &self.target_defaults
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PLAN_HASH_DOMAIN);
        bytes.extend_from_slice(&self.source_program_hash);
        bytes.extend_from_slice(&self.target_program_hash);
        bytes.extend_from_slice(&self.source_epoch.0.to_le_bytes());
        bytes.extend_from_slice(&(self.source_dispositions.len() as u64).to_le_bytes());
        for (source, target) in &self.source_dispositions {
            bytes.extend_from_slice(&source.0.to_le_bytes());
            match target {
                Some(target) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&target.0.to_le_bytes());
                }
                None => bytes.push(0),
            }
        }
        bytes.extend_from_slice(&(self.target_defaults.len() as u64).to_le_bytes());
        for target in &self.target_defaults {
            bytes.extend_from_slice(&target.0.to_le_bytes());
        }
        bytes
    }

    pub fn verify_hash(&self) -> RuntimeUpgradeResult<()> {
        let actual = RuntimeUpgradeHash(sha256(&self.canonical_bytes()));
        if actual != self.plan_hash {
            return Err(RuntimeUpgradeError::UpgradeHashMismatch);
        }
        Ok(())
    }
}

fn hash_plan(
    source_program_hash: [u8; 32],
    target_program_hash: [u8; 32],
    source_epoch: ProgramEpoch,
    source_dispositions: &BTreeMap<AtomSlot, Option<AtomSlot>>,
    target_defaults: &BTreeSet<AtomSlot>,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PLAN_HASH_DOMAIN);
    bytes.extend_from_slice(&source_program_hash);
    bytes.extend_from_slice(&target_program_hash);
    bytes.extend_from_slice(&source_epoch.0.to_le_bytes());
    bytes.extend_from_slice(&(source_dispositions.len() as u64).to_le_bytes());
    for (source, target) in source_dispositions {
        bytes.extend_from_slice(&source.0.to_le_bytes());
        match target {
            Some(target) => {
                bytes.push(1);
                bytes.extend_from_slice(&target.0.to_le_bytes());
            }
            None => bytes.push(0),
        }
    }
    bytes.extend_from_slice(&(target_defaults.len() as u64).to_le_bytes());
    for target in target_defaults {
        bytes.extend_from_slice(&target.0.to_le_bytes());
    }
    sha256(&bytes)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeUpgradeAuthority {
    allowed: BTreeSet<([u8; 32], [u8; 32])>,
}

impl RuntimeUpgradeAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, source_program_hash: [u8; 32], target_program_hash: [u8; 32]) {
        self.allowed
            .insert((source_program_hash, target_program_hash));
    }

    pub fn revoke(&mut self, source_program_hash: [u8; 32], target_program_hash: [u8; 32]) {
        self.allowed
            .remove(&(source_program_hash, target_program_hash));
    }

    pub fn allows(&self, source_program_hash: [u8; 32], target_program_hash: [u8; 32]) -> bool {
        self.allowed
            .contains(&(source_program_hash, target_program_hash))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUpgradeLineageRecord {
    pub source_program_hash: [u8; 32],
    pub target_program_hash: [u8; 32],
    pub plan_hash: RuntimeUpgradeHash,
    pub source_checkpoint_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUpgradeReport {
    pub source_epoch: ProgramEpoch,
    pub target_epoch: ProgramEpoch,
    pub source_program_hash: [u8; 32],
    pub target_program_hash: [u8; 32],
    pub plan_hash: RuntimeUpgradeHash,
    pub source_checkpoint_hash: [u8; 32],
    pub lineage_root: RuntimeUpgradeHash,
    pub migrated_atoms: usize,
    pub dropped_atoms: usize,
    pub defaulted_atoms: usize,
    pub cycle: u64,
    pub logical_time: LogicalTime,
}

pub(crate) fn next_lineage_root(
    previous: RuntimeUpgradeHash,
    target_epoch: ProgramEpoch,
    source_program_hash: [u8; 32],
    target_program_hash: [u8; 32],
    plan_hash: RuntimeUpgradeHash,
    source_checkpoint_hash: [u8; 32],
) -> RuntimeUpgradeHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(LINEAGE_HASH_DOMAIN);
    bytes.extend_from_slice(&previous.0);
    bytes.extend_from_slice(&target_epoch.0.to_le_bytes());
    bytes.extend_from_slice(&source_program_hash);
    bytes.extend_from_slice(&target_program_hash);
    bytes.extend_from_slice(&plan_hash.0);
    bytes.extend_from_slice(&source_checkpoint_hash);
    RuntimeUpgradeHash(sha256(&bytes))
}
