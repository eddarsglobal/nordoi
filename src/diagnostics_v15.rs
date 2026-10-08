use std::collections::{BTreeMap, BTreeSet};

use crate::frontend::{SourceSpan, SourceText};
use crate::modules_v13::{discover_module_imports_v13, ModuleGraphError};
use crate::project_v14::{compile_project_v14, ProjectBuildError, V14ProjectManifest};

pub const V15_DIAGNOSTIC_SCHEMA: &str = "nordoi.diagnostic.v1";
pub const NDX_MANIFEST: &str = "NDX1001";
pub const NDX_SOURCE_IO: &str = "NDX2001";
pub const NDX_SOURCE: &str = "NDX2002";
pub const NDX_MODULE_DECLARATION: &str = "NDX2003";
pub const NDX_MISSING_IMPORT: &str = "NDX2004";
pub const NDX_IMPORT_CYCLE: &str = "NDX2005";
pub const NDX_MODULE_GRAPH: &str = "NDX2101";
pub const NDX_SEMANTIC: &str = "NDX2201";
pub const NDX_LOWERING: &str = "NDX3001";
pub const NDX_PACKAGE: &str = "NDX4001";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V15DiagnosticLocation {
    file: String,
    line: u32,
    column: u32,
    excerpt: String,
    caret_column: u32,
    caret_len: u32,
}

impl V15DiagnosticLocation {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub const fn line(&self) -> u32 {
        self.line
    }

    pub const fn column(&self) -> u32 {
        self.column
    }

    pub fn excerpt(&self) -> &str {
        &self.excerpt
    }

    pub const fn caret_column(&self) -> u32 {
        self.caret_column
    }

    pub const fn caret_len(&self) -> u32 {
        self.caret_len
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V15Diagnostic {
    code: String,
    stage: String,
    message: String,
    location: Option<V15DiagnosticLocation>,
    import_trace: Vec<String>,
}

impl V15Diagnostic {
    pub fn new(
        code: impl Into<String>,
        stage: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            stage: stage.into(),
            message: message.into(),
            location: None,
            import_trace: Vec::new(),
        }
    }

    pub fn with_location(mut self, source: &SourceText, span: SourceSpan) -> Self {
        if span.source() != source.id() {
            return self;
        }
        let Ok(start) = source.position(span.start()) else {
            return self;
        };
        let excerpt = source
            .text()
            .lines()
            .nth(start.line().saturating_sub(1) as usize)
            .unwrap_or("")
            .to_owned();
        let caret_len = if let Ok(end) = source.position(span.end()) {
            if end.line() == start.line() {
                end.column().saturating_sub(start.column()).max(1)
            } else {
                1
            }
        } else {
            1
        };
        self.location = Some(V15DiagnosticLocation {
            file: source.name().to_owned(),
            line: start.line(),
            column: start.column(),
            excerpt,
            caret_column: start.column(),
            caret_len,
        });
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.location = Some(V15DiagnosticLocation {
            file: file.into(),
            line: 0,
            column: 0,
            excerpt: String::new(),
            caret_column: 0,
            caret_len: 0,
        });
        self
    }

    pub fn with_import_trace(mut self, trace: Vec<String>) -> Self {
        self.import_trace = canonical_trace(trace);
        self
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn stage(&self) -> &str {
        &self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn location(&self) -> Option<&V15DiagnosticLocation> {
        self.location.as_ref()
    }

    pub fn import_trace(&self) -> &[String] {
        &self.import_trace
    }

    pub fn render_text(&self) -> String {
        let mut output = format!("error[{}] {}: {}\n", self.code, self.stage, self.message);
        if let Some(location) = &self.location {
            if location.line > 0 {
                output.push_str(&format!(
                    " --> {}:{}:{}\n",
                    location.file, location.line, location.column
                ));
                output.push_str("  |\n");
                output.push_str(&format!("{} | {}\n", location.line, location.excerpt));
                let padding = " ".repeat(location.caret_column.saturating_sub(1) as usize);
                let carets = "^".repeat(location.caret_len.max(1) as usize);
                output.push_str(&format!("  | {padding}{carets}\n"));
            } else {
                output.push_str(&format!(" --> {}\n", location.file));
            }
        }
        if !self.import_trace.is_empty() {
            output.push_str(&format!(
                "  = import-trace: {}\n",
                self.import_trace.join(" -> ")
            ));
        }
        output
    }

    pub fn render_json(&self) -> String {
        let location = match &self.location {
            Some(location) => format!(
                "{{\"file\":\"{}\",\"line\":{},\"column\":{},\"excerpt\":\"{}\"}}",
                json_escape(&location.file),
                location.line,
                location.column,
                json_escape(&location.excerpt)
            ),
            None => "null".to_owned(),
        };
        let trace = self
            .import_trace
            .iter()
            .map(|part| format!("\"{}\"", json_escape(part)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"{V15_DIAGNOSTIC_SCHEMA}\",\"status\":\"error\",\"code\":\"{}\",\"stage\":\"{}\",\"message\":\"{}\",\"location\":{},\"importTrace\":[{}]}}",
            json_escape(&self.code),
            json_escape(&self.stage),
            json_escape(&self.message),
            location,
            trace
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V15ProjectCheckReport {
    project: String,
    version: String,
    entry_module: String,
    module_count: usize,
    import_count: usize,
    nair_format_minor: u16,
    nair_instruction_count: usize,
}

impl V15ProjectCheckReport {
    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn entry_module(&self) -> &str {
        &self.entry_module
    }

    pub const fn module_count(&self) -> usize {
        self.module_count
    }

    pub const fn import_count(&self) -> usize {
        self.import_count
    }

    pub const fn nair_format_minor(&self) -> u16 {
        self.nair_format_minor
    }

    pub const fn nair_instruction_count(&self) -> usize {
        self.nair_instruction_count
    }

    pub fn render_text(&self) -> String {
        format!(
            "check project=\"{}\" version=\"{}\" entry-module=\"{}\" modules={} imports={} nair-minor=0.{} nair-instructions={} diagnostics=0 status=PASS dependency-network=NONE runtime-fs=NONE authority=NONE\n",
            self.project,
            self.version,
            self.entry_module,
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{V15_DIAGNOSTIC_SCHEMA}\",\"status\":\"pass\",\"project\":\"{}\",\"version\":\"{}\",\"entryModule\":\"{}\",\"modules\":{},\"imports\":{},\"nairMinor\":{},\"nairInstructions\":{},\"diagnostics\":[]}}",
            json_escape(&self.project),
            json_escape(&self.version),
            json_escape(&self.entry_module),
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count
        )
    }
}

pub fn check_project_sources_v15(
    manifest: &V14ProjectManifest,
    sources: &[SourceText],
) -> Result<V15ProjectCheckReport, Box<V15Diagnostic>> {
    let graph = discover_import_graph_v15(sources)?;
    if let Some(cycle) = detect_import_cycle_v15(&graph) {
        return Err(Box::new(
            V15Diagnostic::new(
                NDX_IMPORT_CYCLE,
                "module-graph",
                format!("recursive or cyclic import graph: {}", cycle.join(" -> ")),
            )
            .with_import_trace(cycle),
        ));
    }
    let build = compile_project_v14(manifest, sources).map_err(|error| {
        Box::new(diagnostic_from_project_error_v15(
            &error,
            sources,
            Vec::new(),
        ))
    })?;
    Ok(V15ProjectCheckReport {
        project: manifest.name().to_owned(),
        version: manifest.version().to_owned(),
        entry_module: manifest.entry_module().to_owned(),
        module_count: build.module_plan().module_count(),
        import_count: build.module_plan().import_count(),
        nair_format_minor: build.lowering().nair_format_minor(),
        nair_instruction_count: build.lowering().nair_instruction_count(),
    })
}

pub fn diagnostic_from_module_error_v15(
    error: &ModuleGraphError,
    sources: &[SourceText],
    trace: Vec<String>,
) -> V15Diagnostic {
    let (code, stage) = match error {
        ModuleGraphError::Lex(_) | ModuleGraphError::Source(_) => (NDX_SOURCE, "source"),
        ModuleGraphError::Module(_) => (NDX_MODULE_DECLARATION, "module"),
        ModuleGraphError::Nested(_) => (NDX_SEMANTIC, "semantic"),
        ModuleGraphError::Semantic { .. } => (NDX_MODULE_GRAPH, "module-graph"),
    };
    let mut diagnostic =
        V15Diagnostic::new(code, stage, error.to_string()).with_import_trace(trace);
    if let Some(span) = error.primary_span() {
        if let Some(source) = sources.iter().find(|source| source.id() == span.source()) {
            diagnostic = diagnostic.with_location(source, span);
        }
    }
    diagnostic
}

pub fn diagnostic_from_project_error_v15(
    error: &ProjectBuildError,
    sources: &[SourceText],
    trace: Vec<String>,
) -> V15Diagnostic {
    match error {
        ProjectBuildError::Manifest { .. } => {
            V15Diagnostic::new(NDX_MANIFEST, "manifest", error.to_string()).with_import_trace(trace)
        }
        ProjectBuildError::Module(module) => {
            diagnostic_from_module_error_v15(module, sources, trace)
        }
        ProjectBuildError::Nair(_) => {
            V15Diagnostic::new(NDX_LOWERING, "lowering", error.to_string()).with_import_trace(trace)
        }
        ProjectBuildError::Package { .. } => {
            V15Diagnostic::new(NDX_PACKAGE, "package", error.to_string()).with_import_trace(trace)
        }
    }
}

pub fn discover_import_graph_v15(
    sources: &[SourceText],
) -> Result<BTreeMap<String, Vec<String>>, Box<V15Diagnostic>> {
    let mut graph = BTreeMap::new();
    for source in sources {
        let discovery = discover_module_imports_v13(source).map_err(|error| {
            Box::new(diagnostic_from_module_error_v15(
                &error,
                sources,
                Vec::new(),
            ))
        })?;
        graph.insert(discovery.module().to_owned(), discovery.imports().to_vec());
    }
    Ok(graph)
}

pub fn detect_import_cycle_v15(graph: &BTreeMap<String, Vec<String>>) -> Option<Vec<String>> {
    fn visit(
        node: &str,
        graph: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited.contains(node) {
            return None;
        }
        if visiting.contains(node) {
            let start = stack.iter().position(|item| item == node).unwrap_or(0);
            let mut cycle = stack[start..].to_vec();
            cycle.push(node.to_owned());
            return Some(cycle);
        }
        visiting.insert(node.to_owned());
        stack.push(node.to_owned());
        if let Some(imports) = graph.get(node) {
            for import in imports {
                if graph.contains_key(import) {
                    if let Some(cycle) = visit(import, graph, visiting, visited, stack) {
                        return Some(cycle);
                    }
                }
            }
        }
        stack.pop();
        visiting.remove(node);
        visited.insert(node.to_owned());
        None
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();
    for module in graph.keys() {
        if let Some(cycle) = visit(module, graph, &mut visiting, &mut visited, &mut stack) {
            return Some(cycle);
        }
    }
    None
}

pub fn import_trace_from_parents_v15(
    parents: &BTreeMap<String, String>,
    module: &str,
) -> Vec<String> {
    let mut trace = vec![module.to_owned()];
    let mut current = module;
    let mut guard = BTreeSet::new();
    guard.insert(current.to_owned());
    while let Some(parent) = parents.get(current) {
        if !guard.insert(parent.clone()) {
            break;
        }
        trace.push(parent.clone());
        current = parent;
    }
    trace.reverse();
    trace
}

fn canonical_trace(trace: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in trace {
        if output.last() != Some(&item) {
            output.push(item);
        }
    }
    output
}

fn json_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}
