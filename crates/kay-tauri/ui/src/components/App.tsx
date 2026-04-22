import { useReducer, useCallback } from 'react';
import { Channel } from '@tauri-apps/api/core';
import type { IpcAgentEvent } from '../bindings';
import { commands } from '../bindings';
import { SessionView } from './SessionView';
import { PromptInput } from './PromptInput';

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
      }, 16);
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

      // Session ends on terminal events
      if (ev.type === 'TaskComplete' || ev.type === 'Aborted' || ev.type === 'Error') {
        setTimeout(() => dispatch({ type: 'SESSION_ENDED' }), 100);
      }
    };

    try {
      const sessionId = await commands.startSession(prompt, persona, channel);
      dispatch({ type: 'SESSION_STARTED', sessionId });
    } catch (err) {
      console.error('start_session failed:', err);
    }
  }, []);

  const handleStop = useCallback(async () => {
    if (state.sessionId) {
      await commands.stopSession(state.sessionId).catch(console.error);
    }
  }, [state.sessionId]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {state.status !== 'idle' && (
        <SessionView
          events={state.events}
          totalCostUsd={state.totalCostUsd}
          totalTokensIn={state.totalTokensIn}
          totalTokensOut={state.totalTokensOut}
          status={state.status}
        />
      )}
      <PromptInput
        onStart={handleStart}
        onStop={handleStop}
        status={state.status}
      />
    </div>
  );
}
