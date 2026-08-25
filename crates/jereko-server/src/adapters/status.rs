use jereko_core::SessionStatus;

pub fn session_status_str(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Idle => "idle",
        SessionStatus::Completed => "completed",
        SessionStatus::Error => "error",
    }
}
