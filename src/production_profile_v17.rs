use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::effect_audit::hash::sha256;
use crate::project_v14::V14ProjectBuild;
use crate::release_v16::{
    distribution_plan_v16, verify_release_candidate_v16, ReleaseCandidateError,
    V16ReleaseCandidateReport, V16_RELEASE_SCHEMA,
};

pub const V17_PROFILE1_SCHEMA: &str = "nordoi.production-profile-1.v1";
pub const V17_PROFILE1_NAME: &str = "Production Profile 1";
pub const MAX_V17_CHECKSUM_BYTES: usize = 512;
pub const MAX_V17_CERTIFICATE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionProfileCertificationError {
    ReleaseCandidate(ReleaseCandidateError),
    DistributionPackageMismatch,
    ChecksumMismatch,
    ProvenanceMismatch,
    Invariant { message: String },
}

impl Display for ProductionProfileCertificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReleaseCandidate(error) => write!(f, "V1.7 release prerequisite failed: {error}"),
            Self::DistributionPackageMismatch => write!(
                f,
                "V1.7 distribution package mismatch: dist package is not byte-identical to the certified build package"
            ),
            Self::ChecksumMismatch => write!(
                f,
                "V1.7 checksum mismatch: distributed checksum is not the canonical checksum for the certified package"
            ),
            Self::ProvenanceMismatch => write!(
                f,
                "V1.7 provenance mismatch: distributed provenance is not the canonical V1.6 provenance"
            ),
            Self::Invariant { message } => {
                write!(f, "V1.7 Production Profile 1 invariant failed: {message}")
            }
        }
    }
}

impl Error for ProductionProfileCertificationError {}

impl From<ReleaseCandidateError> for ProductionProfileCertificationError {
    fn from(value: ReleaseCandidateError) -> Self {
        Self::ReleaseCandidate(value)
    }
}

pub type ProductionProfileCertificationResult<T> = Result<T, ProductionProfileCertificationError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V17ProductionProfileCertificate {
    project: String,
    version: String,
    entry_module: String,
    module_count: usize,
    import_count: usize,
    nair_format_minor: u16,
    nair_instruction_count: usize,
    package_file_name: String,
    package_sha256: [u8; 32],
    build_witness_sha256: [u8; 32],
    provenance_sha256: [u8; 32],
    certification_sha256: [u8; 32],
    certificate_text: String,
}

impl V17ProductionProfileCertificate {
    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn entry_module(&self) -> &str {
        &self.entry_module
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

    pub fn package_file_name(&self) -> &str {
        &self.package_file_name
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

    pub fn certification_sha256(&self) -> &[u8; 32] {
        &self.certification_sha256
    }

    pub fn certification_sha256_hex(&self) -> String {
        hex_bytes(&self.certification_sha256)
    }

    pub fn certificate_text(&self) -> &str {
        &self.certificate_text
    }

    pub fn render_text(&self) -> String {
        format!(
            "profile1-certify project=\"{}\" version=\"{}\" entry-module=\"{}\" modules={} imports={} nair-minor=0.{} nair-instructions={} package=\"{}\" source-lock=EXACT build-package=EXACT dist-package=EXACT checksum=EXACT provenance=EXACT package-sha256={} certification-sha256={} status=CERTIFIED profile=\"Production Profile 1\" reproducible=true platform-neutral=true dependency-network=NONE runtime-fs=NONE authority=NONE\n",
            escape_text(&self.project),
            escape_text(&self.version),
            escape_text(&self.entry_module),
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count,
            escape_text(&self.package_file_name),
            self.package_sha256_hex(),
            self.certification_sha256_hex(),
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"status\":\"certified\",\"profile\":\"{}\",\"project\":\"{}\",\"version\":\"{}\",\"entryModule\":\"{}\",\"modules\":{},\"imports\":{},\"nairMinor\":{},\"nairInstructions\":{},\"package\":\"{}\",\"sourceLock\":\"exact\",\"buildPackage\":\"exact\",\"distPackage\":\"exact\",\"checksum\":\"exact\",\"provenance\":\"exact\",\"packageSha256\":\"{}\",\"buildWitnessSha256\":\"{}\",\"provenanceSha256\":\"{}\",\"certificationSha256\":\"{}\",\"reproducible\":true,\"platformNeutral\":true,\"dependencyNetwork\":\"NONE\",\"runtimeFs\":\"NONE\",\"authority\":\"NONE\"}}",
            V17_PROFILE1_SCHEMA,
            V17_PROFILE1_NAME,
            escape_json(&self.project),
            escape_json(&self.version),
            escape_json(&self.entry_module),
            self.module_count,
            self.import_count,
            self.nair_format_minor,
            self.nair_instruction_count,
            escape_json(&self.package_file_name),
            self.package_sha256_hex(),
            self.build_witness_sha256_hex(),
            self.provenance_sha256_hex(),
            self.certification_sha256_hex(),
        )
    }
}

pub fn certify_production_profile1_v17(
    build: &V14ProjectBuild,
    existing_lock_text: &str,
    build_package_bytes: &[u8],
    dist_package_bytes: &[u8],
    checksum_text: &str,
    provenance_text: &str,
) -> ProductionProfileCertificationResult<V17ProductionProfileCertificate> {
    if checksum_text.len() > MAX_V17_CHECKSUM_BYTES {
        return Err(ProductionProfileCertificationError::Invariant {
            message: format!(
                "checksum has {} bytes, exceeding certified bound {MAX_V17_CHECKSUM_BYTES}",
                checksum_text.len()
            ),
        });
    }

    let release = verify_release_candidate_v16(build, existing_lock_text, build_package_bytes)?;
    let plan = distribution_plan_v16(&release, build_package_bytes)?;

    if dist_package_bytes != build_package_bytes {
        return Err(ProductionProfileCertificationError::DistributionPackageMismatch);
    }
    if checksum_text != plan.checksum_text() {
        return Err(ProductionProfileCertificationError::ChecksumMismatch);
    }
    if provenance_text != plan.provenance_text() {
        return Err(ProductionProfileCertificationError::ProvenanceMismatch);
    }

    let certificate_text = canonical_certificate_text(&release);
    if certificate_text.len() > MAX_V17_CERTIFICATE_BYTES {
        return Err(ProductionProfileCertificationError::Invariant {
            message: format!(
                "certificate has {} bytes, exceeding certified bound {MAX_V17_CERTIFICATE_BYTES}",
                certificate_text.len()
            ),
        });
    }
    let certification_sha256 = sha256(certificate_text.as_bytes());

    Ok(V17ProductionProfileCertificate {
        project: release.project().to_owned(),
        version: release.version().to_owned(),
        entry_module: release.entry_module().to_owned(),
        module_count: release.module_count(),
        import_count: release.import_count(),
        nair_format_minor: release.nair_format_minor(),
        nair_instruction_count: release.nair_instruction_count(),
        package_file_name: release.package_file_name().to_owned(),
        package_sha256: *release.package_sha256(),
        build_witness_sha256: *release.build_witness_sha256(),
        provenance_sha256: *release.provenance_sha256(),
        certification_sha256,
        certificate_text,
    })
}

fn canonical_certificate_text(release: &V16ReleaseCandidateReport) -> String {
    format!(
        "schema = \"{V17_PROFILE1_SCHEMA}\"\nprofile = \"{V17_PROFILE1_NAME}\"\nrelease-schema = \"{V16_RELEASE_SCHEMA}\"\nproject = \"{}\"\nversion = \"{}\"\nentry = \"{}\"\npackage = \"{}\"\npackage-sha256 = \"{}\"\nbuild-witness-sha256 = \"{}\"\nprovenance-sha256 = \"{}\"\nnair-minor = \"0.{}\"\nnair-instructions = {}\nmodules = {}\nimports = {}\nsource-lock = \"EXACT\"\nbuild-package = \"EXACT\"\ndist-package = \"EXACT\"\nchecksum = \"EXACT\"\nprovenance = \"EXACT\"\nreproducible = true\nplatform-neutral = true\ndependency-network = \"NONE\"\nruntime-fs = \"NONE\"\nauthority = \"NONE\"\n",
        release.project(),
        release.version(),
        release.entry_module(),
        release.package_file_name(),
        release.package_sha256_hex(),
        release.build_witness_sha256_hex(),
        release.provenance_sha256_hex(),
        release.nair_format_minor(),
        release.nair_instruction_count(),
        release.module_count(),
        release.import_count(),
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
