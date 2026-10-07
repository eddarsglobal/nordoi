use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::frontend::SourceText;
use crate::modules_v13::{
    compile_module_graph_v13, lower_module_graph_v13, validate_module_name_v13, ModuleGraphError,
    V13ModuleGraphNairArtifact, V13ModuleGraphPlan, MAX_V13_MODULES, MAX_V13_TOTAL_IMPORT_EDGES,
};
use crate::nair::{NairError, NairProgram};

const V14_BUILD_WITNESS_DOMAIN: &[u8] = b"NORDOI-V1.4-PROJECT-BUILD-PACKAGE\0";
pub const V14_PACKAGE_MAGIC: [u8; 4] = *b"NPKG";
pub const V14_PACKAGE_MAJOR: u16 = 1;
pub const V14_PACKAGE_MINOR: u16 = 0;
pub const V14_MANIFEST_FILE: &str = "NORDOI.toml";
pub const V14_LOCK_FILE: &str = "NORDOI.lock";
pub const MAX_V14_MANIFEST_BYTES: usize = 16 * 1024;
pub const MAX_V14_PACKAGE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_V14_PROJECT_NAME_BYTES: usize = 64;
pub const MAX_V14_VERSION_BYTES: usize = 64;
pub const MAX_V14_SOURCE_ROOT_BYTES: usize = 256;

#[derive(Debug)]
pub enum ProjectBuildError {
    Manifest { message: String },
    Module(ModuleGraphError),
    Nair(NairError),
    Package { message: String },
}

impl ProjectBuildError {
    pub fn is_frontend_failure(&self) -> bool {
        match self {
            Self::Manifest { .. } | Self::Package { .. } => true,
            Self::Module(error) => error.is_frontend_failure(),
            Self::Nair(_) => false,
        }
    }
}

impl Display for ProjectBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest { message } => write!(f, "V1.4 project manifest error: {message}"),
            Self::Module(error) => Display::fmt(error, f),
            Self::Nair(error) => Display::fmt(error, f),
            Self::Package { message } => write!(f, "V1.4 project package error: {message}"),
        }
    }
}

impl Error for ProjectBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Module(error) => Some(error),
            Self::Nair(error) => Some(error),
            Self::Manifest { .. } | Self::Package { .. } => None,
        }
    }
}

impl From<ModuleGraphError> for ProjectBuildError {
    fn from(value: ModuleGraphError) -> Self {
        Self::Module(value)
    }
}

impl From<NairError> for ProjectBuildError {
    fn from(value: NairError) -> Self {
        Self::Nair(value)
    }
}

pub type ProjectBuildResult<T> = Result<T, ProjectBuildError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V14ProjectManifest {
    name: String,
    version: String,
    entry_module: String,
    source_root: String,
    canonical_text: String,
}

impl V14ProjectManifest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn entry_module(&self) -> &str {
        &self.entry_module
    }

    pub fn source_root(&self) -> &str {
        &self.source_root
    }

    pub fn canonical_text(&self) -> &str {
        &self.canonical_text
    }

    pub fn package_file_name(&self) -> String {
        format!("{}-{}.npkg", self.name, self.version)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V14ProjectBuild {
    manifest: V14ProjectManifest,
    module_plan: V13ModuleGraphPlan,
    lowering: V13ModuleGraphNairArtifact,
    witness: Vec<u8>,
    package_bytes: Vec<u8>,
    lock_text: String,
}

impl V14ProjectBuild {
    pub fn manifest(&self) -> &V14ProjectManifest {
        &self.manifest
    }

    pub fn module_plan(&self) -> &V13ModuleGraphPlan {
        &self.module_plan
    }

    pub fn lowering(&self) -> &V13ModuleGraphNairArtifact {
        &self.lowering
    }

    pub fn canonical_v14_build_witness_bytes(&self) -> &[u8] {
        &self.witness
    }

    pub fn package_bytes(&self) -> &[u8] {
        &self.package_bytes
    }

    pub fn lock_text(&self) -> &str {
        &self.lock_text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V14PackageInfo {
    name: String,
    version: String,
    entry_module: String,
    source_root: String,
    module_order: Vec<String>,
    import_edges: Vec<(String, String)>,
    build_witness: Vec<u8>,
    nair_format_minor: u16,
    nair_instruction_count: usize,
}

impl V14PackageInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn entry_module(&self) -> &str {
        &self.entry_module
    }

    pub fn source_root(&self) -> &str {
        &self.source_root
    }

    pub fn module_order(&self) -> &[String] {
        &self.module_order
    }

    pub fn import_edges(&self) -> &[(String, String)] {
        &self.import_edges
    }

    pub fn build_witness(&self) -> &[u8] {
        &self.build_witness
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.nair_format_minor
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.nair_instruction_count
    }
}

pub fn parse_project_manifest_v14(text: &str) -> ProjectBuildResult<V14ProjectManifest> {
    if text.len() > MAX_V14_MANIFEST_BYTES {
        return Err(manifest_error(format!(
            "manifest has {} bytes, exceeding certified bound {MAX_V14_MANIFEST_BYTES}",
            text.len()
        )));
    }

    let mut in_project = false;
    let mut project_seen = false;
    let mut name = None;
    let mut version = None;
    let mut entry = None;
    let mut source_root = None;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            if line != "[project]" {
                return Err(manifest_error(format!(
                    "unsupported section '{line}' at line {line_number}"
                )));
            }
            if project_seen {
                return Err(manifest_error("duplicate [project] section"));
            }
            project_seen = true;
            in_project = true;
            continue;
        }
        if !in_project {
            return Err(manifest_error(format!(
                "key/value before [project] section at line {line_number}"
            )));
        }

        let (raw_key, raw_value) = line.split_once('=').ok_or_else(|| {
            manifest_error(format!("expected key = \"value\" at line {line_number}"))
        })?;
        let key = raw_key.trim();
        let value = parse_quoted_value(raw_value.trim(), line_number)?;
        match key {
            "name" => set_once(&mut name, value, "name")?,
            "version" => set_once(&mut version, value, "version")?,
            "entry" => set_once(&mut entry, value, "entry")?,
            "source-root" => set_once(&mut source_root, value, "source-root")?,
            _ => {
                return Err(manifest_error(format!(
                    "unsupported project key '{key}' at line {line_number}"
                )))
            }
        }
    }

    if !project_seen {
        return Err(manifest_error("missing [project] section"));
    }
    let name = name.ok_or_else(|| manifest_error("missing project key 'name'"))?;
    let version = version.ok_or_else(|| manifest_error("missing project key 'version'"))?;
    let entry_module = entry.ok_or_else(|| manifest_error("missing project key 'entry'"))?;
    let source_root =
        source_root.ok_or_else(|| manifest_error("missing project key 'source-root'"))?;

    validate_project_name(&name)?;
    validate_version(&version)?;
    validate_module_name_v13(&entry_module).map_err(ProjectBuildError::Module)?;
    validate_source_root(&source_root)?;

    let canonical_text = format!(
        "[project]\nname = \"{name}\"\nversion = \"{version}\"\nentry = \"{entry_module}\"\nsource-root = \"{source_root}\"\n"
    );

    Ok(V14ProjectManifest {
        name,
        version,
        entry_module,
        source_root,
        canonical_text,
    })
}

pub fn compile_project_v14(
    manifest: &V14ProjectManifest,
    sources: &[SourceText],
) -> ProjectBuildResult<V14ProjectBuild> {
    let module_plan = compile_module_graph_v13(sources, manifest.entry_module())?;
    let lowering = lower_module_graph_v13(&module_plan)?;
    let nair_bytes = lowering.inner().canonical_nair_bytes();

    let mut witness = Vec::new();
    witness.extend_from_slice(V14_BUILD_WITNESS_DOMAIN);
    encode_string(manifest.name(), &mut witness)?;
    encode_string(manifest.version(), &mut witness)?;
    encode_string(manifest.entry_module(), &mut witness)?;
    encode_string(manifest.source_root(), &mut witness)?;
    encode_strings(module_plan.module_order(), &mut witness)?;
    encode_edges(module_plan.import_edges(), &mut witness)?;
    push_component(module_plan.canonical_v13_witness_bytes(), &mut witness)?;
    push_component(
        lowering.canonical_v13_lowering_witness_bytes(),
        &mut witness,
    )?;
    push_component(nair_bytes, &mut witness)?;

    let package_bytes = encode_package(
        manifest,
        module_plan.module_order(),
        module_plan.import_edges(),
        &witness,
        nair_bytes,
    )?;
    if package_bytes.len() > MAX_V14_PACKAGE_BYTES {
        return Err(package_error(format!(
            "package has {} bytes, exceeding certified bound {MAX_V14_PACKAGE_BYTES}",
            package_bytes.len()
        )));
    }

    let lock_text = build_lock_text(manifest, &module_plan, &lowering, &witness);

    Ok(V14ProjectBuild {
        manifest: manifest.clone(),
        module_plan,
        lowering,
        witness,
        package_bytes,
        lock_text,
    })
}

pub fn inspect_project_package_v14(bytes: &[u8]) -> ProjectBuildResult<V14PackageInfo> {
    if bytes.len() > MAX_V14_PACKAGE_BYTES {
        return Err(package_error(format!(
            "package has {} bytes, exceeding certified bound {MAX_V14_PACKAGE_BYTES}",
            bytes.len()
        )));
    }
    let mut decoder = Decoder::new(bytes);
    let magic = decoder.read_exact(4)?;
    if magic != V14_PACKAGE_MAGIC {
        return Err(package_error("invalid package magic"));
    }
    let major = decoder.read_u16()?;
    let minor = decoder.read_u16()?;
    if major != V14_PACKAGE_MAJOR || minor != V14_PACKAGE_MINOR {
        return Err(package_error(format!(
            "unsupported package format {major}.{minor}"
        )));
    }

    let name = decoder.read_string()?;
    let version = decoder.read_string()?;
    let entry_module = decoder.read_string()?;
    let source_root = decoder.read_string()?;
    validate_project_name(&name)?;
    validate_version(&version)?;
    validate_module_name_v13(&entry_module).map_err(ProjectBuildError::Module)?;
    validate_source_root(&source_root)?;

    let module_order = decoder.read_strings()?;
    let import_edges = decoder.read_edges()?;
    let build_witness = decoder.read_bytes()?;
    let nair_bytes = decoder.read_bytes()?;
    if !decoder.is_finished() {
        return Err(package_error(
            "trailing bytes after canonical package payload",
        ));
    }

    validate_package_graph(&entry_module, &module_order, &import_edges)?;
    if !build_witness.starts_with(V14_BUILD_WITNESS_DOMAIN) {
        return Err(package_error(
            "package build witness has invalid V1.4 domain",
        ));
    }
    let program = NairProgram::from_canonical_bytes(&nair_bytes)?;
    if program.canonical_bytes()? != nair_bytes {
        return Err(package_error("embedded NAIR is not canonical"));
    }
    let nair_format_minor = program.required_format_minor();
    let nair_instruction_count = program.len();

    Ok(V14PackageInfo {
        name,
        version,
        entry_module,
        source_root,
        module_order,
        import_edges,
        build_witness,
        nair_format_minor,
        nair_instruction_count,
    })
}

fn validate_package_graph(
    entry_module: &str,
    module_order: &[String],
    import_edges: &[(String, String)],
) -> ProjectBuildResult<()> {
    if module_order.is_empty() || module_order.len() > MAX_V13_MODULES {
        return Err(package_error(
            "package module count is outside certified V1.3 bounds",
        ));
    }
    for module in module_order {
        validate_module_name_v13(module).map_err(ProjectBuildError::Module)?;
    }
    if !module_order
        .windows(2)
        .all(|pair| pair[0].as_str() < pair[1].as_str())
    {
        return Err(package_error("package module order is not canonical"));
    }
    if module_order
        .binary_search_by(|value| value.as_str().cmp(entry_module))
        .is_err()
    {
        return Err(package_error(
            "package entry module is absent from module table",
        ));
    }
    if import_edges.len() > MAX_V13_TOTAL_IMPORT_EDGES {
        return Err(package_error(
            "package import edge count exceeds certified V1.3 bound",
        ));
    }
    for (index, (from, to)) in import_edges.iter().enumerate() {
        if module_order.binary_search(from).is_err() || module_order.binary_search(to).is_err() {
            return Err(package_error(
                "package import edge references unknown module",
            ));
        }
        if import_edges[..index].iter().any(|existing| {
            existing.0.as_str() == from.as_str() && existing.1.as_str() == to.as_str()
        }) {
            return Err(package_error("package contains duplicate import edge"));
        }
    }
    Ok(())
}

fn encode_package(
    manifest: &V14ProjectManifest,
    module_order: &[String],
    import_edges: &[(String, String)],
    build_witness: &[u8],
    nair_bytes: &[u8],
) -> ProjectBuildResult<Vec<u8>> {
    let mut output = Vec::new();
    output.extend_from_slice(&V14_PACKAGE_MAGIC);
    output.extend_from_slice(&V14_PACKAGE_MAJOR.to_be_bytes());
    output.extend_from_slice(&V14_PACKAGE_MINOR.to_be_bytes());
    encode_string(manifest.name(), &mut output)?;
    encode_string(manifest.version(), &mut output)?;
    encode_string(manifest.entry_module(), &mut output)?;
    encode_string(manifest.source_root(), &mut output)?;
    encode_strings(module_order, &mut output)?;
    encode_edges(import_edges, &mut output)?;
    push_component(build_witness, &mut output)?;
    push_component(nair_bytes, &mut output)?;
    Ok(output)
}

fn build_lock_text(
    manifest: &V14ProjectManifest,
    module_plan: &V13ModuleGraphPlan,
    lowering: &V13ModuleGraphNairArtifact,
    witness: &[u8],
) -> String {
    let mut output = String::new();
    output.push_str("# NORDOI lock v1.4 -- generated; do not edit\n");
    output.push_str("format = 1\n");
    output.push_str(&format!("project = \"{}\"\n", manifest.name()));
    output.push_str(&format!("version = \"{}\"\n", manifest.version()));
    output.push_str(&format!("entry = \"{}\"\n", manifest.entry_module()));
    output.push_str(&format!("source-root = \"{}\"\n", manifest.source_root()));
    output.push_str(&format!("nair = \"0.{}\"\n", lowering.nair_format_minor()));
    output.push_str(&format!("modules = {}\n", module_plan.module_count()));
    for module in module_plan.module_order() {
        output.push_str(&format!("module = \"{module}\"\n"));
    }
    output.push_str(&format!("imports = {}\n", module_plan.import_count()));
    for (from, to) in module_plan.import_edges() {
        output.push_str(&format!("import = \"{from} -> {to}\"\n"));
    }
    output.push_str(&format!("build-witness = \"{}\"\n", hex_bytes(witness)));
    output
}

fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_quote {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch == '#' && !in_quote {
            return &line[..index];
        }
    }
    line
}

fn parse_quoted_value(value: &str, line_number: usize) -> ProjectBuildResult<String> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(manifest_error(format!(
            "value must be a quoted string at line {line_number}"
        )));
    }
    let inner = &value[1..value.len() - 1];
    if inner.contains('"') || inner.contains('\\') || inner.contains('\0') {
        return Err(manifest_error(format!(
            "escapes and embedded quotes are not supported at line {line_number}"
        )));
    }
    Ok(inner.to_owned())
}

fn set_once(slot: &mut Option<String>, value: String, key: &str) -> ProjectBuildResult<()> {
    if slot.replace(value).is_some() {
        return Err(manifest_error(format!("duplicate project key '{key}'")));
    }
    Ok(())
}

fn validate_project_name(name: &str) -> ProjectBuildResult<()> {
    if name.is_empty() || name.len() > MAX_V14_PROJECT_NAME_BYTES {
        return Err(manifest_error(format!(
            "project name must contain 1..={MAX_V14_PROJECT_NAME_BYTES} bytes"
        )));
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(manifest_error(
            "project name may contain only ASCII letters, digits, '-' and '_'",
        ));
    }
    if !name.as_bytes()[0].is_ascii_alphanumeric() {
        return Err(manifest_error(
            "project name must begin with an ASCII letter or digit",
        ));
    }
    Ok(())
}

fn validate_version(version: &str) -> ProjectBuildResult<()> {
    if version.is_empty() || version.len() > MAX_V14_VERSION_BYTES {
        return Err(manifest_error(format!(
            "project version must contain 1..={MAX_V14_VERSION_BYTES} bytes"
        )));
    }
    if !version
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_'))
    {
        return Err(manifest_error(
            "project version contains unsupported characters",
        ));
    }
    Ok(())
}

fn validate_source_root(source_root: &str) -> ProjectBuildResult<()> {
    if source_root.is_empty() || source_root.len() > MAX_V14_SOURCE_ROOT_BYTES {
        return Err(manifest_error(format!(
            "source-root must contain 1..={MAX_V14_SOURCE_ROOT_BYTES} bytes"
        )));
    }
    if source_root.starts_with('/')
        || source_root.starts_with('\\')
        || source_root.contains('\\')
        || source_root.contains(':')
    {
        return Err(manifest_error(
            "source-root must be a portable relative path",
        ));
    }
    for segment in source_root.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(manifest_error(
                "source-root cannot contain empty, '.' or '..' segments",
            ));
        }
        if !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(manifest_error(
                "source-root segments may contain only ASCII letters, digits, '-' and '_'",
            ));
        }
    }
    Ok(())
}

fn encode_strings(values: &[String], output: &mut Vec<u8>) -> ProjectBuildResult<()> {
    let len = u32::try_from(values.len()).map_err(|_| package_error("too many strings"))?;
    output.extend_from_slice(&len.to_be_bytes());
    for value in values {
        encode_string(value, output)?;
    }
    Ok(())
}

fn encode_edges(values: &[(String, String)], output: &mut Vec<u8>) -> ProjectBuildResult<()> {
    let len = u32::try_from(values.len()).map_err(|_| package_error("too many import edges"))?;
    output.extend_from_slice(&len.to_be_bytes());
    for (from, to) in values {
        encode_string(from, output)?;
        encode_string(to, output)?;
    }
    Ok(())
}

fn encode_string(value: &str, output: &mut Vec<u8>) -> ProjectBuildResult<()> {
    push_component(value.as_bytes(), output)
}

fn push_component(value: &[u8], output: &mut Vec<u8>) -> ProjectBuildResult<()> {
    let len = u32::try_from(value.len()).map_err(|_| package_error("component too large"))?;
    output.extend_from_slice(&len.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn manifest_error(message: impl Into<String>) -> ProjectBuildError {
    ProjectBuildError::Manifest {
        message: message.into(),
    }
}

fn package_error(message: impl Into<String>) -> ProjectBuildError {
    ProjectBuildError::Package {
        message: message.into(),
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn read_exact(&mut self, length: usize) -> ProjectBuildResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| package_error("package length overflow"))?;
        if end > self.bytes.len() {
            return Err(package_error("truncated package"));
        }
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> ProjectBuildResult<u16> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> ProjectBuildResult<u32> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_bytes(&mut self) -> ProjectBuildResult<Vec<u8>> {
        let length = self.read_u32()? as usize;
        Ok(self.read_exact(length)?.to_vec())
    }

    fn read_string(&mut self) -> ProjectBuildResult<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(|_| package_error("package contains non-UTF-8 text"))
    }

    fn read_strings(&mut self) -> ProjectBuildResult<Vec<String>> {
        let count = self.read_u32()? as usize;
        if count > 1024 {
            return Err(package_error("unreasonable string table count"));
        }
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_string()?);
        }
        Ok(values)
    }

    fn read_edges(&mut self) -> ProjectBuildResult<Vec<(String, String)>> {
        let count = self.read_u32()? as usize;
        if count > 4096 {
            return Err(package_error("unreasonable import edge count"));
        }
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push((self.read_string()?, self.read_string()?));
        }
        Ok(values)
    }
}
