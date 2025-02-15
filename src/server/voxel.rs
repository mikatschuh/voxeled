#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoxelType {
    Air,
    Solid,
}
impl VoxelType {
    pub fn from_random() -> Self {
        let random_index = crate::random::get_random(0, 1); // 0 oder 1
        match random_index {
            0 => Self::Air,
            1 => Self::Solid,
            _ => unreachable!(), // Sollte nie passieren
        }
    }
    pub fn from_random_weighted() -> Self {
        let random_index = crate::random::get_random(0, 4); // 0 oder 1
        match random_index == 0 {
            false => Self::Air,
            true => Self::Solid,
        }
    }
    pub fn is_solid_u32(&self) -> u32 {
        if *self as u8 > 0 {
            0b1000_0000__0000_0000__0000_0000__0000_0000
        } else {
            0
        }
    }
}
use noise::{NoiseFn, Perlin};

#[derive(Debug)]
pub struct AnimatedNoise {
    noise: Perlin,
    time_scale: f64,
    space_scale: f64,
}

impl AnimatedNoise {
    pub fn new(seed: u32, time_scale: f64, space_scale: f64) -> Self {
        Self {
            noise: Perlin::new(seed),
            time_scale,
            space_scale,
        }
    }

    pub fn get(&self, x: f64, y: f64, z: f64, time: f64) -> f64 {
        let animated_x = x * self.space_scale;
        let animated_y = y * self.space_scale;
        let animated_z = z * self.space_scale;
        let t = time * self.time_scale;

        // Variante 1: Zeit direkt als vierte Dimension nutzen
        // let value = self.noise.get([
        //     animated_x, animated_y, animated_z, t, // Zeit als zusätzliche Dimension
        // ]);

        // Oder Variante 2: Bewegte Koordinaten
        let value = self.noise.get([
            animated_x + t,       // Koordinaten bewegen sich mit der Zeit
            animated_y + t * 0.7, // verschiedene Faktoren für mehr Variation
            animated_z + t * 0.3,
        ]);

        (value + 1.0) * 0.5
    }
    pub fn get_octaves(&self, x: f64, y: f64, z: f64, time: f64, octaves: u32) -> f64 {
        let x = x * self.space_scale;
        let y = y * self.space_scale;
        let z = z * self.space_scale;

        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let persistence = 0.5;

        for _ in 0..octaves {
            value += self.get(x * frequency, y * frequency, z, time) * amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        value
    }
}
