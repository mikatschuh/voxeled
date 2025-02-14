use super::chunk::Chunk;
use glam::IVec3;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

pub struct Chunks {
    chunks: HashMap<IVec3, Mutex<Chunk>>,
}

pub struct ChunkHandle<'a> {
    chunk: MutexGuard<'a, Chunk>,
}

impl Chunks {
    pub fn get<'a>(&'a self, pos: IVec3) -> MutexGuard<'a, Chunk> {
        match self.chunks.get(&pos) {
            Some(chunk) => chunk.lock().unwrap(),
            None => todo!(),
        }
    }
}
