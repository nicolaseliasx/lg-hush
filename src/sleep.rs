//! Power off instead of the LG screensaver: with Quick Start+ on, "off" is the TV's real
//! standby and it returns straight to Luma, rather than showing the clock indefinitely.
use std::time::Instant;

/// On webOS 6.5 the screensaver is a tvpower state, not a foreground app.
pub const SCREENSAVER_STATE: &str = "Screen Saver";

#[derive(Default)]
pub struct Watch {
    fired: bool,
}
impl Watch {
    /// Feed `power/getPowerState`'s state; true once as soon as the screensaver starts.
    pub fn observe(&mut self, _now: Instant, state: &str) -> bool {
        if state != SCREENSAVER_STATE {
            self.fired = false;
            return false;
        }
        !std::mem::replace(&mut self.fired, true)
    }
}

#[cfg(test)]
#[path = "sleep-tests.rs"]
mod tests;
