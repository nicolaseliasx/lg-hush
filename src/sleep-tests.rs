use super::*;
use std::time::Instant;

#[test]
fn the_tv_powers_off_as_soon_as_the_screensaver_state_starts() {
    // webOS 6.5: the screensaver is a tvpower state ("Screen Saver"), not a foreground app.
    // The owner wants no clock at all: power off on the first sighting.
    let now = Instant::now();
    let mut watch = Watch::default();
    assert!(!watch.observe(now, "Active"));
    assert!(watch.observe(now, SCREENSAVER_STATE));
    assert!(
        !watch.observe(now, SCREENSAVER_STATE),
        "once per screensaver"
    );
}

#[test]
fn a_new_screensaver_after_waking_powers_off_again() {
    let now = Instant::now();
    let mut watch = Watch::default();
    assert!(watch.observe(now, SCREENSAVER_STATE));
    assert!(!watch.observe(now, "Active"));
    assert!(watch.observe(now, SCREENSAVER_STATE));
}
