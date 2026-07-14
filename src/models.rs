use std::time::Instant;

#[derive(Clone)]
pub struct PeerState {
    pub name: String,
    pub last_seen: Instant,
    pub last_spoken: Instant,
    pub volume: f32,
    pub ping_ms: u32,
}
