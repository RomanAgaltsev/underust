//! TEMPORARY. Proves the windows CI leg can actually go red.
//!
//! Nothing in the daily loop runs on Windows (spec 9.3), so that job is the only thing
//! standing behind R14's promise that Windows is first class -- and a green check nobody
//! has watched fail is not evidence. This file is deleted once the failure is observed.

#[test]
fn the_windows_leg_can_fail() {
    assert_eq!(
        std::path::MAIN_SEPARATOR,
        '/',
        "deliberate: true on unix, false on windows"
    );
}
