use crate::gpu::vertex::Vertex;
use glam::IVec3;

struct Chunk {
    pos: IVec3,
    bitmap: [[u32; 32]; 32],
    entities: Vec<Entity>,
}
enum Entity {}

#[derive(Clone, Copy)]
struct ChunkFaces([[u32; 32]; 32]);

impl Chunk {
    pub fn with_ground_layer() -> Self {
        Self {
            pos: IVec3::ZERO,
            bitmap: [[1; 32]; 32],
            entities: Vec::new(),
        }
    }
}
const NUM_OF_BLOCKS: usize = 5;
pub fn create_faces(bit_maps: [&[[u32; 32]; 32]; NUM_OF_BLOCKS]) -> [ChunkFaces; 6] {
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
    //fn to_vertices(self) -> ()
}
