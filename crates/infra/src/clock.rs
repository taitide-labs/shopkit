use std::time::SystemTime;

use shopkit_contracts::ports::Clock;
use time::OffsetDateTime;

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::from(SystemTime::now())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    instant: OffsetDateTime,
}

impl FixedClock {
    #[must_use]
    pub const fn new(instant: OffsetDateTime) -> Self {
        Self { instant }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.instant
    }
}
