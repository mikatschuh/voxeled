use std::time::Instant;

/// Eine Struktur die den Zeitpunkt vom letzten Update speichert.
pub struct DeltaTime {
    last_update: Instant,
}
impl DeltaTime {
    ///
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
    /// This method returns the temporal distance to the old update and logs in the point in time for the new update
    pub fn update(&mut self) -> f32 {
        let delta_time = self.last_update.elapsed().as_nanos();
        self.last_update = Instant::now();
        delta_time as f32
    }
}
