use std::time;

pub struct Clock {
    previous: time::Instant,
    accumulator: time::Duration,
    elapsed: time::Duration,
    max_ups: time::Duration,
}

impl Clock {
    pub fn new(max_ups: u64) -> Self {
        let previous = time::Instant::now();
        let accumulator = time::Duration::from_secs(1);
        let elapsed = time::Duration::from_secs(0);
        let max_ups = time::Duration::from_millis(max_ups);

        Self {
            previous,
            elapsed,
            accumulator,
            max_ups,
        }
    }

    pub fn readjust(&mut self) {
        let elapsed = self.previous.elapsed();
        self.elapsed = elapsed;
        self.previous = time::Instant::now();
        self.accumulator += elapsed;
    }

    pub fn should_update(&self) -> bool {
        self.accumulator >= self.max_ups
    }

    pub fn update(&mut self) {
        self.accumulator -= self.max_ups;
    }

    pub fn elapsed(&self) -> time::Duration {
        self.previous.elapsed()
    }

    pub fn inner(&self) -> &time::Instant {
        &self.previous
    }
}
