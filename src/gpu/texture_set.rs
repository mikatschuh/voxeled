#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Texture {
    Stone,
    Dirt,
    Debug,
}
use Texture::*;

impl Texture {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Stone => include_bytes!("stone.png"),
            Dirt => include_bytes!("dirt.png"),
            Debug => include_bytes!("debug_occlusion.png"),
        }
    }
}
