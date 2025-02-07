use crate::gpu::vertex::Vertex;
use glam::IVec3;

pub struct Chunks {
    chunks: Vec<Chunk>,
}

pub struct ChunkHandle {
    chunk: *mut Chunk,
}

impl Chunks {
    pub fn get_handle() -> ChunkHandle {
        todo!()
    }
}

pub struct Chunk {
    pos: IVec3,
    solid_bit_map_x: [[u32; 32]; 32],
    solid_bit_map_y: [[u32; 32]; 32],
    solid_bit_map_z: [[u32; 32]; 32],
    entities: Vec<Entity>,
}
enum Entity {}

#[derive(Clone, Copy)]
struct ChunkFaces([[u32; 32]; 32]);

impl Chunk {
    pub fn with_ground_layer() -> Self {
        Self {
            pos: IVec3::ZERO,
            solid_bit_map_x: [[0; 32]; 32],
            solid_bit_map_y: [[0; 32]; 32],
            solid_bit_map_z: [[0; 32]; 32],
            entities: Vec::new(),
        }
    }
}
const NUM_OF_BLOCKS: usize = 5;
pub fn create_faces(chunk: &Chunk) -> [ChunkFaces; 6] {
    let mut faces = [ChunkFaces([[0; 32]; 32]); 6]; // output value

    // data setup:
    for plane in 0..32 {
        for row_index in 0..32 {
            let mut row = 0_u32;
            for bit_map in bit_maps.iter() {
                row |= bit_map[plane][row_index];
            }
            faces[0].0[plane][row_index] = row & !((row >> 1)/* | (neighbor_row << 31)*/);
        }
    }

    faces
}
/* Cullign Algorithms

integer:

goal: find voxels that arent covered on the left.

#.#..###.###.##.
|              \

>>

.#.#..###.###.##
|              \

!

#.#.##...#...#..
|              \

& with #.#..###.###.##.

#.#..#...#...#..
================
*/
impl ChunkFaces {
    fn to_vertices(self) -> (&'static [Vertex], &'static [u16]) {
        let mut vertices = Vec::new(); // the points
        let mut indices = Vec::new(); // which points form a triangle

        for (plane_num, plane) in self.0.into_iter().enumerate() {
            for (row_num, row) in plane.into_iter().enumerate() {}
        }

        (vertices.leak(), indices.leak())
    }
}
