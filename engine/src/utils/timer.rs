use std::time::{Duration, Instant};

pub struct Timer{
    ups: Duration,
    accumulator: Duration,
    delta: Duration,
    current: Instant,
}

impl Timer{
    pub fn new(ups: u64) -> Self{
        let ups = Duration::from_millis(1000 / ups);
        let accumulator = Duration::new(0, 0);
        let delta = Duration::new(0, 0);
        let current = Instant::now();

        Self{
            ups,
            accumulator,
            delta,
            current
        }
    }

    pub fn reset(&mut self){
        let now = Instant::now();
        self.delta = now - self.current;
        self.current = now;
        self.accumulator += self.delta;
    }

    pub fn update(&mut self){
        self.accumulator -= self.ups;
    }

    pub fn should_update(&self) -> bool{
        self.accumulator >= self.ups
    }
}
