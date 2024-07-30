use crate::warn;
use core::{arch::asm, ops::Add, time::Duration};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct TimerValue(u64);

impl Add for TimerValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        TimerValue(self.0.wrapping_add(other.0))
    }
}

fn read_cntfrq() -> u64 {
    let x: u64;
    unsafe { asm!("mrs {}, CNTFRQ_EL0", out(reg) x) }
    x
}

fn read_cntpct() -> u64 {
    let x: u64;
    unsafe { asm!("mrs {}, CNTPCT_EL0", out(reg) x) }
    x
}

fn current_cntpct() -> TimerValue {
    // Make sure the read is not optimized by the cpu and ran ahead of time
    unsafe { asm!("isb sy") };
    TimerValue(read_cntpct())
}

impl From<TimerValue> for Duration {
    fn from(value: TimerValue) -> Self {
        let frq = read_cntfrq();

        let secs = value.0 / frq;

        let nanos = value.0 % frq;
        let nanos = (nanos * 1_000_000_000) / frq;

        Duration::new(secs, nanos as u32)
    }
}

impl TryFrom<Duration> for TimerValue {
    type Error = &'static str;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        if value < resolution() {
            return Err("Duration too short");
        }
        if value > max_duration() {
            return Err("Duration too long");
        }

        let total = value.as_nanos();
        let frq = read_cntfrq() as u128;
        let val = (frq * total) / 1_000_000_000;

        Ok(TimerValue(val as u64))
    }
}

pub fn resolution() -> Duration {
    Duration::from(TimerValue(1))
}

pub fn max_duration() -> Duration {
    Duration::from(TimerValue(u64::MAX))
}

pub fn uptime() -> Duration {
    current_cntpct().into()
}

pub fn spin_for(duration: Duration) {
    let curr_count = current_cntpct();

    let count_delta: TimerValue = match duration.try_into() {
        Ok(x) => x,
        Err(s) => {
            warn!("ignoring spin for {:?}. {}", duration, s);
            return;
        }
    };
    let target = curr_count + count_delta;
    while TimerValue(read_cntpct()) < target {}
}
