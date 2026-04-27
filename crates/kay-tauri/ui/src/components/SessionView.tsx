import type { IpcAgentEvent } from '../bindings';
import { AgentTrace } from './AgentTrace';
import { CostMeter } from './CostMeter';
import { ToolCallTimeline } from './ToolCallTimeline';

interface Props {
  events: IpcAgentEvent[];
  totalCostUsd: number;
  totalTokensIn: number;
  totalTokensOut: number;
  status: 'running' | 'complete';
}

export function SessionView({ events, totalCostUsd, totalTokensIn, totalTokensOut, status }: Props) {
  const toolCallEvents = events.filter(e => e.type === 'ToolCallComplete') as Extract<IpcAgentEvent, { type: 'ToolCallComplete' }>[];

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
      background: 'var(--bg-primary)',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '8px 16px',
        borderBottom: '1px solid var(--border)',
        background: 'var(--bg-secondary)',
        gap: 12,
        flexShrink: 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{
            width: 8, height: 8, borderRadius: '50%',
            background: status === 'running' ? 'var(--success)' : 'var(--text-muted)',
            flexShrink: 0,
          }} />
          <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
            {status === 'running' ? 'Running' : 'Complete'}
          </span>
        </div>
        <CostMeter
          totalCostUsd={totalCostUsd}
          totalTokensIn={totalTokensIn}
          totalTokensOut={totalTokensOut}
        />
      </div>

      {/* Timeline */}
      {toolCallEvents.length > 0 && (
        <ToolCallTimeline events={toolCallEvents} />
      )}

      {/* Trace */}
      <AgentTrace events={events} />
    </div>
  );
}
