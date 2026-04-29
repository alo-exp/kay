// Tauri UI — Main App Component
// Phase 10: Multi-session manager integration

import { useReducer, useCallback, useState } from 'react';
import { Channel } from '@tauri-apps/api/core';
import type { IpcAgentEvent } from '../bindings';
import { commands } from '../bindings';
import { SessionView } from './SessionView';
import { PromptInput } from './PromptInput';
import { SessionList } from './SessionList';
import { SettingsPanel } from './SettingsPanel';
import { useModelPicker } from './ModelPicker';
import { CommandApprovalDialog, useApprovalDialog } from './CommandApprovalDialog';


// ── State ─────────────────────────────────────────────────────────────────────

interface SessionState {
  sessionId: string | null;
  events: IpcAgentEvent[];
  totalCostUsd: number;
  totalTokensIn: number;
  totalTokensOut: number;
  status: 'idle' | 'running' | 'complete';
}

type Action =
  | { type: 'SESSION_STARTED'; sessionId: string }
  | { type: 'EVENTS_BATCH'; events: IpcAgentEvent[] }
  | { type: 'SESSION_ENDED' };

const initial: SessionState = {
  sessionId: null,
  events: [],
  totalCostUsd: 0,
  totalTokensIn: 0,
  totalTokensOut: 0,
  status: 'idle',
};

function reducer(state: SessionState, action: Action): SessionState {
  switch (action.type) {
    case 'SESSION_STARTED':
      return { ...initial, sessionId: action.sessionId, status: 'running' };
    case 'EVENTS_BATCH': {
      let { totalCostUsd, totalTokensIn, totalTokensOut } = state;
      for (const ev of action.events) {
        if (ev.type === 'Usage') {
          totalCostUsd   += ev.data.cost_usd;
          totalTokensIn  += ev.data.prompt_tokens;
          totalTokensOut += ev.data.completion_tokens;
        }
        if (ev.type === 'Verification') {
          totalCostUsd += ev.data.cost_usd;
        }
      }
      return {
        ...state,
        events: [...state.events, ...action.events],
        totalCostUsd,
        totalTokensIn,
        totalTokensOut,
      };
    }
    case 'SESSION_ENDED':
      return { ...state, status: 'complete' };
    default:
      return state;
  }
}

// ── Component ─────────────────────────────────────────────────────────────────

export function App() {
  const [state, dispatch] = useReducer(reducer, initial);
  const [showSettings, setShowSettings] = useState(false);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [activeSettingsTab, setActiveSettingsTab] = useState<'session' | 'model' | 'verifier' | 'sandbox'>('session');

  const FLUSH_INTERVAL_MS = 16; // ~60fps batching

  // Command approval state
  const {
    currentRequest,
    approvedTools,
    mode: approvalMode,
    requestApproval,
    handleApprove,
    handleDeny,
  } = useApprovalDialog();

  const handleStart = useCallback(async (prompt: string, persona: string) => {
    const channel = new Channel<IpcAgentEvent>();
    const batch: IpcAgentEvent[] = [];
    let flushTimer: ReturnType<typeof setTimeout> | null = null;

    const scheduleFlush = () => {
      if (flushTimer) return;
      flushTimer = setTimeout(() => {
        flushTimer = null;
        if (batch.length > 0) {
          dispatch({ type: 'EVENTS_BATCH', events: [...batch] });
          batch.length = 0;
        }
      }, FLUSH_INTERVAL_MS);
    };

    channel.onmessage = (ev) => {
      batch.push(ev);
      if (batch.length >= 64) {
        if (flushTimer) { clearTimeout(flushTimer); flushTimer = null; }
        dispatch({ type: 'EVENTS_BATCH', events: [...batch] });
        batch.length = 0;
      } else {
        scheduleFlush();
      }

      // Handle ApprovalRequested events from the backend
      if (ev.type === 'ApprovalRequested') {
        const toolName = (ev.data as { tool_name: string }).tool_name;
        const command = (ev.data as { command: string }).command;
        requestApproval(toolName, command, 'warning', state.sessionId || '');
      }

      // Session ends on terminal events
      if (ev.type === 'TaskComplete' || ev.type === 'Aborted' || ev.type === 'Error') {
        setTimeout(() => dispatch({ type: 'SESSION_ENDED' }), 100);
      }
    };

    try {
      const result = await commands.startSession(prompt, persona, channel);
      if (result.status === 'error') {
        console.error('start_session failed:', result.error);
        return;
      }
      dispatch({ type: 'SESSION_STARTED', sessionId: result.data });
      setSelectedSessionId(result.data);
    } catch (err) {
      console.error('start_session failed:', err);
    }
  }, [state.sessionId, requestApproval]);

  const handleStop = useCallback(async () => {
    if (state.sessionId) {
      await commands.stopSession(state.sessionId).catch(console.error);
    }
  }, [state.sessionId]);

  const handleSessionSelect = useCallback((sessionId: string) => {
    setSelectedSessionId(sessionId);
  }, []);

  return (
    <div className="app-container">
      {/* Command Approval Dialog (overlay) */}
      <CommandApprovalDialog
        request={currentRequest}
        onApprove={handleApprove}
        onDeny={handleDeny}
        mode={approvalMode === 'always' ? 'always' : 'on_first_use'}
        approvedTools={approvedTools}
      />

      {/* Settings Panel (overlay) */}
      {showSettings && (
        <SettingsPanel
          projectPath={window.location.pathname}
          onClose={() => setShowSettings(false)}
          activeTab={activeSettingsTab}
          onTabChange={setActiveSettingsTab}
        />
      )}

      {/* Main layout */}
      <div className="app-layout">
        {/* Sidebar - Session List */}
        <div className="app-sidebar">
          <SessionList
            onSessionSelect={handleSessionSelect}
            selectedSessionId={selectedSessionId}
          />
        </div>

        {/* Main content area */}
        <div className="app-main">
          {state.status !== 'idle' && selectedSessionId === state.sessionId ? (
            <SessionView
              events={state.events}
              totalCostUsd={state.totalCostUsd}
              totalTokensIn={state.totalTokensIn}
              totalTokensOut={state.totalTokensOut}
              status={state.status}
            />
          ) : (
            <div className="app-welcome">
              <h2>Welcome to Kay</h2>
              <p>Select a session from the sidebar or start a new one.</p>
            </div>
          )}

          {/* Prompt Input - always visible at bottom */}
          <div className="app-prompt-area">
            <PromptInput
              onStart={handleStart}
              onStop={handleStop}
              status={state.status}
            />
          </div>
        </div>
      </div>

      {/* Header bar with settings toggle */}
      <div className="app-header-bar">
        <span className="app-title">Kay</span>
        <div className="app-header-actions">
          <button
            className="header-btn"
            onClick={() => setShowSettings(true)}
            title="Settings"
          >
            ⚙️
          </button>
        </div>
      </div>
    </div>
  );
}
