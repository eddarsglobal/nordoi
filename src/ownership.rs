use std::collections::HashMap;

use crate::{
    atom::AtomId,
    error::{AtomicError, AtomicResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomainId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipDomain {
    pub id: DomainId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct OwnershipRegistry {
    domains: HashMap<DomainId, OwnershipDomain>,
    owners: HashMap<AtomId, DomainId>,
}

impl Default for OwnershipRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipRegistry {
    pub fn new() -> Self {
        let root = OwnershipDomain {
            id: DomainId(0),
            name: "root".to_owned(),
        };

        let mut domains = HashMap::new();
        domains.insert(root.id, root);

        Self {
            domains,
            owners: HashMap::new(),
        }
    }

    pub fn root(&self) -> DomainId {
        DomainId(0)
    }

    pub fn register_domain(&mut self, domain: OwnershipDomain) -> AtomicResult<()> {
        if self.domains.contains_key(&domain.id) {
            return Err(AtomicError::DuplicateDomain(domain.id));
        }
        self.domains.insert(domain.id, domain);
        Ok(())
    }

    pub fn domain_exists(&self, id: DomainId) -> bool {
        self.domains.contains_key(&id)
    }

    pub fn domain(&self, id: DomainId) -> AtomicResult<&OwnershipDomain> {
        self.domains
            .get(&id)
            .ok_or(AtomicError::UnknownDomain(id))
    }

    pub fn assign(&mut self, atom: AtomId, owner: DomainId) -> AtomicResult<()> {
        if !self.domain_exists(owner) {
            return Err(AtomicError::UnknownDomain(owner));
        }
        self.owners.insert(atom, owner);
        Ok(())
    }

    pub fn owner_of(&self, atom: AtomId) -> AtomicResult<DomainId> {
        self.owners
            .get(&atom)
            .copied()
            .ok_or(AtomicError::UnknownAtom(atom))
    }

    pub fn require_owner(&self, atom: AtomId, attempted: DomainId) -> AtomicResult<()> {
        let owner = self.owner_of(atom)?;
        if owner == attempted {
            Ok(())
        } else {
            Err(AtomicError::OwnershipViolation {
                atom,
                owner,
                attempted,
            })
        }
    }

    pub fn transfer(
        &mut self,
        atom: AtomId,
        from: DomainId,
        to: DomainId,
    ) -> AtomicResult<()> {
        if !self.domain_exists(to) {
            return Err(AtomicError::UnknownDomain(to));
        }
        self.require_owner(atom, from)?;
        self.owners.insert(atom, to);
        Ok(())
    }
}
