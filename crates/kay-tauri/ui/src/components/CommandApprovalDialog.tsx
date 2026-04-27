// Tauri UI — Command Approval Dialog Component
// Phase 10 Wave 5/7: Interactive sandbox confirmation
// Success criteria: approval dialog shows on first tool call, subsequent calls skip

import { useEffect, useRef, useState } from "react";

export interface ApprovalRequest {
  tool_name: string;
  command: string;
  sandbox_status: "safe" | "warning" | "violation";
  session_id: string;
}

interface CommandApprovalDialogProps {
  request: ApprovalRequest | null;
  onApprove: () => void;
  onDeny: () => void;
  mode: "on_first_use" | "always";
  // Track which tools have been approved in this session
  approvedTools: Set<string>;
}

export function CommandApprovalDialog({
  request,
  onApprove,
  onDeny,
  mode,
  approvedTools,
}: CommandApprovalDialogProps) {
  const [countdown, setCountdown] = useState(10); // Auto-deny after 10 seconds
  const dialogRef = useRef<HTMLDivElement>(null);

  // Auto-approve/deny countdown
  useEffect(() => {
    if (!request) {
      setCountdown(10);
      return;
    }

    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          onDeny();
          return 10;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [request, onDeny]);

  // Focus trap and keyboard handling
  useEffect(() => {
    if (!request || !dialogRef.current) return;

    // Focus the dialog when it appears
    dialogRef.current.focus();

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        onApprove();
      } else if (e.key === "Escape") {
        e.preventDefault();
        onDeny();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [request, onApprove, onDeny]);

  if (!request) return null;

  const isFirstUse = !approvedTools.has(request.tool_name);
  const showDialog = mode === "always" || (mode === "on_first_use" && isFirstUse);

  if (!showDialog) {
    // Auto-approve if already approved in this session
    onApprove();
    return null;
  }

  function getStatusColor(status: ApprovalRequest["sandbox_status"]): string {
    switch (status) {
      case "safe":
        return "#22c55e";
      case "warning":
        return "#eab308";
      case "violation":
        return "#ef4444";
      default:
        return "#6b7280";
    }
  }

  function getStatusText(status: ApprovalRequest["sandbox_status"]): string {
    switch (status) {
      case "safe":
        return "Safe";
      case "warning":
        return "Warning";
      case "violation":
        return "Sandbox Violation Risk";
      default:
        return "Unknown";
    }
  }

  function truncateCommand(cmd: string, maxLen: number = 200): string {
    if (cmd.length <= maxLen) return cmd;
    return cmd.substring(0, maxLen) + "...";
  }

  return (
    <div className="approval-overlay" ref={dialogRef} tabIndex={-1}>
      <div className="approval-dialog" role="dialog" aria-modal="true">
        <div className="approval-header">
          <h2>Command Approval Required</h2>
          <span
            className="approval-status-badge"
            style={{ backgroundColor: getStatusColor(request.sandbox_status) }}
          >
            {getStatusText(request.sandbox_status)}
          </span>
        </div>

        <div className="approval-content">
          <div className="approval-tool-info">
            <span className="approval-label">Tool:</span>
            <code className="approval-tool-name">{request.tool_name}</code>
            {isFirstUse && <span className="approval-first-use">(First use this session)</span>}
          </div>

          <div className="approval-command-section">
            <span className="approval-label">Command:</span>
            <pre className="approval-command">
              {truncateCommand(request.command)}
            </pre>
          </div>

          {request.sandbox_status !== "safe" && (
            <div
              className={`approval-warning ${
                request.sandbox_status === "violation" ? "danger" : "caution"
              }`}
            >
              {request.sandbox_status === "violation"
                ? "⚠️ This command may violate sandbox restrictions and could be blocked."
                : "⚡ This command accesses sensitive resources. Verify the command is safe."}
            </div>
          )}
        </div>

        <div className="approval-footer">
          <div className="approval-countdown">
            Auto-deny in {countdown}s
            <div className="countdown-bar">
              <div
                className="countdown-progress"
                style={{ width: `${(countdown / 10) * 100}%` }}
              />
            </div>
          </div>

          <div className="approval-actions">
            <button
              className="approval-btn deny"
              onClick={onDeny}
              autoFocus
            >
              Deny (Esc)
            </button>
            <button
              className="approval-btn approve"
              onClick={onApprove}
            >
              Approve (Enter)
            </button>
          </div>
        </div>

        <div className="approval-help">
          Press Enter to approve • Escape to deny
        </div>
      </div>
    </div>
  );
}

// Hook to manage approval state
export function useApprovalDialog() {
  const [currentRequest, setCurrentRequest] = useState<ApprovalRequest | null>(null);
  const [approvedTools, setApprovedTools] = useState<Set<string>>(new Set());
  const [mode, setMode] = useState<"on_first_use" | "always">("on_first_use");

  const [pendingResolve, setPendingResolve] = useState<((v: boolean) => void) | null>(null);

  function requestApproval(
    tool_name: string,
    command: string,
    sandbox_status: ApprovalRequest["sandbox_status"],
    session_id: string
  ): Promise<boolean> {
    return new Promise((resolve) => {
      setCurrentRequest({
        tool_name,
        command,
        sandbox_status,
        session_id,
      });
      setPendingResolve(() => resolve);
    });
  }

  function handleApprove() {
    if (currentRequest) {
      setApprovedTools((prev) => new Set([...prev, currentRequest.tool_name]));
    }
    setCurrentRequest(null);
    if (pendingResolve) {
      pendingResolve(true);
      setPendingResolve(null);
    }
  }

  function handleDeny() {
    setCurrentRequest(null);
    if (pendingResolve) {
      pendingResolve(false);
      setPendingResolve(null);
    }
  }

  function clearApprovedTools() {
    setApprovedTools(new Set());
  }

  return {
    currentRequest,
    approvedTools,
    mode,
    setMode,
    requestApproval,
    handleApprove,
    handleDeny,
    clearApprovedTools,
  };
}
