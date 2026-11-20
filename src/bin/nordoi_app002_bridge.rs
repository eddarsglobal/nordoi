//! APP-002 research CLI composition: actual V1.3 module result selects one P2.7
//! bundle source in a single native NORDOI process.
//!
//! This does NOT add general effectful imports to .noi, new NAIR opcodes,
//! distributed atomicity, native production security or external authority.
//! Explicit authority, two prevalidated P2.7 plans, identical capability sets,
//! 0/1 typed rule result and fail-closed paths are mandatory.

use super::*;

fn compile_app002_bundle_source(
    path: &OsStr,
) -> Result<nordoi_kernel::P27AtomicBundlePlan, u8> {
    let (name, text) = match load_source(path) {
        Ok(source) => source,
        Err(error) => {
            report_load_error(path, &error);
            return Err(EXIT_IO);
        }
    };
    let source = match SourceText::new(SourceId::new(1), name, text) {
        Ok(source) => source,
        Err(error) => {
            report_plain_error("module-bundle-write", &path.to_string_lossy(), &error);
            return Err(EXIT_FRONTEND);
        }
    };
    match compile_atomic_bundle_plan_p27(&source) {
        Ok(plan) => Ok(plan),
        Err(error) => {
            report_plain_error("module-bundle-write", source.name(), &error);
            Err(if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else {
                EXIT_RUNTIME
            })
        }
    }
}

pub(super) fn run_app002_module_bundle_cli(
    module_root: &Path,
    entry_module: &str,
    approved_source: &OsStr,
    rejected_source: &OsStr,
    key_code: u32,
    grant_dir: Option<&Path>,
) -> u8 {
    // Do not evaluate a rule, create staging, or touch output when ungranted.
    let output_dir = match grant_dir {
        Some(dir) => dir,
        None => {
            report_plain_error(
                "module-bundle-write",
                entry_module,
                "explicit --grant-output-dir required; denied by default",
            );
            return EXIT_RUNTIME;
        }
    };
    if approved_source == rejected_source {
        report_plain_error(
            "module-bundle-write",
            entry_module,
            "approved and rejected source paths must be different",
        );
        return EXIT_FRONTEND;
    }

    // Read and compile *both* output arms before any NORDOI runtime work or
    // file capability grant. Import resolution uses certified V1.3 bounds.
    let sources = match load_module_graph_sources_v13(module_root, entry_module) {
        Ok(sources) => sources,
        Err(exit) => return exit,
    };
    let approved = match compile_app002_bundle_source(approved_source) {
        Ok(plan) => plan,
        Err(exit) => return exit,
    };
    let rejected = match compile_app002_bundle_source(rejected_source) {
        Ok(plan) => plan,
        Err(exit) => return exit,
    };
    let expected_caps = approved.required_capabilities();
    if approved.bundle_name() != rejected.bundle_name()
        || approved.file_count() != rejected.file_count()
        || expected_caps != rejected.required_capabilities()
    {
        report_plain_error(
            "module-bundle-write",
            entry_module,
            "both branches must publish the identical bundle name and exact FileWrite targets",
        );
        return EXIT_FRONTEND;
    }

    // This is an authentic keyboard-input batch passed to the *real* bounded
    // module-graph runtime (no Python-computed decision or fixture value).
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let rule = match execute_module_graph_v13(&sources, entry_module, &input) {
        Ok(rule) => rule,
        Err(error) => {
            report_plain_error("module-bundle-write", entry_module, &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else {
                EXIT_RUNTIME
            };
        }
    };
    let (selected, selected_name) = match rule.result() {
        nordoi_kernel::DynamicValue::Int(1) => (&approved, "approved"),
        nordoi_kernel::DynamicValue::Int(0) => (&rejected, "rejected"),
        result => {
            report_plain_error(
                "module-bundle-write",
                entry_module,
                format_args!(
                    "module result must be INT(0) or INT(1), got {} (no publication)",
                    result.as_text()
                ),
            );
            return EXIT_RUNTIME;
        }
    };

    // Host grant is for *exactly* the prevalidated FileWrite targets.
    // One selected P2.7 plan receives original user input; neither profile
    // silently gains permission to read files, use network or spawn a process.
    let mut authority = CapabilitySet::new();
    for capability in expected_caps.iter().cloned() {
        authority.allow(capability);
    }
    let receipt = match execute_atomic_bundle_p27(selected, key_code, &authority, output_dir) {
        Ok(receipt) => receipt,
        Err(error) => {
            report_plain_error("module-bundle-write", entry_module, &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else if error.is_materialization_failure() {
                EXIT_IO
            } else {
                EXIT_RUNTIME
            };
        }
    };

    // Preserve both underlying canonical receipts; do not falsely claim
    // a new certified cross-profile cryptographic transaction.
    let rule_receipt_hex = hex_bytes(rule.canonical_v13_receipt_bytes());
    let output_receipt = receipt.render_text();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    if err.write_all(output_receipt.as_bytes()).is_err() || err.flush().is_err() {
        report_plain_error("io", "<stderr>", "APP002 failed to write authentic P2.7 receipt");
        return EXIT_IO;
    }

    let summary = format!(
        "module-bundle-write entry-module=\"{}\" modules={} original-key-code={} rule-result={} selected=\"{}\" bundle=\"{}\" files={} bridge=V13_RULE_SELECTS_P27_BRANCH status=COMMITTED authority=EXPLICIT grant-count={} ambient-authority=NONE rule-receipt-hex={} bundle-receipt-sha256={} crash-durability=UNCLAIMED\n",
        entry_module,
        rule.plan().module_count(),
        key_code,
        rule.result().as_text(),
        selected_name,
        receipt.bundle_name(),
        receipt.files().len(),
        expected_caps.len(),
        rule_receipt_hex,
        receipt.receipt_sha256_hex(),
    );
    match write_stdout(summary.as_bytes()) {
        Ok(()) => EXIT_OK,
        Err(error) => {
            report_io_error("<stdout>", &error);
            EXIT_IO
        }
    }
}
