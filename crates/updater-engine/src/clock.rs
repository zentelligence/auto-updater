/// An injectable source of the current time, so timestamp-bearing state (`state.json`,
/// diagnostics) is deterministic in tests.
pub trait Clock {
    /// Returns the current instant as an RFC 3339 timestamp string, e.g. `2026-09-21T04:16:43Z`.
    fn now_rfc3339(&self) -> String;
}

/// The real wall-clock, used in production.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        let now = time::OffsetDateTime::now_utc();
        now.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| now.unix_timestamp().to_string())
    }
}

#[cfg(test)]
#[path = "clock_test.rs"]
mod tests;
