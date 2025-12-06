use glam::IVec3;
use num::pow::Pow;

use crate::{
    random::Noise,
    server::{
        chunk::{map_visible, Chunk, ChunkFaces},
        voxel::VoxelType,
        Chunks,
    },
};

type Seed = u64;
pub trait Generator: Clone + Send + Sync + 'static {
    fn new(seed: Seed) -> Self;
    fn gen(&self, pos: IVec3, other_chunks: &Chunks) -> Chunk;
    fn seed(&self) -> Seed;
}
#[derive(Clone)]
pub struct MountainsAndValleys {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub vertical_area: f64,
    pub exponent: i32,
    pub number_of_octaves: usize,
}
impl Generator for MountainsAndValleys {
    fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 20.0,
            exponent: 2,
            vertical_area: 500.0,
            number_of_octaves: 3,
        }
    }
    fn gen(&self, pos: IVec3, other_chunks: &Chunks) -> Chunk {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        let mut empty = true;
        for x in 0..32 {
            for z in 0..32 {
                let height = self.noise.get_octaves(
                    (x as i32 + pos.x * 32) as f64,
                    0.0,
                    (z as i32 + pos.z * 32) as f64,
                    self.horizontal_area,
                    self.number_of_octaves,
                );
                assert!(height <= 1.0);
                assert!(height >= 0.0);
                for y in 0..32 {
                    voxels[x][y][z] = if y as i32 + pos.y * 32
                        > (2.0.pow(height.pow(self.exponent)) * self.vertical_area) as i32
                    {
                        empty = false;
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
        Chunk {
            pos,
            voxels,
            occlusion_map: if empty {
                [ChunkFaces([[0; 32]; 32]); 6]
            } else {
                map_visible(&voxels, pos, other_chunks)
            },
            entities: Vec::new(),
            is_empty: empty,
        }
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
    fn gen(&self, pos: IVec3, other_chunks: &Chunks) -> Chunk {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for plane in voxels.iter_mut() {
            for row in plane.iter_mut() {
                for voxel in row.iter_mut() {
                    *voxel = VoxelType::random_weighted()
                }
            }
        }
        Chunk {
            pos,
            voxels,
            occlusion_map: map_visible(&voxels, pos, other_chunks),
            entities: Vec::new(),
            is_empty: false,
        }
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
            horizontal_area: 10.0,
            exponent: 1,
            threshold: 0.5,
            number_of_octaves: 1,
        }
    }
    fn gen(&self, pos: IVec3, other_chunks: &Chunks) -> Chunk {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        let mut empty = true;
        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let val = self.noise.get_octaves(
                        (x as i32 + pos.x * 32) as f64,
                        (y as i32 + pos.y * 32) as f64,
                        (z as i32 + pos.z * 32) as f64,
                        self.horizontal_area,
                        self.number_of_octaves,
                    );
                    *voxel = if val.pow(self.exponent) > self.threshold {
                        empty = false;
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }

        Chunk {
            pos,
            voxels,
            occlusion_map: if empty {
                [ChunkFaces([[0; 32]; 32]); 6]
            } else {
                map_visible(&voxels, pos, other_chunks)
            },
            entities: Vec::new(),
            is_empty: empty,
        }
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}
