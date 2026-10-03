use std::collections::{BTreeSet, HashMap};

use crate::{
    atom::{Atom, AtomId},
    capability::{Capability, CapabilitySet},
    dependency::DependencyGraph,
    error::{AtomicError, AtomicResult},
    ownership::{DomainId, OwnershipDomain, OwnershipRegistry},
    scheduler::AtomicScheduler,
    transaction::{AtomicTransaction, TransactionId, TransactionReport},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct AtomicKernel {
    atoms: HashMap<AtomId, Atom>,
    graph: DependencyGraph,
    scheduler: AtomicScheduler,
    capabilities: CapabilitySet,
    ownership: OwnershipRegistry,
    next_atom_id: u64,
    next_domain_id: u64,
    next_transaction_id: u64,
}

impl Default for AtomicKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicKernel {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::new(),
            graph: DependencyGraph::new(),
            scheduler: AtomicScheduler::new(),
            capabilities: CapabilitySet::new(),
            ownership: OwnershipRegistry::new(),
            next_atom_id: 1,
            next_domain_id: 1,
            next_transaction_id: 1,
        }
    }

    pub fn root_domain(&self) -> DomainId {
        self.ownership.root()
    }

    pub fn create_domain(&mut self, name: impl Into<String>) -> AtomicResult<DomainId> {
        let id = DomainId(self.next_domain_id);
        self.next_domain_id += 1;

        self.ownership.register_domain(OwnershipDomain {
            id,
            name: name.into(),
        })?;

        Ok(id)
    }

    pub fn domain(&self, id: DomainId) -> AtomicResult<&OwnershipDomain> {
        self.ownership.domain(id)
    }

    /// Backward-compatible K0.1/K0.2 creation: root owns the atom.
    pub fn create_atom(&mut self, value: impl Into<Value>) -> AtomId {
        let root = self.root_domain();
        self.create_atom_owned(root, value)
            .expect("root ownership domain must always exist")
    }

    pub fn create_atom_owned(
        &mut self,
        owner: DomainId,
        value: impl Into<Value>,
    ) -> AtomicResult<AtomId> {
        if !self.ownership.domain_exists(owner) {
            return Err(AtomicError::UnknownDomain(owner));
        }

        let id = AtomId(self.next_atom_id);
        self.next_atom_id += 1;
        self.atoms.insert(id, Atom::new(id, value.into()));
        self.ownership.assign(id, owner)?;
        Ok(id)
    }

    pub fn owner_of(&self, atom: AtomId) -> AtomicResult<DomainId> {
        self.ensure_exists(atom)?;
        self.ownership.owner_of(atom)
    }

    pub fn require_owner(&self, atom: AtomId, domain: DomainId) -> AtomicResult<()> {
        self.ensure_exists(atom)?;
        self.ownership.require_owner(atom, domain)
    }

    pub fn transfer_atom(
        &mut self,
        atom: AtomId,
        from: DomainId,
        to: DomainId,
    ) -> AtomicResult<()> {
        self.ensure_exists(atom)?;
        self.ownership.transfer(atom, from, to)
    }

    pub fn connect(&mut self, source: AtomId, dependent: AtomId) -> AtomicResult<()> {
        self.ensure_exists(source)?;
        self.ensure_exists(dependent)?;
        self.graph.add_dependency(source, dependent);
        Ok(())
    }

    /// Direct root-domain mutation retained for bootstrap compatibility.
    /// Domain-aware code should prefer transactions.
    pub fn set(&mut self, id: AtomId, next: impl Into<Value>) -> AtomicResult<bool> {
        self.set_owned(self.root_domain(), id, next)
    }

    pub fn set_owned(
        &mut self,
        domain: DomainId,
        id: AtomId,
        next: impl Into<Value>,
    ) -> AtomicResult<bool> {
        self.require_owner(id, domain)?;
        self.apply_single(id, next.into())
    }

    fn apply_single(&mut self, id: AtomId, next: Value) -> AtomicResult<bool> {
        self.ensure_exists(id)?;

        // Preserve the No Work Without Effect law: an identical write exits
        // before dependency traversal, mutation, versioning or scheduling.
        if self.get(id)? == &next {
            return Ok(false);
        }

        // Validate the dependency closure before mutating state.
        let affected = self.graph.affected(id)?;
        let changed = self
            .atoms
            .get_mut(&id)
            .expect("atom existence already checked")
            .set(next);

        debug_assert!(changed, "value equality was checked before mutation");

        self.scheduler.schedule(id);

        for affected_id in affected {
            self.scheduler.schedule(affected_id);
            if let Some(atom) = self.atoms.get_mut(&affected_id) {
                atom.dirty = true;
            }
        }

        Ok(true)
    }

    pub fn begin_transaction(&mut self, domain: DomainId) -> AtomicResult<AtomicTransaction> {
        if !self.ownership.domain_exists(domain) {
            return Err(AtomicError::UnknownDomain(domain));
        }

        let id = TransactionId(self.next_transaction_id);
        self.next_transaction_id += 1;
        Ok(AtomicTransaction::new(id, domain))
    }

    /// Commit is all-or-nothing with respect to staged state writes.
    /// Every ownership/version/dependency validation completes before mutation begins.
    pub fn commit(&mut self, tx: AtomicTransaction) -> AtomicResult<TransactionReport> {
        let staged_writes = tx.writes.len();

        // Phase 1: validate everything before changing any state.
        for (atom, staged) in &tx.writes {
            self.require_owner(*atom, tx.domain)?;
            let actual_version = self.version(*atom)?;
            if actual_version != staged.expected_version {
                return Err(AtomicError::TransactionConflict {
                    atom: *atom,
                    expected_version: staged.expected_version,
                    actual_version,
                });
            }
        }

        // Determine true changes and dependency closure before mutation.
        // This also detects graph errors before state is touched.
        let mut changed_atoms = Vec::new();
        let mut scheduled = BTreeSet::new();

        for (atom, staged) in &tx.writes {
            let current = self.get(*atom)?;
            if current == &staged.value {
                continue;
            }

            changed_atoms.push(*atom);
            scheduled.insert(*atom);
            for affected in self.graph.affected(*atom)? {
                scheduled.insert(affected);
            }
        }

        // Phase 2: state mutation. All fallible validation is already complete.
        for atom in &changed_atoms {
            let next = tx
                .writes
                .get(atom)
                .expect("changed atom must have a staged write")
                .value
                .clone();

            self.atoms
                .get_mut(atom)
                .expect("validated atom must still exist")
                .set(next);
        }

        // Phase 3: one deduplicated invalidation frontier.
        for atom in &scheduled {
            self.scheduler.schedule(*atom);
            if let Some(item) = self.atoms.get_mut(atom) {
                item.dirty = true;
            }
        }

        Ok(TransactionReport {
            id: tx.id,
            staged_writes,
            changed_atoms: changed_atoms.len(),
            scheduled_atoms: scheduled.len(),
        })
    }

    pub fn get(&self, id: AtomId) -> AtomicResult<&Value> {
        self.atoms
            .get(&id)
            .map(|a| &a.value)
            .ok_or(AtomicError::UnknownAtom(id))
    }

    pub fn version(&self, id: AtomId) -> AtomicResult<u64> {
        self.atoms
            .get(&id)
            .map(|a| a.version)
            .ok_or(AtomicError::UnknownAtom(id))
    }

    pub fn pending_work(&self) -> usize {
        self.scheduler.len()
    }

    pub fn flush(&mut self) -> Vec<AtomId> {
        let mut executed = Vec::new();

        while let Some(id) = self.scheduler.pop() {
            if let Some(atom) = self.atoms.get_mut(&id) {
                atom.clean();
            }
            executed.push(id);
        }

        executed
    }

    pub fn allow(&mut self, capability: Capability) {
        self.capabilities.allow(capability);
    }

    pub fn require(&self, capability: &Capability) -> AtomicResult<()> {
        if self.capabilities.contains(capability) {
            Ok(())
        } else {
            Err(AtomicError::CapabilityDenied(capability.clone()))
        }
    }

    fn ensure_exists(&self, id: AtomId) -> AtomicResult<()> {
        self.atoms
            .contains_key(&id)
            .then_some(())
            .ok_or(AtomicError::UnknownAtom(id))
    }
}
