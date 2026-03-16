use std::time::Instant;

use voxine::print_info;

#[derive(Default)]
pub struct TimingAccumulator {
    total: f64,
    max: f64,
}

impl TimingAccumulator {
    pub fn add(&mut self, duration: std::time::Duration) {
        let ms = duration.as_secs_f64() * 1_000.0;
        self.total += ms;
        self.max = self.max.max(ms);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct PerformanceStats {
    last_report: Instant,
    pub frames: u32,
    pub mesh_updates: u32,
    pub uploaded_bytes: u64,
    pub visible_chunks: u64,
    pub visible_faces: u64,
    pub mesh_update_time: TimingAccumulator,
    pub acquire_time: TimingAccumulator,
    pub main_pass_time: TimingAccumulator,
    pub post_process_time: TimingAccumulator,
    pub submit_present_time: TimingAccumulator,
    pub total_draw_time: TimingAccumulator,
}

impl PerformanceStats {
    pub fn new() -> Self {
        Self {
            last_report: Instant::now(),
            frames: 0,
            mesh_updates: 0,
            uploaded_bytes: 0,
            visible_chunks: 0,
            visible_faces: 0,
            mesh_update_time: TimingAccumulator::default(),
            acquire_time: TimingAccumulator::default(),
            main_pass_time: TimingAccumulator::default(),
            post_process_time: TimingAccumulator::default(),
            submit_present_time: TimingAccumulator::default(),
            total_draw_time: TimingAccumulator::default(),
        }
    }

    pub fn maybe_report(&mut self) {
        let elapsed = self.last_report.elapsed();
        if elapsed.as_secs_f64() < 1.0 {
            return;
        }

        let seconds = elapsed.as_secs_f64();
        let fps = f64::from(self.frames) / seconds;
        let avg = |acc: &TimingAccumulator, count: u32| -> f64 {
            if count == 0 {
                0.0
            } else {
                acc.total / f64::from(count)
            }
        };
        let avg_visible_chunks = if self.frames == 0 {
            0
        } else {
            self.visible_chunks / u64::from(self.frames)
        };
        let avg_visible_faces = if self.frames == 0 {
            0
        } else {
            self.visible_faces / u64::from(self.frames)
        };
        let faces_per_chunk = if self.visible_chunks == 0 {
            0
        } else {
            self.visible_faces / self.visible_chunks
        };
        let faces_per_second = self.visible_faces as f64 / seconds;

        print_info!(
            "perf fps:{:.1} faces/frame:{} chunks/frame:{} faces/chunk:{} faces/s:{:.0} draw_ms:{:.3}/{:.3} main:{:.3} mesh_updates:{}",
            fps,
            avg_visible_faces,
            avg_visible_chunks,
            faces_per_chunk,
            faces_per_second,
            avg(&self.total_draw_time, self.frames),
            self.total_draw_time.max,
            avg(&self.main_pass_time, self.frames),
            self.mesh_updates,
        );

        self.last_report = Instant::now();
        self.frames = 0;
        self.mesh_updates = 0;
        self.uploaded_bytes = 0;
        self.visible_chunks = 0;
        self.visible_faces = 0;
        self.mesh_update_time.reset();
        self.acquire_time.reset();
        self.main_pass_time.reset();
        self.post_process_time.reset();
        self.submit_present_time.reset();
        self.total_draw_time.reset();
    }
}
