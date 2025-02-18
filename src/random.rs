use rand::Rng;

pub fn get_random<T: Ord + rand::distributions::uniform::SampleUniform>(min: T, max: T) -> T {
    rand::thread_rng().gen_range(min..=max)
}
pub fn flip_coin() -> bool {
    rand::random()
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
