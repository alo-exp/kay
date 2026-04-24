import { useState, useCallback, useRef } from 'react';

interface Props {
  onStart: (prompt: string, persona: string) => void;
  onStop: () => void;
  status: 'idle' | 'running' | 'complete';
}

const PERSONAS = ['forge', 'sage', 'muse'] as const;

export function PromptInput({ onStart, onStop, status }: Props) {
  const [prompt, setPrompt] = useState('');
  const [persona, setPersona] = useState<string>('forge');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = useCallback(() => {
    if (!prompt.trim() || status === 'running') return;
    onStart(prompt.trim(), persona);
    setPrompt('');
  }, [prompt, persona, status, onStart]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
    if (e.key === 'Escape' && status === 'running') {
      onStop();
    }
  }, [handleSubmit, status, onStop]);

  return (
    <div style={{
      borderTop: '1px solid var(--border)',
      background: 'var(--bg-secondary)',
      padding: 12,
      display: 'flex',
      flexDirection: 'column',
      gap: 8,
      flexShrink: 0,
    }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'flex-start' }}>
        <textarea
          ref={textareaRef}
          value={prompt}
          onChange={e => setPrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter a task… (Cmd+Enter to run, Esc to stop)"
          disabled={status === 'running'}
          rows={3}
          style={{
            flex: 1,
            resize: 'vertical',
            minHeight: 72,
            padding: '8px 12px',
            background: 'var(--bg-primary)',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius-md)',
            color: 'var(--text-primary)',
            fontFamily: 'var(--font-mono)',
            fontSize: 13,
            outline: 'none',
            transition: 'border-color 0.15s',
          }}
          onFocus={e => { e.target.style.borderColor = 'var(--accent)'; }}
          onBlur={e => { e.target.style.borderColor = 'var(--border)'; }}
        />
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <select
            value={persona}
            onChange={e => setPersona(e.target.value)}
            disabled={status === 'running'}
            style={{
              padding: '6px 10px',
              background: 'var(--bg-surface)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-md)',
              color: 'var(--text-primary)',
              fontFamily: 'var(--font-ui)',
              fontSize: 13,
              cursor: 'pointer',
            }}
          >
            {PERSONAS.map(p => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
          {status === 'running' ? (
            <button
              onClick={onStop}
              style={{
                padding: '8px 16px',
                background: 'var(--error)',
                color: '#fff',
                borderRadius: 'var(--radius-md)',
                fontSize: 13,
                fontWeight: 600,
              }}
            >
              Stop (Esc)
            </button>
          ) : (
            <button
              onClick={handleSubmit}
              disabled={!prompt.trim()}
              style={{
                padding: '8px 16px',
                background: prompt.trim() ? 'var(--accent-dim)' : 'var(--bg-hover)',
                color: prompt.trim() ? '#fff' : 'var(--text-muted)',
                borderRadius: 'var(--radius-md)',
                fontSize: 13,
                fontWeight: 600,
                transition: 'background 0.15s',
              }}
            >
              Run ⌘↵
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
