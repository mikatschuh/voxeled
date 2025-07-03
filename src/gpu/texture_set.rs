pub enum Texture {
    Stone,
    Dirt,
}
use Texture::*;

impl Texture {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Stone => include_bytes!("stone.png"),
            Dirt => include_bytes!("dirt.png"),
        }
    }
}
