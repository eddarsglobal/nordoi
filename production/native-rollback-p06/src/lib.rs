//! NORDOI Production Track P0.6A: rollback-witness and external monotonic-anchor contract.
//!
//! This crate does NOT implement a production-grade anchor backend. It defines the fail-closed
//! contract and recovery algorithm for a separately trusted monotonic anchor. A filesystem file
//! inside the same rollback domain is intentionally not provided here as an anchor implementation.
//!
//! The library has no normal dependency on the raw P0.1/P0.3 production crates. Integration is
//! expressed through `ProductionWitnessSource`; concrete P0.2/P0.4 adapters are exercised only by
//! the P0.6A integration tests and belong behind the selected production profile in P0.6B.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChainWitness {
    position: u64,
    digest: [u8; 32],
}

impl ChainWitness {
    pub const fn from_parts(position: u64, digest: [u8; 32]) -> Self {
        Self { position, digest }
    }

    pub const fn position(self) -> u64 {
        self.position
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// Narrow source contract required by the P0.6A recovery algorithm.
///
/// A deployable integration must provide this view from behind the selected protected production
/// profile. The P0.6A library intentionally does not import raw P0.1/P0.3 types itself.
pub trait ProductionWitnessSource {
    type Error;

    fn trust_witness(&self) -> ChainWitness;
    fn contains_trust_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error>;
    fn delivery_witness(&self) -> ChainWitness;
    fn contains_delivery_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductionAnchorRecord {
    generation: u64,
    trust: ChainWitness,
    delivery: ChainWitness,
}

impl ProductionAnchorRecord {
    pub const fn new(generation: u64, trust: ChainWitness, delivery: ChainWitness) -> Self {
        Self {
            generation,
            trust,
            delivery,
        }
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn trust(self) -> ChainWitness {
        self.trust
    }

    pub const fn delivery(self) -> ChainWitness {
        self.delivery
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorStoreError {
    Unavailable,
    Conflict,
    Corrupt,
}

/// Contract for a separately trusted, monotonic, atomically compare-and-advance anchor.
///
/// Implementations must live outside the rollback domain of the local P0.2/P0.4 files. The
/// compare-and-advance operation must be atomic with respect to concurrent writers. Returning Ok
/// means the new record is durably committed and immediately readable.
pub trait MonotonicAnchor {
    fn read(&mut self) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError>;

    fn compare_and_advance(
        &mut self,
        expected: Option<ProductionAnchorRecord>,
        next: ProductionAnchorRecord,
    ) -> Result<(), AnchorStoreError>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorOpenMode {
    Initialize,
    Recover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryError {
    AnchorUnavailable,
    AnchorConflict,
    AnchorCorrupt,
    MissingAnchor,
    AlreadyInitialized,
    InvalidGeneration,
    GenerationOverflow,
    TrustRollback,
    DeliveryRollback,
    TrustFork,
    DeliveryFork,
    LocalTrustChainInvalid,
    LocalDeliveryChainInvalid,
    AnchorDidNotCommit,
}

impl From<AnchorStoreError> for RecoveryError {
    fn from(value: AnchorStoreError) -> Self {
        match value {
            AnchorStoreError::Unavailable => Self::AnchorUnavailable,
            AnchorStoreError::Conflict => Self::AnchorConflict,
            AnchorStoreError::Corrupt => Self::AnchorCorrupt,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Relation {
    Exact,
    Ahead,
}

fn relation<F, E>(
    anchored: ChainWitness,
    current: ChainWitness,
    contains_anchored: F,
    rollback: RecoveryError,
    fork: RecoveryError,
    invalid_local: RecoveryError,
) -> Result<Relation, RecoveryError>
where
    F: FnOnce(ChainWitness) -> Result<bool, E>,
{
    if current.position() < anchored.position() {
        return Err(rollback);
    }
    if current.position() == anchored.position() {
        return if current.digest() == anchored.digest() {
            Ok(Relation::Exact)
        } else {
            Err(fork)
        };
    }
    match contains_anchored(anchored) {
        Ok(true) => Ok(Relation::Ahead),
        Ok(false) => Err(fork),
        Err(_) => Err(invalid_local),
    }
}

fn current_record<W: ProductionWitnessSource>(
    source: &W,
    generation: u64,
) -> ProductionAnchorRecord {
    ProductionAnchorRecord::new(
        generation,
        source.trust_witness(),
        source.delivery_witness(),
    )
}

fn confirm<A: MonotonicAnchor + ?Sized>(
    anchor: &mut A,
    expected: ProductionAnchorRecord,
) -> Result<ProductionAnchorRecord, RecoveryError> {
    match anchor.read()? {
        Some(actual) if actual == expected => Ok(actual),
        _ => Err(RecoveryError::AnchorDidNotCommit),
    }
}

/// Explicit first-boot initialization. Recovery NEVER auto-initializes a missing anchor.
pub fn initialize_production_anchor<A, W>(
    anchor: &mut A,
    source: &W,
) -> Result<ProductionAnchorRecord, RecoveryError>
where
    A: MonotonicAnchor,
    W: ProductionWitnessSource,
{
    if anchor.read()?.is_some() {
        return Err(RecoveryError::AlreadyInitialized);
    }
    let next = current_record(source, 1);
    anchor.compare_and_advance(None, next)?;
    confirm(anchor, next)
}

/// Verify an existing anchor against the currently opened, internally verified P0.2/P0.4 chains.
///
/// - Missing anchor on recovery fails closed.
/// - Anchor ahead of either local chain is a rollback.
/// - Same position with another digest is a fork.
/// - Local state ahead of the anchor may advance only when the anchored witness is a verified
///   ancestor of the current chain.
/// - The anchor update is compare-and-advance and then read-back verified.
pub fn verify_production_recovery<A, W>(
    anchor: &mut A,
    source: &W,
) -> Result<ProductionAnchorRecord, RecoveryError>
where
    A: MonotonicAnchor + ?Sized,
    W: ProductionWitnessSource,
{
    let anchored = anchor.read()?.ok_or(RecoveryError::MissingAnchor)?;
    if anchored.generation() == 0 {
        return Err(RecoveryError::InvalidGeneration);
    }

    let current = current_record(source, anchored.generation());

    let trust_relation = relation(
        anchored.trust(),
        current.trust(),
        |candidate| source.contains_trust_witness(candidate),
        RecoveryError::TrustRollback,
        RecoveryError::TrustFork,
        RecoveryError::LocalTrustChainInvalid,
    )?;

    let delivery_relation = relation(
        anchored.delivery(),
        current.delivery(),
        |candidate| source.contains_delivery_witness(candidate),
        RecoveryError::DeliveryRollback,
        RecoveryError::DeliveryFork,
        RecoveryError::LocalDeliveryChainInvalid,
    )?;

    if trust_relation == Relation::Exact && delivery_relation == Relation::Exact {
        return Ok(anchored);
    }

    let generation = anchored
        .generation()
        .checked_add(1)
        .ok_or(RecoveryError::GenerationOverflow)?;
    let next = current_record(source, generation);
    anchor.compare_and_advance(Some(anchored), next)?;
    confirm(anchor, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryAnchor {
        record: Option<ProductionAnchorRecord>,
        refuse: bool,
    }

    impl MonotonicAnchor for MemoryAnchor {
        fn read(&mut self) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
            Ok(self.record)
        }

        fn compare_and_advance(
            &mut self,
            expected: Option<ProductionAnchorRecord>,
            next: ProductionAnchorRecord,
        ) -> Result<(), AnchorStoreError> {
            if self.refuse || self.record != expected {
                return Err(AnchorStoreError::Conflict);
            }
            if let Some(previous) = expected {
                if next.generation() <= previous.generation() {
                    return Err(AnchorStoreError::Conflict);
                }
            } else if next.generation() != 1 {
                return Err(AnchorStoreError::Conflict);
            }
            self.record = Some(next);
            Ok(())
        }
    }

    #[test]
    fn relation_exact_requires_same_position_and_digest() {
        let witness = ChainWitness::from_parts(7, [3; 32]);
        assert_eq!(
            relation(
                witness,
                witness,
                |_| Ok::<_, ()>(false),
                RecoveryError::TrustRollback,
                RecoveryError::TrustFork,
                RecoveryError::LocalTrustChainInvalid,
            ),
            Ok(Relation::Exact)
        );
    }

    #[test]
    fn relation_older_local_is_rollback() {
        let anchored = ChainWitness::from_parts(9, [1; 32]);
        let current = ChainWitness::from_parts(8, [1; 32]);
        assert_eq!(
            relation(
                anchored,
                current,
                |_| Ok::<_, ()>(true),
                RecoveryError::TrustRollback,
                RecoveryError::TrustFork,
                RecoveryError::LocalTrustChainInvalid,
            ),
            Err(RecoveryError::TrustRollback)
        );
    }

    #[test]
    fn relation_same_position_other_digest_is_fork() {
        let anchored = ChainWitness::from_parts(9, [1; 32]);
        let current = ChainWitness::from_parts(9, [2; 32]);
        assert_eq!(
            relation(
                anchored,
                current,
                |_| Ok::<_, ()>(true),
                RecoveryError::TrustRollback,
                RecoveryError::TrustFork,
                RecoveryError::LocalTrustChainInvalid,
            ),
            Err(RecoveryError::TrustFork)
        );
    }

    #[test]
    fn relation_newer_local_requires_verified_ancestry() {
        let anchored = ChainWitness::from_parts(9, [1; 32]);
        let current = ChainWitness::from_parts(10, [2; 32]);
        assert_eq!(
            relation(
                anchored,
                current,
                |_| Ok::<_, ()>(true),
                RecoveryError::TrustRollback,
                RecoveryError::TrustFork,
                RecoveryError::LocalTrustChainInvalid,
            ),
            Ok(Relation::Ahead)
        );
        assert_eq!(
            relation(
                anchored,
                current,
                |_| Ok::<_, ()>(false),
                RecoveryError::TrustRollback,
                RecoveryError::TrustFork,
                RecoveryError::LocalTrustChainInvalid,
            ),
            Err(RecoveryError::TrustFork)
        );
    }

    #[test]
    fn anchor_contract_rejects_non_monotonic_compare_and_advance() {
        let mut anchor = MemoryAnchor::default();
        let first = ProductionAnchorRecord::new(
            1,
            ChainWitness::from_parts(1, [1; 32]),
            ChainWitness::from_parts(1, [2; 32]),
        );
        anchor.compare_and_advance(None, first).unwrap();
        assert_eq!(
            anchor.compare_and_advance(Some(first), first),
            Err(AnchorStoreError::Conflict)
        );
    }
}
