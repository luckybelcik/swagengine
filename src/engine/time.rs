use std::time::Instant;

pub struct Time {
    last_frame: Instant,
    delta_time: f32,
    frame_count: u32,
    total_time: f32,
    sample_time: f32,
    sampled_frame_count: u32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta_time: 0.0,
            frame_count: 0,
            total_time: 0.0,
            sample_time: 0.0,
            sampled_frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.total_time += self.delta_time;
        self.sample_time += self.delta_time;
        self.frame_count += 1;
        self.sampled_frame_count += 1;
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn average_fps(&self) -> f32 {
        if self.sample_time > 0.0 {
            self.sampled_frame_count as f32 / self.sample_time
        } else {
            0.0
        }
    }

    pub fn reset_average_fps(&mut self) {
        self.sample_time = 0.0;
        self.sampled_frame_count = 0;
    }
}