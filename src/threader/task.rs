use std::time::Instant;

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
    pub fn execute(self) {
        match self {
            Self::Benchmark { label, time } => {
                println!("{} finished at: {}", label, time.elapsed().as_nanos())
            }
        }
    }
}
