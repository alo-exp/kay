//! Token Budget Management for Kay
//!
//! Tracks and manages token usage across sessions.

use serde::{Deserialize, Serialize};

/// Token usage for a single request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Budget configuration
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum tokens per request (0 = unlimited)
    pub max_tokens_per_request: u32,
    /// Maximum tokens per session (0 = unlimited)
    pub max_tokens_per_session: u32,
    /// Warning threshold (percentage)
    pub warning_threshold: f32,
    /// Budget exhausted action
    pub on_exhausted: BudgetAction,
}

/// Action to take when budget is exhausted
#[derive(Debug, Clone, Copy)]
pub enum BudgetAction {
    /// Fail the request
    Fail,
    /// Compact/summarize context and continue
    Compact,
    /// Stop processing
    Stop,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_tokens_per_request: 0,
            max_tokens_per_session: 0,
            warning_threshold: 0.9,
            on_exhausted: BudgetAction::Compact,
        }
    }
}

/// Token budget manager
pub struct TokenBudgetManager {
    budget: TokenBudget,
    total_used: u32,
    request_count: u32,
}

impl TokenBudgetManager {
    pub fn new(budget: TokenBudget) -> Self {
        Self {
            budget,
            total_used: 0,
            request_count: 0,
        }
    }
    
    /// Check if a request fits within the budget
    pub fn can_request(&self, usage: &TokenUsage) -> bool {
        // Check per-request limit
        if self.budget.max_tokens_per_request > 0 
            && usage.total_tokens > self.budget.max_tokens_per_request {
            return false;
        }
        
        // Check per-session limit
        if self.budget.max_tokens_per_session > 0 {
            let projected_total = self.total_used + usage.total_tokens;
            if projected_total > self.budget.max_tokens_per_session {
                return false;
            }
        }
        
        true
    }
    
    /// Record usage and update budget
    pub fn record(&mut self, usage: &TokenUsage) {
        self.total_used += usage.total_tokens;
        self.request_count += 1;
    }
    
    /// Check if we're past the warning threshold
    pub fn past_warning_threshold(&self) -> bool {
        if self.budget.max_tokens_per_session == 0 {
            return false;
        }
        
        let ratio = self.total_used as f32 / self.budget.max_tokens_per_session as f32;
        ratio >= self.budget.warning_threshold
    }
    
    /// Get remaining budget
    pub fn remaining(&self) -> Option<u32> {
        if self.budget.max_tokens_per_session == 0 {
            return None;
        }
        
        Some(self.budget.max_tokens_per_session.saturating_sub(self.total_used))
    }
    
    /// Get total used tokens
    pub fn total_used(&self) -> u32 {
        self.total_used
    }
    
    /// Get request count
    pub fn request_count(&self) -> u32 {
        self.request_count
    }
    
    /// Check if budget is exhausted
    pub fn is_exhausted(&self) -> bool {
        if self.budget.max_tokens_per_session == 0 {
            return false;
        }
        self.total_used >= self.budget.max_tokens_per_session
    }
}

/// Estimate tokens in text (approximate)
pub fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: ~4 chars per token for English
    ((text.len() as f32) / 4.0).ceil() as u32
}

/// Estimate tokens for messages
pub fn estimate_messages_tokens(messages: &[impl AsRef<str>]) -> u32 {
    messages.iter().map(|m| estimate_tokens(m.as_ref())).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_can_request_within_budget() {
        let budget = TokenBudget {
            max_tokens_per_request: 1000,
            max_tokens_per_session: 5000,
            warning_threshold: 0.8,
            on_exhausted: BudgetAction::Fail,
        };
        let manager = TokenBudgetManager::new(budget);
        
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 200,
            total_tokens: 300,
        };
        
        assert!(manager.can_request(&usage));
    }
    
    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("hello world") > 0);
    }
}
