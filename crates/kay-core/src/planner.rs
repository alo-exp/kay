//! Planning System for Kay
//!
//! Enhanced planning with requirements tracking, threat modeling,
//! rollback plans, and quality gates.
//!
//! ## Features
//!
//! - REQ-ID requirement mapping
//! - Threat model per phase
//! - Rollback plan generation
//! - Quality gate compliance (9 dimensions)
//! - Milestone tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A requirement with ID, description, and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Requirement ID (e.g., "REQ-01", "FUNC-01")
    pub id: String,
    /// Requirement description
    pub description: String,
    /// Whether the requirement is met
    pub met: bool,
    /// Evidence that the requirement is met
    pub evidence: Vec<String>,
}

/// Threat identified in the threat model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    /// Threat ID
    pub id: String,
    /// Threat description
    pub description: String,
    /// Severity (critical, high, medium, low)
    pub severity: ThreatSeverity,
    /// Mitigation strategy
    pub mitigation: String,
}

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// A rollback step for reverting changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step description
    pub description: String,
    /// Command to execute for rollback
    pub command: String,
}

/// Quality gate dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityGateDim {
    /// Code quality gates
    CodeQuality,
    /// Security gates
    Security,
    /// Performance gates
    Performance,
    /// Test coverage gates
    TestCoverage,
    /// Documentation gates
    Documentation,
    /// API contract gates
    ApiContract,
    /// Error handling gates
    ErrorHandling,
    /// Resource usage gates
    ResourceUsage,
    /// Observability gates
    Observability,
}

/// A quality gate check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Gate dimension
    pub dimension: QualityGateDim,
    /// Gate description
    pub description: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Notes about the gate status
    pub notes: String,
}

/// A phase in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    /// Phase name
    pub name: String,
    /// Phase description
    pub description: String,
    /// Requirements for this phase
    pub requirements: Vec<Requirement>,
    /// Threats for this phase
    pub threats: Vec<Threat>,
    /// Rollback plan for this phase
    pub rollback: Vec<RollbackStep>,
    /// Quality gates for this phase
    pub quality_gates: Vec<QualityGate>,
    /// Whether this phase is complete
    pub complete: bool,
}

/// A milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone ID
    pub id: String,
    /// Milestone name
    pub name: String,
    /// Milestone description
    pub description: String,
    /// Phases in this milestone
    pub phases: Vec<Phase>,
    /// Whether this milestone is complete
    pub complete: bool,
}

/// The complete plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Plan title
    pub title: String,
    /// Plan description
    pub description: String,
    /// Milestones
    pub milestones: Vec<Milestone>,
    /// Overall status
    pub status: PlanStatus,
}

/// Plan status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// Plan is being created
    Draft,
    /// Plan is ready for execution
    Ready,
    /// Plan is being executed
    InProgress,
    /// Plan completed successfully
    Completed,
    /// Plan failed
    Failed,
}

/// Default quality gates for a phase
pub fn default_quality_gates() -> Vec<QualityGate> {
    vec![
        QualityGate {
            dimension: QualityGateDim::CodeQuality,
            description: "Code compiles without errors".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::CodeQuality,
            description: "No clippy warnings (deny warnings)".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::CodeQuality,
            description: "Formatting correct (cargo fmt)".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::TestCoverage,
            description: "All tests pass".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::TestCoverage,
            description: "New tests added for new functionality".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::Documentation,
            description: "Public API documented".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::ErrorHandling,
            description: "Error types properly defined".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::ErrorHandling,
            description: "Error messages are actionable".to_string(),
            passed: false,
            notes: String::new(),
        },
        QualityGate {
            dimension: QualityGateDim::ResourceUsage,
            description: "No unbounded loops or recursion".to_string(),
            passed: false,
            notes: String::new(),
        },
    ]
}

/// Create a new phase with default quality gates
pub fn create_phase(name: &str, description: &str) -> Phase {
    Phase {
        name: name.to_string(),
        description: description.to_string(),
        requirements: Vec::new(),
        threats: Vec::new(),
        rollback: Vec::new(),
        quality_gates: default_quality_gates(),
        complete: false,
    }
}

/// Create a requirement with the given ID and description
pub fn create_requirement(id: &str, description: &str) -> Requirement {
    Requirement {
        id: id.to_string(),
        description: description.to_string(),
        met: false,
        evidence: Vec::new(),
    }
}

/// Create a threat with the given parameters
pub fn create_threat(
    id: &str,
    description: &str,
    severity: ThreatSeverity,
    mitigation: &str,
) -> Threat {
    Threat {
        id: id.to_string(),
        description: description.to_string(),
        severity,
        mitigation: mitigation.to_string(),
    }
}

/// Create a rollback step
pub fn create_rollback_step(description: &str, command: &str) -> RollbackStep {
    RollbackStep {
        description: description.to_string(),
        command: command.to_string(),
    }
}

/// Mark a quality gate as passed
pub fn pass_quality_gate(gate: &mut QualityGate, notes: &str) {
    gate.passed = true;
    gate.notes = notes.to_string();
}

/// Mark a quality gate as failed
pub fn fail_quality_gate(gate: &mut QualityGate, notes: &str) {
    gate.passed = false;
    gate.notes = notes.to_string();
}

/// Check if all quality gates in a phase have passed
pub fn check_quality_gates(phase: &Phase) -> bool {
    phase.quality_gates.iter().all(|g| g.passed)
}

/// Generate a threat model for a phase
pub fn generate_threat_model(phase: &str) -> Vec<Threat> {
    match phase {
        "implementation" => vec![
            create_threat(
                "THR-01",
                "Code introduces memory leaks",
                ThreatSeverity::High,
                "Use Valgrind/miri for testing, follow Rust lifetime rules",
            ),
            create_threat(
                "THR-02",
                "Panics crash the entire process",
                ThreatSeverity::High,
                "Use Result types, avoid unwrap/expect in production code",
            ),
            create_threat(
                "THR-03",
                "Race conditions in multi-threaded code",
                ThreatSeverity::Critical,
                "Use Send+Sync trait bounds, test with loom",
            ),
        ],
        "testing" => vec![
            create_threat(
                "THR-04",
                "Tests miss edge cases",
                ThreatSeverity::Medium,
                "Add property-based tests, fuzz testing",
            ),
            create_threat(
                "THR-05",
                "Flaky tests cause CI to fail randomly",
                ThreatSeverity::Medium,
                "Use deterministic inputs, mock external services",
            ),
        ],
        "deployment" => vec![
            create_threat(
                "THR-06",
                "Breaking changes to public API",
                ThreatSeverity::Critical,
                "Follow semver, add breaking change tests",
            ),
            create_threat(
                "THR-07",
                "Configuration errors in production",
                ThreatSeverity::High,
                "Validate config at startup, provide defaults",
            ),
        ],
        _ => Vec::new(),
    }
}

/// Generate rollback steps for a phase
pub fn generate_rollback_plan(phase: &str) -> Vec<RollbackStep> {
    match phase {
        "implementation" => vec![
            create_rollback_step(
                "Revert code changes",
                "git checkout HEAD -- crates/",
            ),
            create_rollback_step(
                "Reset build artifacts",
                "cargo clean",
            ),
        ],
        "testing" => vec![
            create_rollback_step(
                "Revert test changes",
                "git checkout HEAD -- tests/",
            ),
        ],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_phase() {
        let phase = create_phase("test", "Test phase");
        assert_eq!(phase.name, "test");
        assert!(!phase.complete);
        assert!(!phase.quality_gates.is_empty());
    }

    #[test]
    fn test_create_requirement() {
        let req = create_requirement("REQ-01", "Test requirement");
        assert_eq!(req.id, "REQ-01");
        assert!(!req.met);
    }

    #[test]
    fn test_quality_gate_pass() {
        let mut gate = default_quality_gates()[0].clone();
        pass_quality_gate(&mut gate, "Compilation successful");
        assert!(gate.passed);
        assert_eq!(gate.notes, "Compilation successful");
    }

    #[test]
    fn test_check_quality_gates() {
        let mut phase = create_phase("test", "Test");
        assert!(!check_quality_gates(&phase));
        
        for gate in &mut phase.quality_gates {
            gate.passed = true;
        }
        assert!(check_quality_gates(&phase));
    }
}
