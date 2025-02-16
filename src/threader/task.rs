pub enum Task {
    Dynamic(Box<dyn (FnOnce()) + Send + 'static>),
}
use Task::*;

impl Task {
    pub fn new_dynamic<F>(func: F) -> Self
    where
        F: (FnOnce()) + Send + 'static,
    {
        Self::Dynamic(Box::new(func))
    }
    pub fn run(self) {
        match self {
            Dynamic(func) => (func)(),
        }
    }
}
/*
           GenerateChunkMesh {
               result,
               cam_pos,
               chunk_pos,
               noise,
               chunks,
           } => {
               let _ = result.send(crate::server::chunk::generate_mesh(
                   cam_pos,
                   chunk_pos,
                   chunks
                       .lock()
                       .unwrap()
                       .get(chunk_pos, |pos| Chunk::from_fractal_noise(pos, &noise, 0.0))
                       .create_faces(),
               ));
           }
*/
