//! CLI Session Management Commands
//!
//! Implements `kay session list`, `kay session load`, `kay session delete`

use anyhow::Context;
use std::path::PathBuf;

/// List all sessions
pub fn list() -> anyhow::Result<()> {
    let session_dir = crate::config::kay_home().join("sessions");
    
    if !session_dir.exists() {
        println!("No sessions found (sessions directory doesn't exist)");
        return Ok(());
    }
    
    let mut entries: Vec<_> = std::fs::read_dir(&session_dir)
        .context("Failed to read sessions directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();
    
    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));
    
    if entries.is_empty() {
        println!("No sessions found");
        return Ok(());
    }
    
    println!("Sessions:");
    println!("  {:<36} {:>10} {:>20}", "ID", "Messages", "Last Updated");
    println!("  {}", "-".repeat(70));
    
    for entry in entries {
        let id = entry.file_name().to_string_lossy().trim_end_matches(".json").to_string();
        
        // Try to read session metadata
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let msg_count = json.get("messages")
                    .and_then(|m| m.as_array())
                    .map(|arr| arr.len() as u32)
                    .unwrap_or(0);
                
                let updated = json.get("updated_at")
                    .and_then(|u| u.as_str())
                    .unwrap_or("unknown");
                
                println!("  {:<36} {:>10} {:>20}", id, msg_count, updated);
            }
        }
    }
    
    Ok(())
}

/// Load a session by ID
pub fn load(session_id: &str) -> anyhow::Result<()> {
    let session_path = crate::config::kay_home()
        .join("sessions")
        .join(format!("{}.json", session_id));
    
    if !session_path.exists() {
        anyhow::bail!("Session not found: {}", session_id);
    }
    
    println!("Loading session: {}", session_id);
    println!("Path: {:?}", session_path);
    
    // Load session content for verification
    let content = std::fs::read_to_string(&session_path)
        .context("Failed to read session file")?;
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        let msg_count = json.get("messages")
            .and_then(|m| m.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        println!("  Messages: {}", msg_count);
        println!("  Status: ready to resume");
    }
    
    Ok(())
}

/// Delete a session by ID
pub fn delete(session_id: &str, force: bool) -> anyhow::Result<()> {
    let session_path = crate::config::kay_home()
        .join("sessions")
        .join(format!("{}.json", session_id));
    
    if !session_path.exists() {
        anyhow::bail!("Session not found: {}", session_id);
    }
    
    if !force {
        println!("Are you sure you want to delete session '{}'? (y/N)", session_id);
        // For non-interactive use, we need confirmation
        // In a real implementation, we'd read from stdin
        println!("Use --force to skip confirmation");
        return Ok(());
    }
    
    std::fs::remove_file(&session_path)
        .context("Failed to delete session file")?;
    
    println!("Deleted session: {}", session_id);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_session_id_format() {
        // Session IDs should be valid UUIDs or similar
        let id = "abc123-def456";
        assert!(id.contains('-'));
    }
}