// command_approval.rs — Phase 10 command approval flow (kay-tauri).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 5 (RED): ApprovalStore stub. Real implementation in GREEN wave.

use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::project_settings::CommandApproval;

/// Represents a decision made by the user for a tool approval request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ApprovalDecision {
    /// User approved the command.
    Approved,
    /// User denied the command.
    Denied,
}

/// Status of a sandbox verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SandboxStatus {
    /// Whether the sandbox is available.
    pub sandbox_available: bool,
    /// Paths that would be affected by this command.
    pub affected_paths: Vec<String>,
    /// Network hosts accessed by this command.
    pub network_access: Vec<String>,
}

/// Request for command approval from the frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ApprovalRequest {
    /// Unique ID for this request (used to match the response).
    pub request_id: String,
    /// Name of the tool being approved.
    pub tool_name: String,
    /// Command or arguments being approved.
    pub command: String,
    /// Sandbox status for the command.
    pub sandbox_status: SandboxStatus,
}

/// Response from the frontend for an approval request.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ApprovalResponse {
    /// ID of the original request.
    pub request_id: String,
    /// User's decision.
    pub decision: ApprovalDecision,
}

/// ApprovalStore persists approval decisions within a session.
#[derive(Debug, Default)]
pub struct ApprovalStore {
    /// Decisions made by the user, keyed by tool name.
    decisions: RwLock<HashMap<String, ApprovalDecision>>,
    /// Pending approval requests waiting for user response.
    pending_requests: RwLock<HashMap<String, ApprovalRequest>>,
}

impl ApprovalStore {
    /// Creates a new empty ApprovalStore.
    pub fn new() -> Self {
        Self {
            decisions: RwLock::new(HashMap::new()),
            pending_requests: RwLock::new(HashMap::new()),
        }
    }

    /// Records a decision for a tool.
    pub fn record_decision(&self, tool_name: &str, decision: ApprovalDecision) {
        let mut decisions = self.decisions.write().unwrap();
        decisions.insert(tool_name.to_string(), decision);
    }

    /// Gets the recorded decision for a tool, if any.
    pub fn get_decision(&self, tool_name: &str) -> Option<ApprovalDecision> {
        let decisions = self.decisions.read().unwrap();
        decisions.get(tool_name).cloned()
    }

    /// Adds a pending approval request.
    pub fn add_pending_request(&self, request: ApprovalRequest) {
        let mut pending = self.pending_requests.write().unwrap();
        pending.insert(request.request_id.clone(), request);
    }

    /// Gets and removes a pending approval request by ID.
    pub fn take_pending_request(&self, request_id: &str) -> Option<ApprovalRequest> {
        let mut pending = self.pending_requests.write().unwrap();
        pending.remove(request_id)
    }

    /// Clears all pending requests (e.g., on session end).
    pub fn clear_pending(&self) {
        let mut pending = self.pending_requests.write().unwrap();
        pending.clear();
    }

    /// Clears all decisions (e.g., on session reset).
    pub fn clear_decisions(&self) {
        let mut decisions = self.decisions.write().unwrap();
        decisions.clear();
    }

    /// Checks if approval is needed based on the command approval mode.
    pub fn needs_approval(&self, tool_name: &str, mode: CommandApproval) -> bool {
        match mode {
            CommandApproval::Off => false,
            CommandApproval::OnFirstUse => {
                // Need approval if no decision recorded yet
                self.get_decision(tool_name).is_none()
            }
            CommandApproval::Always => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_store_records_decisions() {
        let store = ApprovalStore::new();
        store.record_decision("bash", ApprovalDecision::Approved);
        assert_eq!(store.get_decision("bash"), Some(ApprovalDecision::Approved));
    }

    #[test]
    fn approval_store_overwrites_previous_decision() {
        let store = ApprovalStore::new();
        store.record_decision("bash", ApprovalDecision::Approved);
        store.record_decision("bash", ApprovalDecision::Denied);
        assert_eq!(store.get_decision("bash"), Some(ApprovalDecision::Denied));
    }

    #[test]
    fn approval_store_pending_requests() {
        let store = ApprovalStore::new();
        let request = ApprovalRequest {
            request_id: "req-1".to_string(),
            tool_name: "bash".to_string(),
            command: "ls -la".to_string(),
            sandbox_status: SandboxStatus {
                sandbox_available: true,
                affected_paths: vec![".".to_string()],
                network_access: vec![],
            },
        };
        store.add_pending_request(request.clone());
        assert_eq!(store.take_pending_request("req-1"), Some(request));
        assert_eq!(store.take_pending_request("req-1"), None);
    }

    #[test]
    fn needs_approval_off_mode() {
        let store = ApprovalStore::new();
        assert!(!store.needs_approval("bash", CommandApproval::Off));
    }

    #[test]
    fn needs_approval_first_use_mode() {
        let store = ApprovalStore::new();
        // First use needs approval
        assert!(store.needs_approval("bash", CommandApproval::OnFirstUse));
        // After decision, no approval needed
        store.record_decision("bash", ApprovalDecision::Approved);
        assert!(!store.needs_approval("bash", CommandApproval::OnFirstUse));
    }

    #[test]
    fn needs_approval_always_mode() {
        let store = ApprovalStore::new();
        store.record_decision("bash", ApprovalDecision::Approved);
        // Always needs approval regardless of previous decisions
        assert!(store.needs_approval("bash", CommandApproval::Always));
    }
}
