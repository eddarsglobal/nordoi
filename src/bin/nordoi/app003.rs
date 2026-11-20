//! APP-003 — bounded native single-entry-NOI frontend (RESEARCH ONLY).
//!
//! A single effectful entry module contains imports, a pure selector and
//! both file-output arms. This frontend tokenizes that actual NOI source,
//! validates the narrow APP003 grammar, and compiles three internal sources:
//! 1. a real imported V1.3 module graph for the selector,
//! 2. an approved P2.7 bundle plan,
//! 3. a rejected P2.7 bundle plan.
//!
//! Each historical V1.3/P2.7 compiler and runtime remains unchanged.
//! This is not a general new effectful import system, novel NAIR
//! instruction, certified cross-profile proof or admission to production.

use super::*;

#[derive(Clone)]
struct App003Token {
    kind: TokenKind,
    raw: String,
    start: usize,
    end: usize,
}

struct App003Cursor<'a> {
    source: &'a SourceText,
    tokens: Vec<App003Token>,
    at: usize,
}

#[derive(Debug)]
struct App003Compiled {
    entry_module: String,
    pure_entry: SourceText,
    approved: nordoi_kernel::P27AtomicBundlePlan,
    rejected: nordoi_kernel::P27AtomicBundlePlan,
}

impl<'a> App003Cursor<'a> {
    fn new(source: &'a SourceText) -> Result<Self, String> {
        let lexed = lex(source).map_err(|e| format!("APP003 lexical error: {e}"))?;
        let mut tokens = Vec::new();
        for token in lexed.into_iter().filter(|t| !t.is_trivia() && t.kind() != &TokenKind::Eof) {
            let span = token.span();
            let raw = source
                .slice(span)
                .map_err(|e| format!("APP003 invalid token span: {e}"))?
                .to_owned();
            tokens.push(App003Token {
                kind: token.kind().clone(),
                raw,
                start: span.start().get() as usize,
                end: span.end().get() as usize,
            });
        }
        Ok(Self { source, tokens, at: 0 })
    }

    fn peek(&self) -> Option<&App003Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Result<App003Token, String> {
        let token = self.peek().cloned().ok_or_else(|| {
            format!("APP003 unexpected end of source at token {}", self.at)
        })?;
        self.at += 1;
        Ok(token)
    }

    fn eat(&mut self, expected: &str) -> bool {
        if self.peek().is_some_and(|t| t.raw == expected) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &str) -> Result<App003Token, String> {
        let token = self.next()?;
        if token.raw != expected {
            return Err(format!(
                "APP003 expected '{expected}', got '{}' at byte {}",
                token.raw, token.start
            ));
        }
        Ok(token)
    }

    fn identifier(&mut self) -> Result<String, String> {
        let token = self.next()?;
        if token.kind != TokenKind::Identifier {
            return Err(format!(
                "APP003 expected identifier at byte {}, got '{}'",
                token.start, token.raw
            ));
        }
        Ok(token.raw)
    }

    fn module_path(&mut self) -> Result<String, String> {
        let mut name = self.identifier()?;
        while self.eat(".") {
            let part = self.identifier()?;
            name.push('.');
            name.push_str(&part);
        }
        validate_module_name_v13(&name)
            .map_err(|e| format!("APP003 invalid module path '{name}': {e}"))?;
        Ok(name)
    }

    fn quoted(&mut self) -> Result<String, String> {
        let token = self.next()?;
        if token.kind != TokenKind::QuotedText {
            return Err(format!(
                "APP003 expected quoted bundle name at byte {}",
                token.start
            ));
        }
        Ok(token.raw)
    }

    fn braced_source(&mut self) -> Result<String, String> {
        self.expect("{")?;
        let open_end = self.tokens[self.at - 1].end;
        let mut depth = 1usize;
        let mut i = self.at;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::Punctuation('{') => depth += 1,
                TokenKind::Punctuation('}') => {
                    depth -= 1;
                    if depth == 0 {
                        let close_start = self.tokens[i].start;
                        self.at = i + 1;
                        return Ok(self.source.text()[open_end..close_start].to_owned());
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Err("APP003 unclosed branch block".to_owned())
    }

    fn finish(&self) -> Result<(), String> {
        if let Some(token) = self.peek() {
            return Err(format!(
                "APP003 extra source after final semicolon at byte {} ('{}')",
                token.start, token.raw
            ));
        }
        Ok(())
    }
}

fn compile_app003_single_entry(
    source: &SourceText,
    requested_module: &str,
) -> Result<App003Compiled, String> {
    let mut cursor = App003Cursor::new(source)?;

    cursor.expect("module")?;
    let module = cursor.module_path()?;
    cursor.expect(";")?;
    if module != requested_module {
        return Err(format!(
            "APP003 requested entry module '{requested_module}' but source declares '{module}'"
        ));
    }

    // Narrow entry grammar: only imports, FileWrite and one key-code input.
    // No function bodies, arbitrary host effects or interpolation of syntax.
    let mut imports = Vec::new();
    while cursor.eat("import") {
        let target = cursor.module_path()?;
        cursor.expect(";")?;
        if imports.iter().any(|old| old == &target) {
            return Err(format!("APP003 duplicate import '{target}'"));
        }
        imports.push(target);
        if imports.len() > 8 {
            return Err("APP003 permits at most 8 imports".to_owned());
        }
    }
    if imports.is_empty() {
        return Err("APP003 requires at least one imported rule module".to_owned());
    }
    let mut aliases = BTreeSet::new();
    for path in &imports {
        let alias = path.rsplit('.').next().unwrap_or_default();
        if !aliases.insert(alias.to_owned()) {
            return Err("APP003 ambiguous import aliases".to_owned());
        }
    }

    cursor.expect("effect")?;
    cursor.expect("FileWrite")?;
    cursor.expect(";")?;
    cursor.expect("input")?;
    cursor.expect("key_code")?;
    cursor.expect(";")?;

    cursor.expect("entry")?;
    cursor.expect("main")?;
    cursor.expect("selects")?;
    let selector_alias = cursor.identifier()?;
    cursor.expect(".")?;
    let selector_fn = cursor.identifier()?;
    cursor.expect("(")?;
    cursor.expect("key_code")?;
    cursor.expect(")")?;
    if !aliases.contains(&selector_alias) {
        return Err(format!(
            "APP003 selector references unknown import alias '{selector_alias}'"
        ));
    }
    cursor.expect("writes")?;
    let bundle_quoted = cursor.quoted()?;
    cursor.expect("{")?;
    cursor.expect("approved")?;
    let approved_body = cursor.braced_source()?;
    cursor.expect("rejected")?;
    let rejected_body = cursor.braced_source()?;
    cursor.expect("}")?;
    cursor.expect(";")?;
    cursor.finish()?;

    // The original source contains one entire effectful declaration. The
    // bounded compiler constructs scoped immutable internal programs.
    let mut pure = format!("module {module};\n");
    for target in imports {
        pure.push_str(&format!("import {target};\n"));
    }
    pure.push_str(&format!(
        "input key_code;\nentry main returns {selector_alias}.{selector_fn}(key_code);\n"
    ));

    fn compile_arm(
        module: &str,
        bundle_quoted: &str,
        body: &str,
        source_name: &str,
        id: u32,
    ) -> Result<nordoi_kernel::P27AtomicBundlePlan, String> {
        let text = format!(
            "module {module};\neffect FileWrite;\ninput key_code;\nentry main writes {bundle_quoted} {{\n{body}\n}};\n"
        );
        let source = SourceText::new(SourceId::new(id), source_name, text)
            .map_err(|e| format!("APP003 internal arm source invalid: {e}"))?;
        compile_atomic_bundle_plan_p27(&source)
            .map_err(|e| format!("APP003 cannot compile {source_name}: {e}"))
    }

    // Compile both arms, even the one not selected by the rule. This ensures
    // failed syntax and mismatched permissions cannot be hidden in dead arms.
    let approved = compile_arm(&module, &bundle_quoted, &approved_body, "<app003-approved>", 200)?;
    let rejected = compile_arm(&module, &bundle_quoted, &rejected_body, "<app003-rejected>", 201)?;
    if approved.bundle_name() != rejected.bundle_name()
        || approved.file_count() != rejected.file_count()
        || approved.required_capabilities() != rejected.required_capabilities()
    {
        return Err(
            "APP003 both branches require identical bundle name and exact FileWrite targets"
                .to_owned(),
        );
    }

    let pure_entry = SourceText::new(
        SourceId::new(1),
        format!("<app003-pure-{module}>"),
        pure,
    )
    .map_err(|e| format!("APP003 cannot construct imported rule entry: {e}"))?;

    Ok(App003Compiled {
        entry_module: module,
        pure_entry,
        approved,
        rejected,
    })
}

fn load_app003_imported_graph(
    root: &Path,
    compiled: &App003Compiled,
) -> Result<Vec<SourceText>, u8> {
    let mut pending = BTreeSet::new();
    let mut loaded = BTreeMap::new();
    let mut next_source_id = 2u32;
    pending.insert(compiled.entry_module.clone());

    while let Some(module_name) = pending.pop_first() {
        if loaded.contains_key(&module_name) {
            continue;
        }
        if loaded.len() >= MAX_V13_MODULES {
            report_plain_error(
                "integrated-bundle-run",
                &compiled.entry_module,
                format_args!("module graph exceeds V1.3 bound {MAX_V13_MODULES}"),
            );
            return Err(EXIT_FRONTEND);
        }
        let source = if module_name == compiled.entry_module {
            compiled.pure_entry.clone()
        } else {
            let path = module_path_v13(root, &module_name);
            let (name, text) = match load_source(path.as_os_str()) {
                Ok(source) => source,
                Err(error) => {
                    report_load_error(path.as_os_str(), &error);
                    return Err(EXIT_IO);
                }
            };
            let source = SourceText::new(SourceId::new(next_source_id), name, text);
            next_source_id += 1;
            match source {
                Ok(source) => source,
                Err(error) => {
                    report_plain_error("integrated-bundle-run", &module_name, &error);
                    return Err(EXIT_FRONTEND);
                }
            }
        };

        let discovery = match discover_module_imports_v13(&source) {
            Ok(d) => d,
            Err(error) => {
                report_plain_error("integrated-bundle-run", source.name(), &error);
                return Err(EXIT_FRONTEND);
            }
        };
        if discovery.module() != module_name {
            report_plain_error(
                "integrated-bundle-run",
                &module_name,
                format_args!(
                    "module path requested '{module_name}', source declares '{}'",
                    discovery.module()
                ),
            );
            return Err(EXIT_FRONTEND);
        }
        for import in discovery.imports() {
            if !loaded.contains_key(import) {
                pending.insert(import.clone());
            }
        }
        loaded.insert(module_name, source);
    }
    Ok(loaded.into_values().collect())
}

pub(super) fn run_app003_integrated_cli(
    source_root: &Path,
    entry_module: &str,
    key_code: u32,
    grant: Option<&Path>,
) -> u8 {
    // Fail closed before source processing, module computation and any file
    // creation. The user grants a specific host directory explicitly.
    let output_dir = match grant {
        Some(dir) => dir,
        None => {
            report_plain_error(
                "integrated-bundle-run",
                entry_module,
                "explicit --grant-output-dir required; authority denied by default",
            );
            return EXIT_RUNTIME;
        }
    };

    let path = module_path_v13(source_root, entry_module);
    let (name, text) = match load_source(path.as_os_str()) {
        Ok(source) => source,
        Err(error) => {
            report_load_error(path.as_os_str(), &error);
            return EXIT_IO;
        }
    };
    let single_source = match SourceText::new(SourceId::new(1), name, text) {
        Ok(source) => source,
        Err(error) => {
            report_plain_error("integrated-bundle-run", entry_module, &error);
            return EXIT_FRONTEND;
        }
    };
    let compiled = match compile_app003_single_entry(&single_source, entry_module) {
        Ok(compiled) => compiled,
        Err(error) => {
            report_plain_error("integrated-bundle-run", entry_module, error);
            return EXIT_FRONTEND;
        }
    };
    let sources = match load_app003_imported_graph(source_root, &compiled) {
        Ok(sources) => sources,
        Err(exit) => return exit,
    };

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
            report_plain_error("integrated-bundle-run", entry_module, &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else {
                EXIT_RUNTIME
            };
        }
    };
    let (selected, branch_name) = match rule.result() {
        nordoi_kernel::DynamicValue::Int(1) => (&compiled.approved, "approved"),
        nordoi_kernel::DynamicValue::Int(0) => (&compiled.rejected, "rejected"),
        other => {
            report_plain_error(
                "integrated-bundle-run",
                entry_module,
                format_args!(
                    "rule result must be exactly INT(0) or INT(1); got {} (no publication)",
                    other.as_text()
                ),
            );
            return EXIT_RUNTIME;
        }
    };

    // True P2.7 engine decides the target authorization. The selector is
    // pure: no FileWrite grant exists during V1.3 execution.
    let capabilities = selected.required_capabilities();
    let mut authority = CapabilitySet::new();
    for capability in &capabilities {
        authority.allow(capability.clone());
    }
    let receipt = match execute_atomic_bundle_p27(selected, key_code, &authority, output_dir) {
        Ok(receipt) => receipt,
        Err(error) => {
            report_plain_error("integrated-bundle-run", entry_module, &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else if error.is_materialization_failure() {
                EXIT_IO
            } else {
                EXIT_RUNTIME
            };
        }
    };

    // Authentic original receipts, not an invented combined certification.
    let real_receipt = receipt.render_text();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    if let Err(error) = err.write_all(real_receipt.as_bytes()).and_then(|_| err.flush()) {
        report_io_error("<stderr>", &error);
        return EXIT_IO;
    }
    let summary = format!(
        "integrated-bundle-run entry-module=\"{}\" source-profile=APP003_SINGLE_NOI_IMPORTS_AND_EFFECTS modules={} original-key-code={} rule-result={} selected=\"{}\" bundle=\"{}\" files={} bridge=V13_PURE_RULE_TO_P27_EFFECT status=COMMITTED authority=EXPLICIT grant-count={} ambient-authority=NONE rule-receipt-hex={} bundle-receipt-sha256={} crash-durability=UNCLAIMED\n",
        entry_module,
        rule.plan().module_count(),
        key_code,
        rule.result().as_text(),
        branch_name,
        receipt.bundle_name(),
        receipt.files().len(),
        capabilities.len(),
        hex_bytes(rule.canonical_v13_receipt_bytes()),
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
