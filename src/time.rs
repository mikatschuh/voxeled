use crossbeam::atomic::AtomicCell;
use std::{sync::Arc, time::Instant};

/// Eine Struktur die den Zeitpunkt vom letzten Update speichert und updaten kann.
pub struct DeltaTimeMeter {
    last_update: Instant,
    current: DeltaTime,
}
impl DeltaTimeMeter {
    /// Erstellt eine neue DeltaTime - Instanz.
    pub fn now() -> Self {
        Self {
            last_update: Instant::now(),
            current: DeltaTime(Arc::new(AtomicCell::new(0))),
        }
    }
    /// Lockt die neue DeltaTime ein.
    pub fn update(&mut self) {
        self.current
            .0
            .store(self.last_update.elapsed().as_nanos() as u64);
        self.last_update = Instant::now()
    }
    pub fn new_reader(&self) -> DeltaTime {
        self.current.clone()
    }
}
#[derive(Clone)]
pub struct DeltaTime(Arc<AtomicCell<u64>>);

impl DeltaTime {
    /// Gibt die Delta-Time zurück. In Millisekunden.
    pub fn get(&self) -> f32 {
        self.0.load() as f32 / 1_000_000.0
    }
}
use std::fmt;
impl fmt::Debug for DeltaTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
