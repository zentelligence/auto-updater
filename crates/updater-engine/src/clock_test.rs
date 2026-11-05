use super::*;
use crate::test_support::FixedClock;

#[test]
fn fixed_clock_returns_its_configured_value() {
    let clock = FixedClock("2026-09-21T04:16:43Z");

    assert_eq!(clock.now_rfc3339(), "2026-09-21T04:16:43Z");
}

#[test]
fn system_clock_produces_a_parseable_rfc3339_timestamp() {
    let timestamp = SystemClock.now_rfc3339();

    let parsed = time::OffsetDateTime::parse(&timestamp, &time::format_description::well_known::Rfc3339);
    assert!(
        parsed.is_ok(),
        "expected a parseable RFC 3339 timestamp, got {timestamp:?}"
    );
}
