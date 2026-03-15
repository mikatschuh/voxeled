#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Texture {
    CrackedStone,
    Stone,
    Dirt0,
    Dirt1,
    Debug,
}
use Texture::*;

impl Texture {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            CrackedStone => include_bytes!("normal_stone-8x8.png"),
            Stone => include_bytes!("normal_stone-8x8.png"),
            Dirt0 => include_bytes!("dirt-0.png"),
            Dirt1 => include_bytes!("dirt-1.png"),
            Debug => include_bytes!("normal_stone-8x8.png"),
        }
    }
}
