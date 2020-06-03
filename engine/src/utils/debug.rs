use log::info;
use std::fmt;
use std::time::{Instant, Duration};

#[derive(Debug, Copy, Clone)]
pub struct DebugInfo{
    pub timer: Instant,

    pub frames: usize,
    pub updates: usize,
    pub total_updates: usize,
}

impl Default for DebugInfo{
    fn default() -> Self{
        Self{
            timer: Instant::now(),

            frames: 0,
            updates: 0,
            total_updates: 0,
        }
    }
}

impl fmt::Display for DebugInfo{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Timer: {:?} Frames: {} Updates: {} Total: {}", self.timer.elapsed(), self.frames, self.updates, self.total_updates)
    }
}

impl DebugInfo{
    pub fn new() -> Self{
        Default::default()
    }

    pub fn update(self) -> Self{
        if self.timer.elapsed() >= Duration::from_secs(1){
            info!("{}", self);
            return Self { total_updates: self.total_updates, .. Default::default() };
        }

        self
    }
}
