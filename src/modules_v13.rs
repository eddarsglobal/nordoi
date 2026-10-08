use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::frontend::{
    analyze_module_unit, lex, LexError, ModuleError, SourceError, SourceId, SourceText, Token,
    TokenKind,
};
use crate::input::InputBatch;
use crate::nested_control_v12::{
    execute_nested_function_control_source_v12, lower_nested_function_control_plan_v12,
    NestedFunctionControlError, V12NestedControlExecutionReport, V12NestedControlNairArtifact,
    V12NestedControlPlan,
};

const V13_WITNESS_DOMAIN: &[u8] = b"NORDOI-V1.3-REAL-MODULES-IMPORTS\0";
const V13_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V1.3-REAL-MODULES-IMPORTS-RECEIPT\0";

pub const MAX_V13_MODULES: usize = 64;
pub const MAX_V13_IMPORTS_PER_MODULE: usize = 32;
pub const MAX_V13_TOTAL_IMPORT_EDGES: usize = 256;

#[derive(Debug)]
pub enum ModuleGraphError {
    Lex(LexError),
    Module(ModuleError),
    Source(SourceError),
    Nested(NestedFunctionControlError),
    Semantic { message: String },
}

impl ModuleGraphError {
    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::Nested(error) if !error.is_frontend_failure())
    }

    pub fn primary_span(&self) -> Option<crate::frontend::SourceSpan> {
        match self {
            Self::Lex(error) => error.span(),
            Self::Module(error) => error.primary_span(),
            Self::Nested(error) => error.primary_span(),
            Self::Source(_) | Self::Semantic { .. } => None,
        }
    }
}

impl Display for ModuleGraphError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(error) => Display::fmt(error, f),
            Self::Module(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Nested(error) => Display::fmt(error, f),
            Self::Semantic { message } => write!(f, "V1.3 module graph error: {message}"),
        }
    }
}

impl Error for ModuleGraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lex(error) => Some(error),
            Self::Module(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Nested(error) => Some(error),
            Self::Semantic { .. } => None,
        }
    }
}

impl From<LexError> for ModuleGraphError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<ModuleError> for ModuleGraphError {
    fn from(value: ModuleError) -> Self {
        Self::Module(value)
    }
}

impl From<SourceError> for ModuleGraphError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NestedFunctionControlError> for ModuleGraphError {
    fn from(value: NestedFunctionControlError) -> Self {
        Self::Nested(value)
    }
}

pub type ModuleGraphResult<T> = Result<T, ModuleGraphError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V13ModuleDiscovery {
    module: String,
    imports: Vec<String>,
}

impl V13ModuleDiscovery {
    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn imports(&self) -> &[String] {
        &self.imports
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V13ModuleGraphPlan {
    entry_module: String,
    module_order: Vec<String>,
    import_edges: Vec<(String, String)>,
    combined_source: SourceText,
    inner: V12NestedControlPlan,
    witness: Vec<u8>,
}

impl V13ModuleGraphPlan {
    pub fn entry_module(&self) -> &str {
        &self.entry_module
    }

    pub fn module_order(&self) -> &[String] {
        &self.module_order
    }

    pub fn import_edges(&self) -> &[(String, String)] {
        &self.import_edges
    }

    pub fn module_count(&self) -> usize {
        self.module_order.len()
    }

    pub fn import_count(&self) -> usize {
        self.import_edges.len()
    }

    pub fn combined_source(&self) -> &SourceText {
        &self.combined_source
    }

    pub fn inner_plan(&self) -> &V12NestedControlPlan {
        &self.inner
    }

    pub fn canonical_v13_witness_bytes(&self) -> &[u8] {
        &self.witness
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V13ModuleGraphNairArtifact {
    inner: V12NestedControlNairArtifact,
    witness: Vec<u8>,
}

impl V13ModuleGraphNairArtifact {
    pub fn inner(&self) -> &V12NestedControlNairArtifact {
        &self.inner
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.inner.nair_instruction_count()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.inner.nair_format_minor()
    }

    pub fn runtime_call_bound(&self) -> usize {
        self.inner.runtime_call_count()
    }

    pub fn canonical_v13_lowering_witness_bytes(&self) -> &[u8] {
        &self.witness
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V13ModuleGraphExecutionReport {
    plan: V13ModuleGraphPlan,
    inner: V12NestedControlExecutionReport,
    receipt: Vec<u8>,
}

impl V13ModuleGraphExecutionReport {
    pub fn plan(&self) -> &V13ModuleGraphPlan {
        &self.plan
    }

    pub fn inner(&self) -> &V12NestedControlExecutionReport {
        &self.inner
    }

    pub fn result(&self) -> crate::dynamic_input_v07::DynamicValue {
        self.inner.result()
    }

    pub fn runtime_calls(&self) -> usize {
        self.inner.runtime_calls()
    }

    pub fn runtime_branches(&self) -> usize {
        self.inner.runtime_branches()
    }

    pub fn call_body_instructions(&self) -> usize {
        self.inner.call_body_instructions()
    }

    pub fn max_call_depth(&self) -> usize {
        self.inner.max_call_depth()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.inner.lowering().nair_format_minor()
    }

    pub fn canonical_v13_receipt_bytes(&self) -> &[u8] {
        &self.receipt
    }
}

#[derive(Debug, Clone)]
struct SignificantToken {
    token: Token,
    text: String,
}

#[derive(Debug, Clone)]
struct ParsedModule {
    name: String,
    tokens: Vec<SignificantToken>,
    skip_ranges: Vec<(usize, usize)>,
    prelude_ranges: Vec<(usize, usize)>,
    imports: BTreeMap<String, String>,
    functions: BTreeSet<String>,
    constants: BTreeSet<String>,
    has_input: bool,
    has_entry: bool,
}

pub fn discover_module_imports_v13(source: &SourceText) -> ModuleGraphResult<V13ModuleDiscovery> {
    let parsed = parse_module(source.clone())?;
    Ok(V13ModuleDiscovery {
        module: parsed.name,
        imports: parsed.imports.into_values().collect(),
    })
}

pub fn compile_module_graph_v13(
    sources: &[SourceText],
    entry_module: &str,
) -> ModuleGraphResult<V13ModuleGraphPlan> {
    validate_module_name_v13(entry_module)?;
    if sources.is_empty() {
        return Err(semantic("module graph is empty"));
    }
    if sources.len() > MAX_V13_MODULES {
        return Err(semantic(format!(
            "module graph has {} modules, exceeding the certified bound {MAX_V13_MODULES}",
            sources.len()
        )));
    }

    let mut modules = BTreeMap::new();
    for source in sources {
        let parsed = parse_module(source.clone())?;
        if modules.insert(parsed.name.clone(), parsed).is_some() {
            return Err(semantic("duplicate canonical module identity"));
        }
    }

    if !modules.contains_key(entry_module) {
        return Err(semantic(format!(
            "entry module '{entry_module}' is missing"
        )));
    }

    for module in modules.values() {
        if module.imports.len() > MAX_V13_IMPORTS_PER_MODULE {
            return Err(semantic(format!(
                "module '{}' has {} imports, exceeding the certified bound {MAX_V13_IMPORTS_PER_MODULE}",
                module.name,
                module.imports.len()
            )));
        }
    }

    let mut reachable = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    visit_module(entry_module, &modules, &mut visiting, &mut reachable)?;

    let module_order = reachable.iter().cloned().collect::<Vec<_>>();
    let mut import_edges = Vec::new();
    for name in &module_order {
        let module = modules.get(name).expect("reachable module must exist");
        for target in module.imports.values() {
            if reachable.contains(target) {
                import_edges.push((name.clone(), target.clone()));
            }
        }
    }
    if import_edges.len() > MAX_V13_TOTAL_IMPORT_EDGES {
        return Err(semantic(format!(
            "module graph has {} import edges, exceeding the certified bound {MAX_V13_TOTAL_IMPORT_EDGES}",
            import_edges.len()
        )));
    }

    for name in &module_order {
        let module = modules.get(name).expect("reachable module must exist");
        if name != entry_module && (module.has_input || module.has_entry) {
            return Err(semantic(format!(
                "imported module '{}' must be library-only in V1.3 and cannot declare input or entry",
                module.name
            )));
        }
        if name != entry_module && !module.constants.is_empty() {
            return Err(semantic(format!(
                "imported module '{}' cannot declare const in the V1.3 vertical slice",
                module.name
            )));
        }
        if name != entry_module && !module.prelude_ranges.is_empty() {
            return Err(semantic(format!(
                "imported module '{}' cannot declare type/effect prelude items in the V1.3 vertical slice",
                module.name
            )));
        }
    }

    let entry = modules
        .get(entry_module)
        .expect("entry module existence checked");
    if !entry.has_entry {
        return Err(semantic(format!(
            "entry module '{entry_module}' must declare exactly one entry"
        )));
    }

    let mut prefixes = BTreeMap::new();
    for (index, name) in module_order.iter().enumerate() {
        prefixes.insert(name.clone(), format!("v13m{index}_"));
    }

    let mut combined = format!("module {entry_module};\n");
    combined.push_str(&render_entry_prelude(entry));
    combined.push('\n');
    for name in &module_order {
        let module = modules.get(name).expect("reachable module must exist");
        let rewritten = rewrite_module(module, &modules, &prefixes)?;
        combined.push_str(&rewritten);
        combined.push('\n');
    }

    let combined_source = SourceText::new(SourceId::new(0x13), "<v1.3-module-graph>", combined)?;
    let inner =
        crate::nested_control_v12::compile_nested_function_control_plan_v12(&combined_source)?;

    let mut witness = Vec::new();
    witness.extend_from_slice(V13_WITNESS_DOMAIN);
    encode_string(entry_module, &mut witness);
    witness.extend_from_slice(&(module_order.len() as u32).to_be_bytes());
    for name in &module_order {
        encode_string(name, &mut witness);
    }
    witness.extend_from_slice(&(import_edges.len() as u32).to_be_bytes());
    for (from, to) in &import_edges {
        encode_string(from, &mut witness);
        encode_string(to, &mut witness);
    }
    push_component(&mut witness, inner.canonical_v12_semantic_bytes());

    Ok(V13ModuleGraphPlan {
        entry_module: entry_module.to_owned(),
        module_order,
        import_edges,
        combined_source,
        inner,
        witness,
    })
}

pub fn lower_module_graph_v13(
    plan: &V13ModuleGraphPlan,
) -> ModuleGraphResult<V13ModuleGraphNairArtifact> {
    let inner = lower_nested_function_control_plan_v12(plan.inner_plan())?;
    let mut witness = Vec::new();
    witness.extend_from_slice(V13_WITNESS_DOMAIN);
    push_component(&mut witness, plan.canonical_v13_witness_bytes());
    push_component(&mut witness, inner.canonical_v12_witness_bytes());
    Ok(V13ModuleGraphNairArtifact { inner, witness })
}

pub fn execute_module_graph_v13(
    sources: &[SourceText],
    entry_module: &str,
    input: &InputBatch,
) -> ModuleGraphResult<V13ModuleGraphExecutionReport> {
    let plan = compile_module_graph_v13(sources, entry_module)?;
    let inner = execute_nested_function_control_source_v12(plan.combined_source(), input)?;

    let mut receipt = Vec::new();
    receipt.extend_from_slice(V13_RECEIPT_DOMAIN);
    push_component(&mut receipt, plan.canonical_v13_witness_bytes());
    push_component(&mut receipt, inner.canonical_v12_receipt_bytes());

    Ok(V13ModuleGraphExecutionReport {
        plan,
        inner,
        receipt,
    })
}

fn parse_module(source: SourceText) -> ModuleGraphResult<ParsedModule> {
    let unit = analyze_module_unit(&source)?;
    let declaration = unit.module().ok_or_else(|| {
        semantic(format!(
            "source '{}' requires an explicit module declaration",
            source.name()
        ))
    })?;
    let name = declaration.path().canonical_text(&source)?;
    validate_module_name_v13(&name)?;

    let raw_tokens = lex(&source)?;
    let mut tokens = Vec::new();
    for token in raw_tokens {
        if token.kind() == &TokenKind::Eof || token.is_trivia() {
            continue;
        }
        tokens.push(SignificantToken {
            text: source.slice(token.span())?.to_owned(),
            token,
        });
    }

    let mut skip_ranges = Vec::new();
    let mut imports = BTreeMap::new();
    let mut prelude_ranges = Vec::new();
    let mut functions = BTreeSet::new();
    let mut constants = BTreeSet::new();
    let mut has_input = false;
    let mut has_entry = false;
    let mut index = 0usize;

    while index < tokens.len() {
        let text = tokens[index].text.as_str();
        match text {
            "module" => {
                let end = declaration_end(&tokens, index, "module")?;
                skip_ranges.push((index, end));
                index = end + 1;
            }
            "import" => {
                let end = declaration_end(&tokens, index, "import")?;
                let path = parse_path_tokens(&tokens[index + 1..end])?;
                validate_module_name_v13(&path)?;
                let alias = path
                    .rsplit('.')
                    .next()
                    .expect("validated module path has one segment")
                    .to_owned();
                if let Some(existing) = imports.insert(alias.clone(), path.clone()) {
                    return Err(semantic(format!(
                        "module '{name}' has ambiguous import alias '{alias}' for '{existing}' and '{path}'"
                    )));
                }
                skip_ranges.push((index, end));
                index = end + 1;
            }
            "type" | "effect" => {
                let end = statement_end(&tokens, index, text)?;
                prelude_ranges.push((index, end));
                skip_ranges.push((index, end));
                index = end + 1;
            }
            "fn" => {
                let function = tokens.get(index + 1).ok_or_else(|| {
                    semantic(format!("module '{name}' has incomplete fn declaration"))
                })?;
                if function.token.kind() != &TokenKind::Identifier {
                    return Err(semantic(format!(
                        "module '{name}' has invalid function name"
                    )));
                }
                if !functions.insert(function.text.clone()) {
                    return Err(semantic(format!(
                        "module '{name}' declares duplicate function '{}'",
                        function.text
                    )));
                }
                index = statement_end(&tokens, index, "fn")? + 1;
            }
            "input" => {
                if has_input {
                    return Err(semantic(format!(
                        "module '{name}' declares more than one input"
                    )));
                }
                has_input = true;
                index = statement_end(&tokens, index, "input")? + 1;
            }
            "entry" => {
                if has_entry {
                    return Err(semantic(format!(
                        "module '{name}' declares more than one entry"
                    )));
                }
                has_entry = true;
                index = statement_end(&tokens, index, "entry")? + 1;
            }
            "const" => {
                let constant = tokens.get(index + 1).ok_or_else(|| {
                    semantic(format!("module '{name}' has incomplete const declaration"))
                })?;
                if constant.token.kind() != &TokenKind::Identifier {
                    return Err(semantic(format!(
                        "module '{name}' has invalid constant name"
                    )));
                }
                if !constants.insert(constant.text.clone()) {
                    return Err(semantic(format!(
                        "module '{name}' declares duplicate constant '{}'",
                        constant.text
                    )));
                }
                index = statement_end(&tokens, index, "const")? + 1;
            }
            _ => {
                return Err(semantic(format!(
                    "module '{name}' has unsupported top-level declaration beginning with '{text}'"
                )));
            }
        }
    }

    Ok(ParsedModule {
        name,
        tokens,
        skip_ranges,
        prelude_ranges,
        imports,
        functions,
        constants,
        has_input,
        has_entry,
    })
}

fn visit_module(
    name: &str,
    modules: &BTreeMap<String, ParsedModule>,
    visiting: &mut BTreeSet<String>,
    reachable: &mut BTreeSet<String>,
) -> ModuleGraphResult<()> {
    if reachable.contains(name) {
        return Ok(());
    }
    if !visiting.insert(name.to_owned()) {
        return Err(semantic(format!(
            "recursive or cyclic import graph reaches module '{name}'"
        )));
    }
    let module = modules
        .get(name)
        .ok_or_else(|| semantic(format!("module '{name}' is missing")))?;
    for target in module.imports.values() {
        if !modules.contains_key(target) {
            return Err(semantic(format!(
                "module '{}' imports missing module '{target}'",
                module.name
            )));
        }
        visit_module(target, modules, visiting, reachable)?;
    }
    visiting.remove(name);
    reachable.insert(name.to_owned());
    Ok(())
}

fn render_entry_prelude(module: &ParsedModule) -> String {
    let mut output = String::new();
    let mut previous_word = false;
    for (start, end) in &module.prelude_ranges {
        for token in &module.tokens[*start..=*end] {
            let is_word = matches!(
                token.token.kind(),
                TokenKind::Identifier | TokenKind::NumericCandidate | TokenKind::QuotedText
            );
            append_token(&mut output, &token.text, is_word, &mut previous_word);
        }
        output.push('\n');
        previous_word = false;
    }
    output
}

fn rewrite_module(
    module: &ParsedModule,
    modules: &BTreeMap<String, ParsedModule>,
    prefixes: &BTreeMap<String, String>,
) -> ModuleGraphResult<String> {
    let own_prefix = prefixes
        .get(&module.name)
        .expect("reachable module prefix must exist");
    let mut output = String::new();
    let mut previous_word = false;
    let mut index = 0usize;

    while index < module.tokens.len() {
        if let Some(end) = skipped_end(index, &module.skip_ranges) {
            index = end + 1;
            continue;
        }

        if index + 3 < module.tokens.len()
            && module.tokens[index].token.kind() == &TokenKind::Identifier
            && module.tokens[index + 1].text == "."
            && module.tokens[index + 2].token.kind() == &TokenKind::Identifier
            && module.tokens[index + 3].text == "("
        {
            let alias = &module.tokens[index].text;
            let function = &module.tokens[index + 2].text;
            let target = module.imports.get(alias).ok_or_else(|| {
                semantic(format!(
                    "module '{}' references unknown import alias '{alias}'",
                    module.name
                ))
            })?;
            let target_module = modules.get(target).ok_or_else(|| {
                semantic(format!(
                    "module '{}' imports missing module '{target}'",
                    module.name
                ))
            })?;
            if !target_module.functions.contains(function) {
                return Err(semantic(format!(
                    "module '{}' imports '{}' but function '{}.{}' does not exist",
                    module.name, target, alias, function
                )));
            }
            let prefix = prefixes.get(target).ok_or_else(|| {
                semantic(format!("module '{target}' is not reachable from entry"))
            })?;
            append_token(
                &mut output,
                &format!("{prefix}{function}"),
                true,
                &mut previous_word,
            );
            index += 3;
            continue;
        }

        let token = &module.tokens[index];
        let next_is_call = module
            .tokens
            .get(index + 1)
            .is_some_and(|next| next.text == "(");
        if token.token.kind() == &TokenKind::Identifier
            && ((next_is_call && module.functions.contains(&token.text))
                || module.constants.contains(&token.text))
        {
            append_token(
                &mut output,
                &format!("{own_prefix}{}", token.text),
                true,
                &mut previous_word,
            );
        } else {
            let is_word = matches!(
                token.token.kind(),
                TokenKind::Identifier | TokenKind::NumericCandidate | TokenKind::QuotedText
            );
            append_token(&mut output, &token.text, is_word, &mut previous_word);
        }
        index += 1;
    }

    Ok(output)
}

fn append_token(output: &mut String, text: &str, is_word: bool, previous_word: &mut bool) {
    if *previous_word && is_word {
        output.push(' ');
    }
    output.push_str(text);
    *previous_word = is_word;
}

fn skipped_end(index: usize, ranges: &[(usize, usize)]) -> Option<usize> {
    ranges
        .iter()
        .find_map(|(start, end)| (*start == index).then_some(*end))
}

fn declaration_end(
    tokens: &[SignificantToken],
    start: usize,
    kind: &str,
) -> ModuleGraphResult<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start + 1) {
        if token.text == ";" {
            return Ok(index);
        }
        if matches!(token.text.as_str(), "{" | "}" | "(" | ")" | "[" | "]") {
            return Err(semantic(format!("invalid {kind} declaration")));
        }
    }
    Err(semantic(format!("missing ';' after {kind} declaration")))
}

fn statement_end(
    tokens: &[SignificantToken],
    start: usize,
    kind: &str,
) -> ModuleGraphResult<usize> {
    let mut stack = Vec::new();
    for (index, token) in tokens.iter().enumerate().skip(start + 1) {
        match token.text.as_str() {
            "(" | "{" | "[" => stack.push(token.text.as_str()),
            ")" => {
                if stack.pop() != Some("(") {
                    return Err(semantic(format!(
                        "unbalanced delimiters in {kind} declaration"
                    )));
                }
            }
            "}" => {
                if stack.pop() != Some("{") {
                    return Err(semantic(format!(
                        "unbalanced delimiters in {kind} declaration"
                    )));
                }
            }
            "]" => {
                if stack.pop() != Some("[") {
                    return Err(semantic(format!(
                        "unbalanced delimiters in {kind} declaration"
                    )));
                }
            }
            ";" if stack.is_empty() => return Ok(index),
            _ => {}
        }
    }
    Err(semantic(format!("missing ';' after {kind} declaration")))
}

fn parse_path_tokens(tokens: &[SignificantToken]) -> ModuleGraphResult<String> {
    if tokens.is_empty() {
        return Err(semantic("import requires a module path"));
    }
    let mut output = String::new();
    let mut expect_name = true;
    for token in tokens {
        if expect_name {
            if token.token.kind() != &TokenKind::Identifier {
                return Err(semantic("import path requires identifier segments"));
            }
            if !output.is_empty() {
                output.push('.');
            }
            output.push_str(&token.text);
        } else if token.text != "." {
            return Err(semantic("import path requires '.' between segments"));
        }
        expect_name = !expect_name;
    }
    if expect_name {
        return Err(semantic("import path cannot end with '.'"));
    }
    Ok(output)
}

pub fn validate_module_name_v13(name: &str) -> ModuleGraphResult<()> {
    if name.is_empty() {
        return Err(semantic("module name cannot be empty"));
    }
    for segment in name.split('.') {
        if segment.is_empty()
            || !segment.bytes().enumerate().all(|(index, byte)| {
                if index == 0 {
                    byte.is_ascii_alphabetic() || byte == b'_'
                } else {
                    byte.is_ascii_alphanumeric() || byte == b'_'
                }
            })
        {
            return Err(semantic(format!("invalid canonical module path '{name}'")));
        }
    }
    Ok(())
}

fn semantic(message: impl Into<String>) -> ModuleGraphError {
    ModuleGraphError::Semantic {
        message: message.into(),
    }
}

fn encode_string(value: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&(value.len() as u32).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn push_component(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}
