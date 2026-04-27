// Tauri UI — Session List Component
// Phase 10 Wave 7: Session management UI
// Success criteria: list sessions, spawn/pause/resume/fork/kill from GUI

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  id: string;
  status: "Running" | "Paused" | "Completed" | "Failed" | "Killed";
  created_at: number; // Unix timestamp
  last_active: number; // Unix timestamp
  persona: string;
  prompt_preview: string;
}

interface SessionListProps {
  onSessionSelect?: (sessionId: string) => void;
  selectedSessionId?: string | null;
}

export function SessionList({ onSessionSelect, selectedSessionId }: SessionListProps) {
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  // Load sessions on mount and poll every 5 seconds
  useEffect(() => {
    loadSessions();
    const interval = setInterval(loadSessions, 5000);
    return () => clearInterval(interval);
  }, []);

  async function loadSessions() {
    try {
      const result = await invoke<SessionInfo[]>("list_sessions");
      // Sort by last_active descending
      const sorted = [...result].sort((a, b) => b.last_active - a.last_active);
      setSessions(sorted);
      setError(null);
    } catch (e) {
      console.error("Failed to load sessions:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handlePause(sessionId: string) {
    setActionLoading(sessionId);
    try {
      await invoke("pause_session", { sessionId });
      await loadSessions();
    } catch (e) {
      setError(`Failed to pause session: ${e}`);
    } finally {
      setActionLoading(null);
    }
  }

  async function handleResume(sessionId: string) {
    setActionLoading(sessionId);
    try {
      await invoke("resume_session", { sessionId });
      await loadSessions();
    } catch (e) {
      setError(`Failed to resume session: ${e}`);
    } finally {
      setActionLoading(null);
    }
  }

  async function handleFork(sessionId: string) {
    setActionLoading(sessionId);
    try {
      await invoke<string>("fork_session", { sessionId, persona: null });
      await loadSessions();
    } catch (e) {
      setError(`Failed to fork session: ${e}`);
    } finally {
      setActionLoading(null);
    }
  }

  async function handleKill(sessionId: string) {
    if (!confirm("Are you sure you want to kill this session?")) return;
    setActionLoading(sessionId);
    try {
      await invoke("kill_session", { sessionId });
      await loadSessions();
    } catch (e) {
      setError(`Failed to kill session: ${e}`);
    } finally {
      setActionLoading(null);
    }
  }

  function formatTimestamp(ts: number): string {
    const date = new Date(ts * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return date.toLocaleDateString();
  }

  function getStatusColor(status: SessionInfo["status"]): string {
    switch (status) {
      case "Running":
        return "#22c55e"; // green
      case "Paused":
        return "#eab308"; // yellow
      case "Completed":
        return "#6b7280"; // gray
      case "Failed":
        return "#ef4444"; // red
      case "Killed":
        return "#dc2626"; // dark red
      default:
        return "#6b7280";
    }
  }

  if (loading && sessions.length === 0) {
    return (
      <div className="session-list">
        <div className="session-list-loading">Loading sessions...</div>
      </div>
    );
  }

  return (
    <div className="session-list">
      {error && <div className="session-list-error">{error}</div>}

      <div className="session-list-header">
        <h3>Sessions ({sessions.length})</h3>
        <button className="session-list-refresh" onClick={loadSessions} title="Refresh">
          ↻
        </button>
      </div>

      {sessions.length === 0 ? (
        <div className="session-list-empty">
          No sessions yet. Start a new session to begin.
        </div>
      ) : (
        <div className="session-list-items">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={`session-item ${selectedSessionId === session.id ? "selected" : ""}`}
              onClick={() => onSessionSelect?.(session.id)}
            >
              <div className="session-item-header">
                <span
                  className="session-status-dot"
                  style={{ backgroundColor: getStatusColor(session.status) }}
                  title={session.status}
                />
                <span className="session-persona">
                  {session.persona || "default"}
                </span>
                <span className="session-status">{session.status}</span>
              </div>

              <div className="session-item-preview">
                {session.prompt_preview || "No prompt preview"}
              </div>

              <div className="session-item-meta">
                <span className="session-time">Active: {formatTimestamp(session.last_active)}</span>
              </div>

              <div className="session-item-actions">
                {session.status === "Running" && (
                  <button
                    className="session-action-btn pause"
                    onClick={(e) => {
                      e.stopPropagation();
                      handlePause(session.id);
                    }}
                    disabled={actionLoading === session.id}
                    title="Pause"
                  >
                    ⏸
                  </button>
                )}
                {session.status === "Paused" && (
                  <button
                    className="session-action-btn resume"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleResume(session.id);
                    }}
                    disabled={actionLoading === session.id}
                    title="Resume"
                  >
                    ▶
                  </button>
                )}
                {(session.status === "Running" || session.status === "Paused") && (
                  <>
                    <button
                      className="session-action-btn fork"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleFork(session.id);
                      }}
                      disabled={actionLoading === session.id}
                      title="Fork"
                    >
                      ⎘
                    </button>
                    <button
                      className="session-action-btn kill"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleKill(session.id);
                      }}
                      disabled={actionLoading === session.id}
                      title="Kill"
                    >
                      ✕
                    </button>
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
