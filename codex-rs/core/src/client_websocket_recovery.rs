use std::time::Duration;
use std::time::Instant;

const COOLDOWNS: [Duration; 4] = [
    Duration::from_secs(60),
    Duration::from_secs(5 * 60),
    Duration::from_secs(15 * 60),
    Duration::from_secs(30 * 60),
];

/// Session state only: no timer, background probe, or cancellation of active requests.
#[derive(Debug, Default)]
pub(super) struct WebsocketRecovery {
    cooldown_index: usize,
    retry_at: Option<Instant>,
    generation: u64,
}

impl WebsocketRecovery {
    pub(super) fn enabled(&self, now: Instant) -> bool {
        self.retry_at.is_none_or(|retry_at| now >= retry_at)
    }

    pub(super) fn activate_fallback(&mut self, now: Instant) -> bool {
        if !self.enabled(now) {
            return false;
        }
        self.retry_at = Some(now + COOLDOWNS[self.cooldown_index]);
        self.cooldown_index = (self.cooldown_index + 1).min(COOLDOWNS.len() - 1);
        self.generation = self.generation.wrapping_add(1);
        true
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn record_completed(&mut self, generation: u64) {
        if generation == self.generation {
            self.cooldown_index = 0;
            self.retry_at = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_is_checked_only_at_request_boundaries() {
        let mut state = WebsocketRecovery::default();
        let now = Instant::now();
        assert!(state.enabled(now));
        assert!(state.activate_fallback(now));
        assert!(!state.enabled(now + Duration::from_secs(59)));
        assert!(!state.activate_fallback(now + Duration::from_secs(59)));
        assert!(state.enabled(now + COOLDOWNS[0]));
        assert_eq!(state.cooldown_index, 1);
    }

    #[test]
    fn failed_recovery_uses_increasing_cooldowns_then_retries_every_half_hour() {
        let mut state = WebsocketRecovery::default();
        let mut now = Instant::now();
        for cooldown in COOLDOWNS {
            assert!(state.activate_fallback(now));
            assert!(!state.enabled(now));
            assert!(!state.enabled(now + cooldown - Duration::from_secs(1)));
            now += cooldown;
            assert!(state.enabled(now));
        }
        // There is no total retry limit, and the interval stays at 30 minutes.
        for _ in 0..100 {
            assert!(state.activate_fallback(now));
            assert!(!state.enabled(now + Duration::from_secs(1799)));
            now += Duration::from_secs(1800);
            assert!(state.enabled(now));
        }
        assert_eq!(state.cooldown_index, 3);
    }

    #[test]
    fn completed_recovery_restarts_the_cooldown_sequence() {
        let mut state = WebsocketRecovery::default();
        let mut now = Instant::now();
        for cooldown in COOLDOWNS {
            assert!(state.activate_fallback(now));
            now += cooldown;
        }
        state.record_completed(state.generation());
        assert!(state.enabled(now));
        assert_eq!(state.cooldown_index, 0);
        assert!(state.activate_fallback(now));
        assert!(!state.enabled(now + Duration::from_secs(59)));
        assert!(state.enabled(now + COOLDOWNS[0]));
    }

    #[test]
    fn completion_from_an_old_stream_cannot_undo_new_fallback() {
        let mut state = WebsocketRecovery::default();
        let old_generation = state.generation();
        let now = Instant::now();
        state.activate_fallback(now);
        state.record_completed(old_generation);
        assert!(!state.enabled(now));
        assert_eq!(state.cooldown_index, 1);
    }

    #[test]
    fn independent_sessions_do_not_share_fallback_state() {
        let mut first = WebsocketRecovery::default();
        let second = WebsocketRecovery::default();
        let now = Instant::now();
        first.activate_fallback(now);
        assert!(!first.enabled(now));
        assert!(second.enabled(now));
    }
}
