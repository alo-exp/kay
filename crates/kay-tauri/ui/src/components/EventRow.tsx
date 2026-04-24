import { lazy, Suspense } from 'react';
import type { IpcAgentEvent } from '../bindings';

const DiffViewer = lazy(() => import('./DiffViewer').then(m => ({ default: m.DiffViewer })));

interface Props {
  event: IpcAgentEvent;
  allEvents: IpcAgentEvent[];
}

export function EventRow({ event, allEvents: _allEvents }: Props) {
  switch (event.type) {
    case 'TextDelta':
      return <TextRow content={event.data.content} />;

    case 'ToolCallStart':
    case 'ToolCallDelta':
      return null; // buffered; shown when ToolCallComplete arrives

    case 'ToolCallComplete':
      return (
        <ToolCallCard
          id={event.data.id}
          name={event.data.name}
          args={event.data.arguments}
        />
      );

    case 'ToolCallMalformed':
      return <ErrorRow message={`Malformed tool call: ${event.data.raw}`} />;

    case 'ToolOutput':
      return null; // streamed into active ToolCallCard

    case 'TaskComplete':
      return <TaskCompleteRow verified={event.data.verified} outcome={event.data.outcome} />;

    case 'ImageRead':
      return <ImageRow dataUrl={event.data.data_url} path={event.data.path} />;

    case 'SandboxViolation':
      return <SandboxAlertRow
        toolName={event.data.tool_name}
        resource={event.data.resource}
        policyRule={event.data.policy_rule}
      />;

    case 'Paused':
      return <PausedRow />;

    case 'Aborted':
      return <AbortedRow reason={event.data.reason} />;

    case 'Usage':
      return <UsageRow
        promptTokens={event.data.prompt_tokens}
        completionTokens={event.data.completion_tokens}
        costUsd={event.data.cost_usd}
      />;

    case 'Retry':
      return <RetryRow attempt={event.data.attempt} reason={event.data.reason} delayMs={event.data.delay_ms} />;

    case 'Error':
      return <ErrorRow message={event.data.message} />;

    case 'ContextTruncated':
      return <ContextTruncatedRow dropped={event.data.dropped_symbols} budget={event.data.budget_tokens} />;

    case 'IndexProgress':
      return null; // rendered in status bar

    case 'Verification':
      return <VerificationCard
        criticRole={event.data.critic_role}
        verdict={event.data.verdict}
        reason={event.data.reason}
        costUsd={event.data.cost_usd}
      />;

    case 'VerifierDisabled':
      return <VerifierDisabledRow reason={event.data.reason} costUsd={event.data.cost_usd} />;

    case 'Unknown':
      return <UnknownEventRow eventType={event.data.event_type} />;

    default: {
      // Compile-time exhaustiveness check: if a new IpcAgentEvent variant is added
      // but not handled above, TypeScript will error on the never assignment.
      const _never: never = event as never;
      void _never; // suppress "unused variable" — keep the assertion
      return <UnknownEventRow eventType={(event as { type: string }).type} />;
    }
  }
}

// ── Row components ─────────────────────────────────────────────────────────────

function TextRow({ content }: { content: string }) {
  return (
    <div style={{
      fontFamily: 'var(--font-mono)',
      fontSize: 13,
      color: 'var(--text-primary)',
      whiteSpace: 'pre-wrap',
      wordBreak: 'break-word',
    }}>
      {content}
    </div>
  );
}

function ToolCallCard({ id: _id, name, args }: { id: string; name: string; args: unknown }) {
  const isEdit = name === 'edit_file' || name === 'write_file';
  const argsObj = args as Record<string, string> | null;

  return (
    <div style={{
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-md)',
      overflow: 'hidden',
      marginBlock: 4,
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '6px 12px',
        background: 'var(--bg-surface)',
        borderBottom: isEdit ? '1px solid var(--border)' : undefined,
      }}>
        <span style={{ color: 'var(--tool-call)', fontFamily: 'var(--font-mono)', fontSize: 12 }}>
          {name}
        </span>
      </div>
      {isEdit && argsObj && (
        <Suspense fallback={<div style={{ padding: 8, color: 'var(--text-muted)', fontSize: 12 }}>Loading diff…</div>}>
          <DiffViewer original={argsObj.original ?? ''} modified={argsObj.content ?? argsObj.new_content ?? ''} />
        </Suspense>
      )}
    </div>
  );
}

function VerificationCard({ criticRole, verdict, reason, costUsd }: {
  criticRole: string; verdict: string; reason: string; costUsd: number;
}) {
  const pass = verdict === 'pass';
  return (
    <div style={{
      border: `1px solid ${pass ? 'var(--success)' : 'var(--error)'}`,
      borderRadius: 'var(--radius-md)',
      padding: '8px 12px',
      marginBlock: 4,
      background: 'var(--bg-surface)',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <span style={{ fontSize: 14 }}>{pass ? '✓' : '✗'}</span>
        <span style={{
          color: pass ? 'var(--success)' : 'var(--error)',
          fontFamily: 'var(--font-mono)',
          fontSize: 12,
          fontWeight: 600,
        }}>
          {criticRole}
        </span>
        <span style={{ color: 'var(--text-muted)', fontSize: 11 }}>${costUsd.toFixed(4)}</span>
      </div>
      <div style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{reason}</div>
    </div>
  );
}

function TaskCompleteRow({ verified, outcome }: { verified: boolean; outcome: unknown }) {
  const outcomeObj = outcome as { Pass?: unknown; Fail?: { reason: string }; Pending?: { reason: string } };
  return (
    <div style={{
      padding: '6px 12px',
      borderRadius: 'var(--radius-md)',
      background: verified ? 'rgba(63,185,80,0.1)' : 'rgba(248,81,73,0.1)',
      border: `1px solid ${verified ? 'var(--success)' : 'var(--error)'}`,
      color: verified ? 'var(--success)' : 'var(--error)',
      fontSize: 12,
      fontFamily: 'var(--font-mono)',
      marginBlock: 4,
    }}>
      {verified ? '✓ Task complete' : `✗ Task failed${outcomeObj.Fail ? ': ' + outcomeObj.Fail.reason : ''}`}
    </div>
  );
}

function ImageRow({ dataUrl, path }: { dataUrl: string; path: string }) {
  return (
    <div style={{ marginBlock: 4 }}>
      <div style={{ color: 'var(--text-muted)', fontSize: 11, marginBottom: 4 }}>{path}</div>
      <img
        src={dataUrl}
        alt={path}
        style={{ maxWidth: '100%', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border)' }}
      />
    </div>
  );
}

function SandboxAlertRow({ toolName, resource, policyRule }: {
  toolName: string; resource: string; policyRule: string;
}) {
  return (
    <div style={{
      padding: '6px 12px',
      borderRadius: 'var(--radius-md)',
      background: 'rgba(248,81,73,0.1)',
      border: '1px solid var(--error)',
      color: 'var(--error)',
      fontSize: 12,
      fontFamily: 'var(--font-mono)',
      marginBlock: 4,
    }}>
      🛡 Sandbox violation — {toolName}: {resource} [{policyRule}]
    </div>
  );
}

function PausedRow() {
  return (
    <div style={{ color: 'var(--text-muted)', fontSize: 12, fontStyle: 'italic', padding: '4px 0' }}>
      ⏸ Paused
    </div>
  );
}

function AbortedRow({ reason }: { reason: string }) {
  return (
    <div style={{ color: 'var(--warning)', fontSize: 12, fontFamily: 'var(--font-mono)', padding: '4px 0' }}>
      ⏹ Aborted: {reason}
    </div>
  );
}

function UsageRow({ promptTokens, completionTokens, costUsd }: {
  promptTokens: number; completionTokens: number; costUsd: number;
}) {
  return (
    <div style={{
      color: 'var(--text-muted)',
      fontSize: 11,
      fontFamily: 'var(--font-mono)',
      padding: '2px 0',
    }}>
      {promptTokens}↑ {completionTokens}↓ ${costUsd.toFixed(4)}
    </div>
  );
}

function RetryRow({ attempt, reason, delayMs }: { attempt: number; reason: string; delayMs: number }) {
  return (
    <div style={{ color: 'var(--warning)', fontSize: 12, fontFamily: 'var(--font-mono)', padding: '2px 0' }}>
      ↻ Retry #{attempt} ({reason}, {delayMs}ms)
    </div>
  );
}

function ErrorRow({ message }: { message: string }) {
  return (
    <div style={{
      color: 'var(--error)',
      fontSize: 12,
      fontFamily: 'var(--font-mono)',
      padding: '4px 8px',
      background: 'rgba(248,81,73,0.08)',
      borderRadius: 'var(--radius-sm)',
      marginBlock: 2,
    }}>
      ✗ {message}
    </div>
  );
}

function ContextTruncatedRow({ dropped, budget }: { dropped: number; budget: number }) {
  return (
    <div style={{ color: 'var(--warning)', fontSize: 12, padding: '2px 0' }}>
      ⚠ Context truncated: {dropped} symbols dropped (budget: {budget} tokens)
    </div>
  );
}

function VerifierDisabledRow({ reason, costUsd }: { reason: string; costUsd: number }) {
  return (
    <div style={{ color: 'var(--text-muted)', fontSize: 12, padding: '2px 0' }}>
      Verifier disabled: {reason} (spent ${costUsd.toFixed(4)})
    </div>
  );
}

function UnknownEventRow({ eventType }: { eventType: string }) {
  return (
    <div style={{ color: 'var(--text-muted)', fontSize: 11, fontFamily: 'var(--font-mono)', padding: '2px 0' }}>
      [unknown: {eventType}]
    </div>
  );
}
