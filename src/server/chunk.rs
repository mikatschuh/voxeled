use crate::gpu::vertex::Vertex;
use glam::IVec3;

struct Chunk {
    pos: IVec3,
    bitmap: [[u32; 32]; 32],
    entities: Vec<Entity>,
}
enum Entity {}

struct Plane([u32; 32]);

impl Chunk {
    pub fn with_ground_layer() -> Self {
        Self {
            pos: IVec3::ZERO,
            bitmap: [[1; 32]; 32],
            entities: Vec::new(),
        }
    }
}
pub fn create_faces(bit_map: &[[u32; 32]; 32]) -> [[Plane; 32]; 6] {
    let mut planes = [[Plane([0; 32]); 32]; 32];

    for row in self.bitmap.iter() {
        for block in row.iter() {}
    }
    // row = original & !(original >> 1)
}
/* Cullign Algorithm

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
