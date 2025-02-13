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
use noise::{NoiseFn, Perlin, Seedable};

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
        // Verwende die Zeit als vierte Dimension für smooth Animation
        let animated_x = x * self.space_scale;
        let animated_y = y * self.space_scale;
        let animated_z = z * self.space_scale;
        let t = time * self.time_scale;

        // 4D Noise durch Interpolation von zwei 3D Noise Werten
        let noise1 = self.noise.get([animated_x, animated_y, animated_z]);
        let noise2 = self
            .noise
            .get([animated_x + 123.0, animated_y + 456.0, animated_z + 789.0]);

        // Smooth Interpolation zwischen den Noise Werten basierend auf der Zeit
        let blend = (t.sin() + 1.0) * 0.5;
        noise1 * (1.0 - blend) + noise2 * blend
    }
}
