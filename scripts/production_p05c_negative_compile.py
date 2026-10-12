#!/usr/bin/env python3
"""P0.5C negative compilation and policy-mutation tests."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import tomllib
from copy import deepcopy
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts/production_p05c_graph_check.py"
POLICY_PATH = ROOT / "production/p05c_build_policy.json"
PROFILE = ROOT / "production/native-effect-profile-p05c"

spec = importlib.util.spec_from_file_location("p05c_checker", CHECKER_PATH)
assert spec and spec.loader
checker = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = checker
spec.loader.exec_module(checker)

policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
with (PROFILE / "Cargo.toml").open("rb") as handle:
    manifest = tomllib.load(handle)
source = (PROFILE / "src/lib.rs").read_text(encoding="utf-8")


def expect_policy_denied(label: str, mutate) -> None:
    candidate = deepcopy(manifest)
    mutate(candidate)
    try:
        checker.validate_manifest_data(candidate, policy)
    except checker.PolicyError:
        print(f"DENIED: {label}")
        return
    raise AssertionError(f"mutation unexpectedly accepted: {label}")


def expect_source_denied(label: str, candidate: str) -> None:
    try:
        checker.validate_profile_source(candidate, policy)
    except checker.PolicyError:
        print(f"DENIED: {label}")
        return
    raise AssertionError(f"source mutation unexpectedly accepted: {label}")


def cargo_probe(label: str, rust_source: str, should_pass: bool) -> None:
    with tempfile.TemporaryDirectory(prefix="nordoi-p05c-probe-") as tmp:
        root = Path(tmp)
        (root / "src").mkdir()
        profile_path = PROFILE.resolve().as_posix()
        (root / "Cargo.toml").write_text(
            "[package]\n"
            f'name = "p05c-probe-{label.replace("_", "-")}"\n'
            'version = "0.0.0"\n'
            'edition = "2021"\n'
            'publish = false\n\n'
            "[dependencies]\n"
            f'nordoi-production-profile-p05c = {{ path = "{profile_path}" }}\n',
            encoding="utf-8",
        )
        (root / "src/main.rs").write_text(rust_source, encoding="utf-8")
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(root / "target")
        result = subprocess.run(
            ["cargo", "check", "--offline", "--quiet"],
            cwd=root,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        if should_pass and result.returncode != 0:
            raise AssertionError(f"allowed compile probe failed: {label}\n{result.stderr}")
        if not should_pass and result.returncode == 0:
            raise AssertionError(f"forbidden compile probe unexpectedly succeeded: {label}")
        print(("ALLOWED" if should_pass else "DENIED") + f": compile {label}")


def main() -> int:
    checker.validate_manifest_data(manifest, policy)
    checker.validate_profile_source(source, policy)

    cargo_probe(
        "profile_only",
        "use nordoi_production_profile_p05c::{profile_id, FencedProductionEffects};\n"
        "fn witness<S, B, A>(_: Option<FencedProductionEffects<S, B, A>>) {}\n"
        "fn main() { let _ = profile_id(); witness::<(), (), ()>(None); }\n",
        True,
    )
    cargo_probe(
        "raw_p03_import",
        "use nordoi_native_effect_p03::GuardedEffectBackend;\n"
        "fn main() { let _ = core::mem::size_of::<Option<GuardedEffectBackend<(), ()>>>(); }\n",
        False,
    )
    cargo_probe(
        "kernel_import",
        "use nordoi_kernel::EffectBackend;\n"
        "fn main() { fn _x<T: EffectBackend>() {} }\n",
        False,
    )
    cargo_probe(
        "trust_import",
        "use nordoi_native_trust_p01::durable::DurableTrustGate;\n"
        "fn main() { let _ = core::mem::size_of::<Option<DurableTrustGate>>(); }\n",
        False,
    )

    expect_policy_denied(
        "direct kernel dependency",
        lambda data: data.setdefault("dependencies", {}).__setitem__("nordoi_kernel", {"path": "../.."}),
    )
    expect_policy_denied(
        "second raw P0.3 alias",
        lambda data: data.setdefault("dependencies", {}).__setitem__(
            "raw_effects", {"package": "nordoi-native-effect-p03", "path": "../native-effect-p03"}
        ),
    )
    expect_policy_denied(
        "new Cargo feature",
        lambda data: data.__setitem__("features", {"raw": []}),
    )
    expect_source_denied(
        "raw backend re-export",
        source + "\npub use nordoi_native_effect_p03::GuardedEffectBackend;\n",
    )
    expect_source_denied(
        "kernel raw trait import",
        source + "\nuse nordoi_kernel::EffectBackend;\n",
    )
    expect_source_denied(
        "unsafe block",
        source + "\npub const fn bad() { unsafe {} }\n",
    )

    print("P0.5C NEGATIVE BUILD-GRAPH TESTS: 10/10 PASS")
    print("GLOBAL KERNEL CLOSURE: NOT CLAIMED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
