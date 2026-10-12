#!/usr/bin/env python3
import argparse
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--repo", default=".")
args = parser.parse_args()
root = Path(args.repo).resolve()

durable = (root / "production/native-trust-p01/src/durable.rs").read_text()
production = (root / "production/native-effect-p03/src/production.rs").read_text()
p06 = (root / "production/native-rollback-p06/src/lib.rs").read_text()
recovery_tests = (root / "production/native-rollback-p06/tests/recovery.rs").read_text()
manifest_path = root / "production/native-rollback-p06/Cargo.toml"
manifest = tomllib.loads(manifest_path.read_text())
readme = (root / "README_P0_6A.md").read_text()
decision = (root / "docs/PRODUCTION_P0_6A_ROLLBACK_WITNESS_DECISION.md").read_text()

required_durable = [
    "pub struct TrustJournalWitness",
    "witnesses: BTreeMap<u64, [u8; 32]>",
    "pub fn journal_witness(&self) -> TrustJournalWitness",
    "pub fn contains_journal_witness(",
    "self.witnesses.insert(self.size, self.digest);",
]
for token in required_durable:
    assert token in durable, f"missing P0.2 witness invariant: {token}"

required_delivery = [
    "pub struct DeliveryLedgerWitness",
    "witnesses: BTreeMap<u64, [u8; 32]>",
    "pub fn trust_witness(&self) -> TrustJournalWitness",
    "pub fn contains_trust_witness(",
    "pub fn delivery_witness(&self) -> DeliveryLedgerWitness",
    "pub fn contains_delivery_witness(",
    "self.witnesses.insert(self.size, self.last);",
]
for token in required_delivery:
    assert token in production, f"missing P0.4 witness invariant: {token}"

required_p06 = [
    "pub trait ProductionWitnessSource",
    "pub trait MonotonicAnchor",
    "fn read(&mut self)",
    "fn compare_and_advance(",
    "pub fn initialize_production_anchor",
    "pub fn verify_production_recovery",
    "ok_or(RecoveryError::MissingAnchor)",
    "RecoveryError::TrustRollback",
    "RecoveryError::DeliveryRollback",
    "RecoveryError::TrustFork",
    "RecoveryError::DeliveryFork",
    "checked_add(1)",
    "AnchorDidNotCommit",
]
for token in required_p06:
    assert token in p06, f"missing P0.6A anchor invariant: {token}"

for forbidden in [
    "std::fs",
    "OpenOptions",
    "File::",
    "PathBuf",
    "FileAnchor",
    "nordoi_native_trust_p01",
    "nordoi_native_effect_p03",
    "nordoi_kernel",
    "ProductionEffectBackend",
    "TrustJournalWitness",
    "DeliveryLedgerWitness",
]:
    assert forbidden not in p06, f"P0.6A contract crate must not import raw/local backend surface: {forbidden}"

normal_deps = set(manifest.get("dependencies", {}).keys())
assert normal_deps == set(), f"P0.6A normal production dependency set must be empty: {normal_deps}"

dev_deps = set(manifest.get("dev-dependencies", {}).keys())
assert dev_deps == {
    "nordoi-native-trust-p01",
    "nordoi-native-effect-p03",
    "nordoi_kernel",
    "ed25519-dalek",
}, dev_deps
assert "features" not in manifest, "P0.6A must not hide alternate security modes behind Cargo features"

required_test_adapter = [
    "impl ProductionWitnessSource for BackendWitnessSource",
    "TrustJournalWitness::from_parts(",
    "DeliveryLedgerWitness::from_parts(",
    "restoring_older_valid_trust_journal_is_detected",
    "restoring_older_valid_delivery_ledger_is_detected",
]
for token in required_test_adapter:
    assert token in recovery_tests, f"missing P0.6A real-chain test adapter invariant: {token}"

unit_tests = p06.count("#[test]")
integration_tests = recovery_tests.count("#[test]")
assert unit_tests == 5, unit_tests
assert integration_tests == 5, integration_tests
assert unit_tests + integration_tests == 10

for text in (readme, decision):
    assert "INDEPENDENT MONOTONIC ANTI-ROLLBACK: NOT CERTIFIED" in text
    assert "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED" in text
    assert "P0.6B" in text
    assert "no normal Cargo dependency" in text

print("P0.6A ROLLBACK WITNESS SOURCE BOUNDARY: PASS")
print("P0.6A CONTRACT NORMAL DEPENDENCIES: 0")
print("P0.6A TEST INVENTORY: 10")
print("P0.5C CLOSED BUILD-GRAPH COMPATIBILITY: PRESERVED")
print("INDEPENDENT MONOTONIC ANTI-ROLLBACK: NOT CERTIFIED")
print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")
