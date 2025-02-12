use std::time::{Duration, Instant};

pub enum Task {
    Benchmark { label: &'static str, time: Instant },
}

impl Task {
    pub fn new_benchmark(label: &'static str) -> Self {
        Self::Benchmark {
            label,
            time: Instant::now(),
        }
    }
    pub fn execute(self, i: usize) {
        match self {
            Self::Benchmark { label, time } => {
                std::thread::sleep(Duration::from_millis(300));
                println!("#{}: {} finished at: {:#?}", i, label, time.elapsed())
            }
        }
    }
    pub fn finish_protocol() -> Vec<Self> {
        vec![Self::Benchmark {
            label: "last",
            time: Instant::now(),
        }]
    }
}
