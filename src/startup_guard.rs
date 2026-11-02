//! Transport-independent startup policy, also exercised by the AAW1 host test gate.
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WppAction { Accept, ReplyPing, IgnorePing, Reject }

pub(crate) fn wpp_action(expected: u16, actual: u16) -> WppAction {
    if actual == expected { WppAction::Accept }
    else if actual == 8 { WppAction::ReplyPing }
    else if actual == 9 { WppAction::IgnorePing }
    else { WppAction::Reject }
}

pub(crate) const BOTH_SIDES_READY: u8 = 3;
pub(crate) const STARTUP_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Default)]
pub(crate) struct StartupGuard {
    armed_at: Option<Duration>,
    totals: (usize, usize),
}
impl StartupGuard {
    /// Returns true exactly when normal streaming monitoring is armed.
    pub(crate) fn check(&mut self, elapsed: Duration, ready: u8, totals: (usize, usize), idle: Duration)
        -> Result<bool, &'static str> {
        if ready & BOTH_SIDES_READY != BOTH_SIDES_READY {
            return if elapsed >= STARTUP_TIMEOUT { Err("initial AA negotiation timed out before both TLS sides became ready") } else { Ok(false) };
        }
        let Some(last) = self.armed_at else {
            self.armed_at = Some(elapsed);
            self.totals = totals;
            return Ok(true);
        };
        if elapsed.saturating_sub(last) >= idle {
            if totals.0 == self.totals.0 || totals.1 == self.totals.1 {
                return Err("unexpected transfer stall after AA negotiation");
            }
            self.armed_at = Some(elapsed);
            self.totals = totals;
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sec(n: u64) -> Duration { Duration::from_secs(n) }
    #[test]
    fn ping_does_not_consume_start_or_connect_status() {
        let mut expected = 7;
        let mut accepted = Vec::new();
        for actual in [8, 9, 8, 7, 8, 6] {
            if wpp_action(expected, actual) == WppAction::Accept {
                accepted.push(actual); expected = 6;
            }
        }
        assert_eq!(accepted, [7, 6]);
    }
    #[test]
    fn wrong_status_frame_is_rejected() {
        assert_eq!(wpp_action(6, 7), WppAction::Reject);
        assert_eq!(wpp_action(7, 6), WppAction::Reject);
        assert_eq!(wpp_action(2, 65535), WppAction::Reject);
    }
    #[test]
    fn ping_requires_reply_and_ping_response_is_ignored() {
        for expected in [2, 7, 6] {
            assert_eq!(wpp_action(expected, 8), WppAction::ReplyPing);
            assert_eq!(wpp_action(expected, 9), WppAction::IgnorePing);
        }
    }
    #[test]
    fn no_false_ten_second_timeout_during_version_or_tls() {
        let mut g = StartupGuard::default();
        assert_eq!(g.check(sec(10), 0, (0, 0), sec(10)), Ok(false));
        assert_eq!(g.check(sec(15), 1, (0, 0), sec(10)), Ok(false));
        assert_eq!(g.check(sec(44), 2, (0, 0), sec(10)), Ok(false));
    }
    #[test]
    fn startup_remains_bounded() {
        for flags in [0, 1, 2] {
            assert!(StartupGuard::default().check(sec(45), flags, (999, 999), sec(10)).is_err());
        }
    }
    #[test]
    fn full_idle_window_starts_when_both_sides_are_ready() {
        let mut g = StartupGuard::default();
        assert_eq!(g.check(sec(20), 3, (0, 0), sec(10)), Ok(true));
        assert_eq!(g.check(sec(29), 3, (0, 0), sec(10)), Ok(false));
        assert!(g.check(sec(30), 3, (0, 0), sec(10)).is_err());
    }
    #[test]
    fn passthrough_arms_immediately_and_checks_both_directions() {
        for totals in [(10, 0), (0, 10)] {
            let mut g = StartupGuard::default();
            assert_eq!(g.check(sec(0), 3, (0, 0), sec(10)), Ok(true));
            assert!(g.check(sec(10), 3, totals, sec(10)).is_err());
        }
    }
    #[test]
    fn active_stream_updates_baseline_per_window() {
        let mut g = StartupGuard::default();
        assert_eq!(g.check(sec(0), 3, (9, 8), sec(10)), Ok(true));
        assert_eq!(g.check(sec(10), 3, (10, 20), sec(10)), Ok(false));
        assert_eq!(g.check(sec(20), 3, (11, 21), sec(10)), Ok(false));
        assert!(g.check(sec(30), 3, (11, 30), sec(10)).is_err());
    }
}
