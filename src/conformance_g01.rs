use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

pub const G01_SCHEMA: &str = "nordoi.constitutional-conformance.g0.1";
pub const G01_BASELINE_TAG: &str = "p2.7";
pub const G01_BASELINE_COMMIT: &str = "0a634d6cab52074e18b42d1ffea00ce02cdba359";
pub const G01_BASELINE_CI: u64 = 37_956_122_196;
pub const G01_PRINCIPLE_COUNT: usize = 322;
pub const G01_DELIVERY_BASIS_POINTS: u32 = 5_835;
pub const G01_FUTURE_NATIVE_RULE_COUNT: usize = 6;

const PRINCIPLES_TSV: &str = include_str!("../governance/constitutional_conformance_v1.tsv");
const DELIVERY_TSV: &str = include_str!("../governance/nordoi_v1_delivery_model_v1.tsv");
const FUTURE_GATE_TSV: &str = include_str!("../governance/future_native_gate_v1.tsv");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStatus {
    Certified,
    Partial,
}

impl EvidenceStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "CERTIFIED" => Some(Self::Certified),
            "PARTIAL" => Some(Self::Partial),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Certified => "CERTIFIED",
            Self::Partial => "PARTIAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalPrinciple {
    pub id: String,
    pub title: String,
    pub evidence_status: EvidenceStatus,
    pub scope: String,
    pub evidence_ref: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryDomain {
    pub id: String,
    pub title: String,
    pub weight_percent: u32,
    pub completion_percent: u32,
    pub status: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FutureNativeRule {
    pub id: String,
    pub rule: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConformanceError {
    MalformedRow {
        table: &'static str,
        line: usize,
    },
    InvalidPrincipleId {
        line: usize,
        value: String,
    },
    InvalidEvidenceStatus {
        line: usize,
        value: String,
    },
    DuplicatePrincipleId {
        id: String,
    },
    MissingPrinciple {
        expected: String,
    },
    InvalidNumber {
        table: &'static str,
        line: usize,
        value: String,
    },
    InvalidDeliveryWeight {
        sum: u32,
    },
    InvalidCompletionPercent {
        line: usize,
        value: u32,
    },
    InvalidPrincipleCount {
        count: usize,
    },
    InvalidFutureRuleCount {
        count: usize,
    },
    EmptyField {
        table: &'static str,
        line: usize,
    },
}

impl Display for ConformanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow { table, line } => write!(f, "malformed {table} row at line {line}"),
            Self::InvalidPrincipleId { line, value } => {
                write!(
                    f,
                    "invalid constitutional principle id '{value}' at line {line}"
                )
            }
            Self::InvalidEvidenceStatus { line, value } => {
                write!(f, "invalid evidence status '{value}' at line {line}")
            }
            Self::DuplicatePrincipleId { id } => {
                write!(f, "duplicate constitutional principle '{id}'")
            }
            Self::MissingPrinciple { expected } => {
                write!(f, "missing constitutional principle '{expected}'")
            }
            Self::InvalidNumber { table, line, value } => {
                write!(f, "invalid number '{value}' in {table} at line {line}")
            }
            Self::InvalidDeliveryWeight { sum } => {
                write!(f, "delivery weights must sum to 100, got {sum}")
            }
            Self::InvalidCompletionPercent { line, value } => {
                write!(
                    f,
                    "delivery completion at line {line} must be <= 100, got {value}"
                )
            }
            Self::InvalidPrincipleCount { count } => {
                write!(
                    f,
                    "constitutional principle count must be {G01_PRINCIPLE_COUNT}, got {count}"
                )
            }
            Self::InvalidFutureRuleCount { count } => {
                write!(
                    f,
                    "future-native rule count must be {G01_FUTURE_NATIVE_RULE_COUNT}, got {count}"
                )
            }
            Self::EmptyField { table, line } => write!(f, "empty field in {table} at line {line}"),
        }
    }
}

impl std::error::Error for ConformanceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalConformanceReport {
    principles: Vec<ConstitutionalPrinciple>,
    delivery_domains: Vec<DeliveryDomain>,
    future_native_rules: Vec<FutureNativeRule>,
    certified_count: usize,
    partial_count: usize,
    evidence_basis_points: u32,
    delivery_basis_points: u32,
}

impl ConstitutionalConformanceReport {
    pub fn principles(&self) -> &[ConstitutionalPrinciple] {
        &self.principles
    }

    pub fn delivery_domains(&self) -> &[DeliveryDomain] {
        &self.delivery_domains
    }

    pub fn future_native_rules(&self) -> &[FutureNativeRule] {
        &self.future_native_rules
    }

    pub const fn certified_count(&self) -> usize {
        self.certified_count
    }

    pub const fn partial_count(&self) -> usize {
        self.partial_count
    }

    pub const fn evidence_basis_points(&self) -> u32 {
        self.evidence_basis_points
    }

    pub const fn delivery_basis_points(&self) -> u32 {
        self.delivery_basis_points
    }

    pub fn render_text(&self) -> String {
        let mut out = String::new();
        out.push_str("constitutional-conformance");
        out.push_str(" schema=\"");
        out.push_str(G01_SCHEMA);
        out.push_str("\" baseline-tag=\"");
        out.push_str(G01_BASELINE_TAG);
        out.push_str("\" baseline-commit=");
        out.push_str(G01_BASELINE_COMMIT);
        out.push_str(" baseline-ci=");
        out.push_str(&G01_BASELINE_CI.to_string());
        out.push_str(" principles=");
        out.push_str(&self.principles.len().to_string());
        out.push_str(" certified=");
        out.push_str(&self.certified_count.to_string());
        out.push_str(" partial=");
        out.push_str(&self.partial_count.to_string());
        out.push_str(" evidence-coverage=");
        out.push_str(&format_basis_points(self.evidence_basis_points));
        out.push_str(" full-v1-delivery-estimate=");
        out.push_str(&format_basis_points(self.delivery_basis_points));
        out.push_str(" delivery-estimate-kind=PLANNING_NOT_CERTIFICATION");
        out.push_str(" future-native-gate=MANDATORY_GOVERNANCE");
        out.push_str(" future-native-rules=");
        out.push_str(&self.future_native_rules.len().to_string());
        out.push_str(" runtime-semantics=UNCHANGED authority=NONE status=GOVERNED\n");
        out
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"status\":\"governed\",\"baselineTag\":\"{}\",\"baselineCommit\":\"{}\",\"baselineCi\":{},\"principles\":{},\"certified\":{},\"partial\":{},\"evidenceCoverageBasisPoints\":{},\"fullV1DeliveryEstimateBasisPoints\":{},\"deliveryEstimateKind\":\"planning_not_certification\",\"futureNativeGate\":\"mandatory_governance\",\"futureNativeRules\":{},\"runtimeSemantics\":\"unchanged\",\"authority\":\"none\"}}",
            G01_SCHEMA,
            G01_BASELINE_TAG,
            G01_BASELINE_COMMIT,
            G01_BASELINE_CI,
            self.principles.len(),
            self.certified_count,
            self.partial_count,
            self.evidence_basis_points,
            self.delivery_basis_points,
            self.future_native_rules.len(),
        )
    }
}

pub fn constitutional_conformance_report_g01(
) -> Result<ConstitutionalConformanceReport, ConformanceError> {
    let principles = parse_principles(PRINCIPLES_TSV)?;
    let delivery_domains = parse_delivery_domains(DELIVERY_TSV)?;
    let future_native_rules = parse_future_native_rules(FUTURE_GATE_TSV)?;

    if principles.len() != G01_PRINCIPLE_COUNT {
        return Err(ConformanceError::InvalidPrincipleCount {
            count: principles.len(),
        });
    }
    if future_native_rules.len() != G01_FUTURE_NATIVE_RULE_COUNT {
        return Err(ConformanceError::InvalidFutureRuleCount {
            count: future_native_rules.len(),
        });
    }

    for (index, principle) in principles.iter().enumerate() {
        let expected = format!("C{}", index + 1);
        if principle.id != expected {
            return Err(ConformanceError::MissingPrinciple { expected });
        }
    }

    let certified_count = principles
        .iter()
        .filter(|principle| principle.evidence_status == EvidenceStatus::Certified)
        .count();
    let partial_count = principles.len() - certified_count;
    let evidence_basis_points =
        ((certified_count as u64 * 10_000) / principles.len() as u64) as u32;
    let delivery_basis_points = delivery_domains.iter().fold(0_u32, |sum, domain| {
        sum + domain.weight_percent * domain.completion_percent
    });

    if delivery_basis_points != G01_DELIVERY_BASIS_POINTS {
        return Err(ConformanceError::InvalidNumber {
            table: "delivery",
            line: 0,
            value: delivery_basis_points.to_string(),
        });
    }

    Ok(ConstitutionalConformanceReport {
        principles,
        delivery_domains,
        future_native_rules,
        certified_count,
        partial_count,
        evidence_basis_points,
        delivery_basis_points,
    })
}

fn parse_principles(input: &str) -> Result<Vec<ConstitutionalPrinciple>, ConformanceError> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in input.lines().enumerate().skip(1) {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(ConformanceError::MalformedRow {
                table: "principles",
                line: line_number,
            });
        }
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err(ConformanceError::EmptyField {
                table: "principles",
                line: line_number,
            });
        }
        let id = fields[0];
        let numeric = id
            .strip_prefix('C')
            .and_then(|value| value.parse::<usize>().ok());
        if numeric.is_none() {
            return Err(ConformanceError::InvalidPrincipleId {
                line: line_number,
                value: id.to_owned(),
            });
        }
        if !seen.insert(id.to_owned()) {
            return Err(ConformanceError::DuplicatePrincipleId { id: id.to_owned() });
        }
        let evidence_status = EvidenceStatus::parse(fields[2]).ok_or_else(|| {
            ConformanceError::InvalidEvidenceStatus {
                line: line_number,
                value: fields[2].to_owned(),
            }
        })?;
        result.push(ConstitutionalPrinciple {
            id: id.to_owned(),
            title: fields[1].to_owned(),
            evidence_status,
            scope: fields[3].to_owned(),
            evidence_ref: fields[4].to_owned(),
            next_action: fields[5].to_owned(),
        });
    }
    Ok(result)
}

fn parse_delivery_domains(input: &str) -> Result<Vec<DeliveryDomain>, ConformanceError> {
    let mut result = Vec::new();
    let mut weight_sum = 0_u32;
    for (index, line) in input.lines().enumerate().skip(1) {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(ConformanceError::MalformedRow {
                table: "delivery",
                line: line_number,
            });
        }
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err(ConformanceError::EmptyField {
                table: "delivery",
                line: line_number,
            });
        }
        let weight_percent =
            fields[2]
                .parse::<u32>()
                .map_err(|_| ConformanceError::InvalidNumber {
                    table: "delivery",
                    line: line_number,
                    value: fields[2].to_owned(),
                })?;
        let completion_percent =
            fields[3]
                .parse::<u32>()
                .map_err(|_| ConformanceError::InvalidNumber {
                    table: "delivery",
                    line: line_number,
                    value: fields[3].to_owned(),
                })?;
        if completion_percent > 100 {
            return Err(ConformanceError::InvalidCompletionPercent {
                line: line_number,
                value: completion_percent,
            });
        }
        weight_sum += weight_percent;
        result.push(DeliveryDomain {
            id: fields[0].to_owned(),
            title: fields[1].to_owned(),
            weight_percent,
            completion_percent,
            status: fields[4].to_owned(),
            next_action: fields[5].to_owned(),
        });
    }
    if weight_sum != 100 {
        return Err(ConformanceError::InvalidDeliveryWeight { sum: weight_sum });
    }
    Ok(result)
}

fn parse_future_native_rules(input: &str) -> Result<Vec<FutureNativeRule>, ConformanceError> {
    let mut result = Vec::new();
    for (index, line) in input.lines().enumerate().skip(1) {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err(ConformanceError::MalformedRow {
                table: "future-native-gate",
                line: line_number,
            });
        }
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err(ConformanceError::EmptyField {
                table: "future-native-gate",
                line: line_number,
            });
        }
        result.push(FutureNativeRule {
            id: fields[0].to_owned(),
            rule: fields[1].to_owned(),
            description: fields[2].to_owned(),
        });
    }
    Ok(result)
}

fn format_basis_points(value: u32) -> String {
    format!("{}.{:02}%", value / 100, value % 100)
}
