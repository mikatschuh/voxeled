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
