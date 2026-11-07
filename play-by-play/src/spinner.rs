use std::time::Duration;

use tokio::time::{self, Instant, Interval};

#[derive(Clone, Copy)]
pub(crate) struct Spinner {
    started: Instant,
}

impl Spinner {
    pub(crate) const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    const PERIOD: Duration = Duration::from_millis(80);

    pub(crate) fn new() -> Spinner {
        Spinner {
            started: Instant::now(),
        }
    }

    pub(crate) fn turns(&self) -> Interval {
        time::interval_at(self.started + Spinner::PERIOD, Spinner::PERIOD)
    }

    pub(crate) fn frame(&self) -> &'static str {
        let turns = self.started.elapsed().as_millis() / Spinner::PERIOD.as_millis();
        Spinner::FRAMES[usize::try_from(turns).unwrap_or_default() % Spinner::FRAMES.len()]
    }
}
