pub struct Task {
    func: Box<dyn (FnOnce(usize)) + Send + 'static>,
}

impl Task {
    pub fn new(func: Box<dyn (FnOnce(usize)) + Send + 'static>) -> Self {
        Self { func }
    }
    pub fn run(self, i: usize) {
        (self.func)(i)
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
