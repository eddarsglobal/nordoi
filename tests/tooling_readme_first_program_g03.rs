//! G0.3 R3-R5: README makes official first use discoverable before optional build-once.
//! This is a documentation navigation guard, not a human usability experiment.

const README: &str = include_str!("../README.md");
const OFFICIAL: &str = include_str!("../docs/NORDOI_G02_FIRST_USE_QUICKSTART.md");
const OPTIONAL: &str =
    include_str!("../docs/NORDOI_G03_R3_R1_BUILD_ONCE_TUTORIAL_CANDIDATE.md");

#[test]
fn start_here_links_are_real_and_official_guide_comes_first() {
    let start = README.find("## Start here — run your first NORDOI program").unwrap();
    let official = README
        .find("[official first-use quickstart](docs/NORDOI_G02_FIRST_USE_QUICKSTART.md)")
        .unwrap();
    let optional = README
        .find(
            "[optional build-once, run-directly tutorial](docs/NORDOI_G03_R3_R1_BUILD_ONCE_TUTORIAL_CANDIDATE.md)",
        )
        .unwrap();
    let governance = README.find("## Two counters, never one").unwrap();

    assert!(start < official && official < optional && optional < governance);
    assert!(OFFICIAL.starts_with("# NORDOI G0.2 — First-use quickstart"));
    assert!(OPTIONAL.contains("NON-NORMATIVE DOCUMENTATION EXPERIMENT"));
}

#[test]
fn readme_preserves_real_command_and_explicit_authority() {
    const COMMAND: &str =
        "cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console";
    assert!(README.contains(COMMAND));
    assert!(OFFICIAL.contains(COMMAND));
    assert!(README.contains("effect ConsoleWrite;"));
    assert!(README.contains("explicit permission"));
    assert!(README.contains("declaring an effect is not a grant"));
    assert!(README.contains("not a replacement beginner guide"));
    assert!(README.contains("No improvement in human learning or NORDOI engine speed"));
    assert!(README.contains("# NORDOI G0.1 — Constitutional Conformance Matrix"));
}
