use nordoi_kernel::parse_project_manifest_v14;

fn valid(extra: &str) -> String {
    format!(
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n{extra}"
    )
}

#[test]
fn valid_manifest_parses_and_canonicalizes() {
    let manifest = parse_project_manifest_v14(&valid("")).unwrap();
    assert_eq!(manifest.name(), "demo");
    assert_eq!(manifest.version(), "0.1.0");
    assert_eq!(manifest.entry_module(), "app.main");
    assert_eq!(manifest.source_root(), "src");
    assert_eq!(manifest.package_file_name(), "demo-0.1.0.npkg");
    assert_eq!(
        manifest.canonical_text(),
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n"
    );
}

#[test]
fn missing_project_section_is_rejected() {
    let error = parse_project_manifest_v14("name = \"demo\"\n").unwrap_err();
    assert!(format!("{error}").contains("before [project]"));
}

#[test]
fn duplicate_key_is_rejected() {
    let error = parse_project_manifest_v14(&valid("name = \"other\"\n")).unwrap_err();
    assert!(format!("{error}").contains("duplicate project key 'name'"));
}

#[test]
fn unknown_key_is_rejected() {
    let error = parse_project_manifest_v14(&valid("network = \"on\"\n")).unwrap_err();
    assert!(format!("{error}").contains("unsupported project key 'network'"));
}

#[test]
fn unsafe_project_name_is_rejected() {
    let text = "[project]\nname = \"../demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n";
    let error = parse_project_manifest_v14(text).unwrap_err();
    assert!(format!("{error}").contains("project name"));
}

#[test]
fn invalid_entry_module_is_rejected() {
    let text = "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app..main\"\nsource-root = \"src\"\n";
    assert!(parse_project_manifest_v14(text).is_err());
}

#[test]
fn source_root_rejects_traversal_and_absolute_paths() {
    for source_root in ["../src", "/tmp/src", "src/../other", "C:/src"] {
        let text = format!(
            "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"{source_root}\"\n"
        );
        assert!(parse_project_manifest_v14(&text).is_err(), "{source_root}");
    }
}

#[test]
fn comments_and_spacing_do_not_change_canonical_manifest() {
    let a = parse_project_manifest_v14(&valid("")).unwrap();
    let b = parse_project_manifest_v14(
        "# project\n[project]\n name = \"demo\" # x\n version=\"0.1.0\"\n entry = \"app.main\"\n source-root = \"src\"\n",
    )
    .unwrap();
    assert_eq!(a, b);
}
