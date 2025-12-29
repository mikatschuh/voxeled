use crate::gpu::texture_set::Texture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelType {
    Air,
    CrackedStone,
    Stone,
    Dirt,
}

use VoxelType::*;
impl VoxelType {
    pub fn from_random() -> Self {
        let random_index = crate::random::get_random(0, 2); // 0 oder 1
        match random_index {
            0 => Self::Air,
            1 => Self::Stone,
            2 => Self::Dirt,
            _ => unreachable!(), // Sollte nie passieren
        }
    }

    pub fn random_weighted() -> Self {
        let random_index = crate::random::get_random(0, 4); // 0 oder 1
        match random_index == 0 {
            false => Self::Dirt,
            true => Self::Stone,
        }
    }

    pub fn is_solid_u32(self) -> u32 {
        if self as u8 > 0 {
            0b1000_0000__0000_0000__0000_0000__0000_0000
        } else {
            0
        }
    }

    pub fn texture(self) -> Texture {
        match self {
            Stone => Texture::Stone,
            CrackedStone => Texture::CrackedStone,
            Dirt => Texture::Dirt,
            _ => panic!("Air has no texture!"),
        }
    }
}

pub type VoxelData3D = [[[VoxelType; 32]; 32]; 32];

pub fn fill(fill: VoxelType) -> VoxelData3D {
    [[[fill; 32]; 32]; 32]
}
