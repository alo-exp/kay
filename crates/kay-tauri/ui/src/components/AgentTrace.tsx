import { useRef, useEffect, useState, useCallback } from 'react';
import type { IpcAgentEvent } from '../bindings';
import { EventRow } from './EventRow';

interface Props {
  events: IpcAgentEvent[];
}

export function AgentTrace({ events }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [userScrolled, setUserScrolled] = useState(false);

  // Auto-scroll unless user has scrolled up
  useEffect(() => {
    if (!userScrolled) {
      bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [events, userScrolled]);

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
    setUserScrolled(!atBottom);
  }, []);

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      style={{
        flex: 1,
        overflowY: 'auto',
        padding: '12px 16px',
        display: 'flex',
        flexDirection: 'column',
        gap: 4,
      }}
    >
      {events.map((ev, i) => (
        <EventRow key={i} event={ev} allEvents={events} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
}
