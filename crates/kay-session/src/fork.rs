use uuid::Uuid;
use crate::error::SessionError;
use crate::index::Session;
use crate::store::SessionStore;

/// Fork a session: create a child session with `parent_id` = parent's UUID.
///
/// The child inherits `persona`, `model`, and `cwd` from the parent.
/// It starts with a fresh empty transcript and `status = "active"`.
/// `parent_id` satisfies SESS-04 (reserved for Phase 10 multi-agent
/// orchestration); FK is ON DELETE SET NULL.
pub fn fork_session(
    store: &SessionStore,
    parent_id: &Uuid,
) -> Result<Session, SessionError> {
    unimplemented!("W-5 GREEN: SELECT parent row, INSERT child with parent_id FK")
}
