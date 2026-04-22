import type { IpcAgentEvent } from '../bindings';

type ToolCallCompleteEvent = Extract<IpcAgentEvent, { type: 'ToolCallComplete' }>;

interface Props {
  events: ToolCallCompleteEvent[];
}

const TOOL_COLORS: Record<string, string> = {
  edit_file:    '#bc8cff',
  write_file:   '#79c0ff',
  read_file:    '#56d364',
  execute_commands: '#ffa657',
  task_complete: '#3fb950',
};

export function ToolCallTimeline({ events }: Props) {
  return (
    <div style={{
      display: 'flex',
      gap: 4,
      padding: '6px 16px',
      borderBottom: '1px solid var(--border)',
      overflowX: 'auto',
      background: 'var(--bg-secondary)',
      flexShrink: 0,
    }}>
      {events.map((ev, i) => {
        const color = TOOL_COLORS[ev.data.name] ?? 'var(--text-muted)';
        return (
          <div
            key={i}
            title={ev.data.name}
            style={{
              width: 12,
              height: 12,
              borderRadius: 2,
              background: color,
              flexShrink: 0,
            }}
          />
        );
      })}
    </div>
  );
}
