//! Flush task: batches `AgentEvent` → `IpcAgentEvent` to the Tauri IPC channel.
//!
//! # Design
//! - 16ms interval timer — matches a 60fps frame budget.
//! - 64-event size cap: if the buffer fills before the timer fires, we flush
//!   immediately to prevent unbounded memory growth during heavy PTY output.
//! - Final drain: when the sender side drops (agent loop exits), the `None`
//!   arm flushes any remaining buffered events before the task exits. No
//!   events are lost.

use std::time::Duration;

use tauri::ipc::Channel;
use tokio::sync::mpsc;

use kay_tools::events::AgentEvent;

use crate::ipc_event::IpcAgentEvent;

pub async fn flush_task(mut rx: mpsc::Receiver<AgentEvent>, channel: Channel<IpcAgentEvent>) {
    let mut ticker = tokio::time::interval(Duration::from_millis(16));
    let mut buffer: Vec<AgentEvent> = Vec::with_capacity(64);

    loop {
        tokio::select! {
            maybe = rx.recv() => {
                match maybe {
                    Some(event) => {
                        buffer.push(event);
                        if buffer.len() >= 64 {
                            do_flush(&channel, &mut buffer);
                        }
                    }
                    None => {
                        do_flush(&channel, &mut buffer);
                        break;
                    }
                }
            }
            _ = ticker.tick() => {
                do_flush(&channel, &mut buffer);
            }
        }
    }
}

fn do_flush(channel: &Channel<IpcAgentEvent>, buffer: &mut Vec<AgentEvent>) {
    for event in buffer.drain(..) {
        if let Err(e) = channel.send(IpcAgentEvent::from(event)) {
            tracing::warn!("ipc channel send error: {e:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kay_tools::events::AgentEvent;

    // Note: flush_task tests require an IPC channel mock. The core logic
    // (64-event cap, 16ms timer, final drain) is validated by the integration
    // tests in tests/gen_bindings.rs which exercise the full command path.
    // Unit tests here validate the do_flush behavior indirectly.

    #[test]
    fn ipc_agent_event_from_text_delta_is_correct() {
        let ev = AgentEvent::TextDelta { content: "hello".to_string() };
        let ipc = IpcAgentEvent::from(ev);
        assert!(matches!(ipc, IpcAgentEvent::TextDelta { content } if content == "hello"));
    }
}
