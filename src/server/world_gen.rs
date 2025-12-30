use num::pow::Pow;

use crate::{
    random::Noise,
    server::{
        chunks::ChunkID,
        voxel::{VoxelData3D, VoxelType},
    },
};

type Seed = u64;
pub trait Generator: Clone + Send + Sync + 'static {
    fn new(seed: Seed) -> Self;
    fn gen(&self, chunk_id: ChunkID) -> VoxelData3D;
    fn seed(&self) -> Seed;
}

#[derive(Clone)]
pub struct MountainsAndValleys {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub vertical_area: f64,
    pub number_of_octaves: usize,
}

impl Generator for MountainsAndValleys {
    fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 20.0,
            vertical_area: 1200.0,
            number_of_octaves: 3,
        }
    }

    fn gen(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for x in 0..32 {
            for z in 0..32 {
                let height = self.noise.get_octaves(
                    (x as i32 + chunk_id.pos.x * 32 << chunk_id.lod) as f64,
                    0.0,
                    (z as i32 + chunk_id.pos.z * 32 << chunk_id.lod) as f64,
                    self.horizontal_area,
                    self.number_of_octaves,
                );
                assert!(height <= 1.0);
                assert!(height >= 0.0);
                for y in 0..32 {
                    voxels[x][y][z] = if y as i32 + chunk_id.pos.y * 32 << chunk_id.lod
                        < (2.0.pow(height) * self.vertical_area) as i32
                    {
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct WhiteNoise {
    pub seed: Seed,
}

impl Generator for WhiteNoise {
    fn new(seed: Seed) -> Self {
        Self { seed }
    }
    fn gen(&self, _chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for plane in voxels.iter_mut() {
            for row in plane.iter_mut() {
                for voxel in row.iter_mut() {
                    *voxel = VoxelType::random_weighted()
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct RainDrops {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub exponent: i32,
    pub threshold: f64,
    pub number_of_octaves: usize,
}

impl Generator for RainDrops {
    fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 5.0,
            exponent: 1,
            threshold: 0.8,
            number_of_octaves: 1,
        }
    }
    fn gen(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let val = self.noise.get_octaves(
                        (x as i32 + chunk_id.pos.x * 32 << chunk_id.lod) as f64,
                        (y as i32 + chunk_id.pos.y * 32 << chunk_id.lod) as f64,
                        (z as i32 + chunk_id.pos.z * 32 << chunk_id.lod) as f64,
                        self.horizontal_area,
                        self.number_of_octaves,
                    );
                    *voxel = if val.pow(self.exponent) > self.threshold {
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct OpenCaves {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub exponent: i32,
    pub threshold: f64,
    pub number_of_octaves: usize,

    pub material_noise: Noise,
    pub material_scale: f64,
    pub material_threshold: f64,
    pub material_octaves: usize,
}

impl Generator for OpenCaves {
    fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 32.0, // 8.0,
            exponent: 1,
            threshold: 0.5,
            number_of_octaves: 9,

            material_noise: Noise::new(seed as u32 ^ 0b11010101010101010100011010101010),
            material_scale: 8.0,
            material_threshold: 0.6,
            material_octaves: 3,
        }
    }

    fn gen(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let pos = (
                        (x as i32 + chunk_id.pos.x * 32 << chunk_id.lod) as f64,
                        (y as i32 + chunk_id.pos.y * 32 << chunk_id.lod) as f64,
                        (z as i32 + chunk_id.pos.z * 32 << chunk_id.lod) as f64,
                    );

                    let val = self.noise.get_octaves(
                        pos.0,
                        pos.1,
                        pos.2,
                        self.horizontal_area,
                        self.number_of_octaves,
                    );

                    *voxel = if val.pow(self.exponent) <= self.threshold {
                        VoxelType::Air
                    } else {
                        let mat = self.material_noise.get_octaves(
                            pos.0,
                            pos.1,
                            pos.2,
                            self.material_scale,
                            self.material_octaves,
                        );

                        match mat {
                            _ if mat >= self.material_threshold => VoxelType::CrackedStone,
                            _ => VoxelType::Stone,
                        }
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}
