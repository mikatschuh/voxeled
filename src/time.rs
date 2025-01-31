use std::time::Instant;

/// Eine Struktur die den Zeitpunkt vom letzten Update speichert.
pub struct DeltaTime {
    last_update: Instant,
}
impl DeltaTime {
    /// Erstellt eine neue DeltaTime - Instanz.
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
    /// Gibt die die temporale Distanz zum letzten Update zurück und setzt den Zeitpunkt des letzten Updates auf jetzt.
    pub fn update(&mut self) -> f32 {
        let delta_time = self.last_update.elapsed().as_nanos();
        self.last_update = Instant::now();
        delta_time as f32
    }
}
