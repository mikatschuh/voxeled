#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Texture {
    CrackedStone,
    Stone,
    Dirt,
    Debug,
}
use Texture::*;

impl Texture {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            CrackedStone => include_bytes!("cracked_stone.png"),
            Stone => include_bytes!("normal_stone.png"),
            Dirt => include_bytes!("dirt.png"),
            Debug => include_bytes!("debug_occlusion.png"),
        }
    }
}
