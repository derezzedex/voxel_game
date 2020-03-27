use std::time;

pub fn to_secs(dur: time::Duration) -> f64{
    dur.as_secs() as f64 +
    dur.subsec_nanos() as f64 / 1e6
}

pub struct Clock{
    previous: time::Instant,
    pub accumulator: time::Duration,
    pub elapsed: time::Duration,
    pub max_ups: time::Duration
}

impl Clock{
    pub fn new(max_ups: u64) -> Self{
        let previous = time::Instant::now();
        let accumulator = time::Duration::from_secs(1);
        let elapsed = time::Duration::from_secs(0);
        let max_ups = time::Duration::from_millis(max_ups);

        Self{
            previous,
            elapsed,
            accumulator,
            max_ups
        }
    }

    pub fn readjust(&mut self){
        let elapsed = self.previous.elapsed();
        self.elapsed = elapsed;
        self.previous = time::Instant::now();
        self.accumulator += elapsed;
    }

    pub fn should_update(&self) -> bool{
        self.accumulator >= self.max_ups
    }

    pub fn update(&mut self){
        self.accumulator -= self.max_ups;
    }

    pub fn elapsed(&self) -> time::Duration{
        self.previous.elapsed()
    }

    pub fn get_timer(&self) -> &time::Instant{
        &self.previous
    }
}
