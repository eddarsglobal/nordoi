use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::effect_audit::hash::sha256;
use crate::project_v14::{
    inspect_project_package_v14, V14ProjectBuild, V14_PACKAGE_MAJOR, V14_PACKAGE_MINOR,
};

pub const V16_RELEASE_SCHEMA: &str = "nordoi.release.v1";
pub const V16_PROVENANCE_SCHEMA: &str = "nordoi.release.provenance.v1";
pub const MAX_V16_PROVENANCE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseCandidateError {
    LockMismatch,
    PackageInvalid { message: String },
    PackageMismatch,
    Invariant { message: String },
}

impl Display for ReleaseCandidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockMismatch => write!(
                f,
                "V1.6 release lock mismatch: NORDOI.lock does not match the current canonical build graph"
            ),
            Self::PackageInvalid { message } => {
                write!(f, "V1.6 release package validation failed: {message}")
            }
            Self::PackageMismatch => write!(
                f,
                "V1.6 release package mismatch: build artifact does not match the current canonical build"
            ),
            Self::Invariant { message } => write!(f, "V1.6 release invariant failed: {message}"),
        }
    }
}

impl Error for ReleaseCandidateError {}

pub type ReleaseCandidateResult<T> = Result<T, ReleaseCandidateError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V16ReleaseCandidateReport {
    project: String,
    version: String,
    entry_module: String,
    package_file_name: String,
    module_count: usize,
    import_count: usize,
    nair_format_minor: u16,
    nair_instruction_count: usize,
    package_sha256: [u8; 32],
    build_witness_sha256: [u8; 32],
    provenance_sha256: [u8; 32],
    provenance_text: String,
}

impl V16ReleaseCandidateReport {
    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn entry_module(&self) -> &str {
        &self.entry_module
    }

    pub fn package_file_name(&self) -> &str {
        &self.package_file_name
    }

    pub fn module_count(&self) -> usize {
        self.module_count
    }

    pub fn import_count(&self) -> usize {
        self.import_count
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.nair_format_minor
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.nair_instruction_count
    }

    pub fn package_sha256(&self) -> &[u8; 32] {
        &self.package_sha256
    }

    pub fn package_sha256_hex(&self) -> String {
        hex_bytes(&self.package_sha256)
    }

    pub fn build_witness_sha256(&self) -> &[u8; 32] {
        &self.build_witness_sha256
    }

    pub fn build_witness_sha256_hex(&self) -> String {
        hex_bytes(&self.build_witness_sha256)
    }

    pub fn provenance_sha256(&self) -> &[u8; 32] {
        &self.provenance_sha256
    }

    pub fn provenance_sha256_hex(&self) -> String {
        hex_bytes(&self.provenance_sha256)
    }

    pub fn provenance_text(&self) -> &str {
        &self.provenance_text
    }

    pub fn render_text(&self) -> String {
        format!(
            "release-check project=\"{}\" version=\"{}\" entry-module=\"{}\" modules={} imports={} nair-minor=0.{} nair-instructions={} package=\"{}\" package-format={}.{} lock=EXACT package-match=EXACT package-sha256={} provenance-sha256={} status=PASS reproducible=true platform-neutral=true dependency-network=NONE runtime-fs=NONE authority=NONE\n",
            escape_text(&self.project),
            escape_text(&self.version),
            escape_text(&self.entry_module),
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count,
            escape_text(&self.package_file_name),
            V14_PACKAGE_MAJOR,
            V14_PACKAGE_MINOR,
            self.package_sha256_hex(),
            self.provenance_sha256_hex(),
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"status\":\"pass\",\"project\":\"{}\",\"version\":\"{}\",\"entryModule\":\"{}\",\"modules\":{},\"imports\":{},\"nairMinor\":{},\"nairInstructions\":{},\"package\":\"{}\",\"packageFormat\":\"{}.{}\",\"lock\":\"exact\",\"packageMatch\":\"exact\",\"packageSha256\":\"{}\",\"buildWitnessSha256\":\"{}\",\"provenanceSha256\":\"{}\",\"reproducible\":true,\"platformNeutral\":true,\"dependencyNetwork\":\"NONE\",\"runtimeFs\":\"NONE\",\"authority\":\"NONE\"}}",
            V16_RELEASE_SCHEMA,
            escape_json(&self.project),
            escape_json(&self.version),
            escape_json(&self.entry_module),
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count,
            escape_json(&self.package_file_name),
            V14_PACKAGE_MAJOR,
            V14_PACKAGE_MINOR,
            self.package_sha256_hex(),
            self.build_witness_sha256_hex(),
            self.provenance_sha256_hex(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V16DistributionPlan {
    package_file_name: String,
    checksum_file_name: String,
    provenance_file_name: String,
    package_bytes: Vec<u8>,
    checksum_text: String,
    provenance_text: String,
}

impl V16DistributionPlan {
    pub fn package_file_name(&self) -> &str {
        &self.package_file_name
    }

    pub fn checksum_file_name(&self) -> &str {
        &self.checksum_file_name
    }

    pub fn provenance_file_name(&self) -> &str {
        &self.provenance_file_name
    }

    pub fn package_bytes(&self) -> &[u8] {
        &self.package_bytes
    }

    pub fn checksum_text(&self) -> &str {
        &self.checksum_text
    }

    pub fn provenance_text(&self) -> &str {
        &self.provenance_text
    }
}

pub fn verify_release_candidate_v16(
    build: &V14ProjectBuild,
    existing_lock_text: &str,
    existing_package_bytes: &[u8],
) -> ReleaseCandidateResult<V16ReleaseCandidateReport> {
    if existing_lock_text != build.lock_text() {
        return Err(ReleaseCandidateError::LockMismatch);
    }

    let package_info = inspect_project_package_v14(existing_package_bytes).map_err(|error| {
        ReleaseCandidateError::PackageInvalid {
            message: error.to_string(),
        }
    })?;

    if existing_package_bytes != build.package_bytes() {
        return Err(ReleaseCandidateError::PackageMismatch);
    }

    let manifest = build.manifest();
    if package_info.name() != manifest.name()
        || package_info.version() != manifest.version()
        || package_info.entry_module() != manifest.entry_module()
        || package_info.source_root() != manifest.source_root()
        || package_info.module_order() != build.module_plan().module_order()
        || package_info.import_edges() != build.module_plan().import_edges()
        || package_info.build_witness() != build.canonical_v14_build_witness_bytes()
        || package_info.nair_format_minor() != build.lowering().nair_format_minor()
        || package_info.nair_instruction_count() != build.lowering().nair_instruction_count()
    {
        return Err(ReleaseCandidateError::Invariant {
            message: "validated package metadata diverges from canonical build".to_owned(),
        });
    }

    let package_sha256 = sha256(existing_package_bytes);
    let build_witness_sha256 = sha256(build.canonical_v14_build_witness_bytes());
    let provenance_text = canonical_provenance_text(build, &package_sha256, &build_witness_sha256);
    if provenance_text.len() > MAX_V16_PROVENANCE_BYTES {
        return Err(ReleaseCandidateError::Invariant {
            message: format!(
                "provenance has {} bytes, exceeding certified bound {MAX_V16_PROVENANCE_BYTES}",
                provenance_text.len()
            ),
        });
    }
    let provenance_sha256 = sha256(provenance_text.as_bytes());

    Ok(V16ReleaseCandidateReport {
        project: manifest.name().to_owned(),
        version: manifest.version().to_owned(),
        entry_module: manifest.entry_module().to_owned(),
        package_file_name: manifest.package_file_name(),
        module_count: build.module_plan().module_count(),
        import_count: build.module_plan().import_count(),
        nair_format_minor: build.lowering().nair_format_minor(),
        nair_instruction_count: build.lowering().nair_instruction_count(),
        package_sha256,
        build_witness_sha256,
        provenance_sha256,
        provenance_text,
    })
}

pub fn distribution_plan_v16(
    report: &V16ReleaseCandidateReport,
    package_bytes: &[u8],
) -> ReleaseCandidateResult<V16DistributionPlan> {
    if sha256(package_bytes) != *report.package_sha256() {
        return Err(ReleaseCandidateError::PackageMismatch);
    }

    let package_file_name = report.package_file_name().to_owned();
    let checksum_file_name = format!("{package_file_name}.sha256");
    let provenance_file_name = format!("{}-{}.provenance", report.project(), report.version());
    let checksum_text = format!("{}  {}\n", report.package_sha256_hex(), package_file_name);

    Ok(V16DistributionPlan {
        package_file_name,
        checksum_file_name,
        provenance_file_name,
        package_bytes: package_bytes.to_vec(),
        checksum_text,
        provenance_text: report.provenance_text().to_owned(),
    })
}

fn canonical_provenance_text(
    build: &V14ProjectBuild,
    package_sha256: &[u8; 32],
    build_witness_sha256: &[u8; 32],
) -> String {
    let manifest = build.manifest();
    format!(
        "schema = \"{V16_PROVENANCE_SCHEMA}\"\nproject = \"{}\"\nversion = \"{}\"\nentry = \"{}\"\npackage = \"{}\"\npackage-sha256 = \"{}\"\nbuild-witness-sha256 = \"{}\"\npackage-format = \"{}.{}\"\nnair-minor = \"0.{}\"\nnair-instructions = {}\nmodules = {}\nimports = {}\nreproducible = true\nplatform-neutral = true\ndependency-network = \"NONE\"\nruntime-fs = \"NONE\"\nauthority = \"NONE\"\n",
        manifest.name(),
        manifest.version(),
        manifest.entry_module(),
        manifest.package_file_name(),
        hex_bytes(package_sha256),
        hex_bytes(build_witness_sha256),
        V14_PACKAGE_MAJOR,
        V14_PACKAGE_MINOR,
        build.lowering().nair_format_minor(),
        build.lowering().nair_instruction_count(),
        build.module_plan().module_count(),
        build.module_plan().import_count(),
    )
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

fn escape_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output
}

fn escape_json(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", ch as u32);
            }
            other => output.push(other),
        }
    }
    output
}
