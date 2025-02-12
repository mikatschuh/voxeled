use super::voxel::VoxelType;
use glam::{IVec3, UVec3, Vec3};

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
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub pos: IVec3,
    pub voxels: [[[VoxelType; 32]; 32]; 32],
    pub entities: Vec<Entity>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum Entity {}

/// Speichert eine Bitmap wo ein Bit gesetzt ist wenn ein Face von einer Richtung nicht verdeckt ist.
#[derive(Debug, Clone, Copy)]
pub struct ChunkFaces([[u32; 32]; 32]);
impl ChunkFaces {
    fn new() -> Self {
        Self([[0; 32]; 32])
    }
}

impl Chunk {
    const NUM_OF_SOLID_BLOCKS: usize = 5;
    const NUM_OF_NON_SOLID_BLOCKS: usize = 5;

    pub fn from_white_noise() -> Self {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for plane in voxels.iter_mut() {
            for row in plane.iter_mut() {
                for voxel in row.iter_mut() {
                    *voxel = VoxelType::from_random_weighted()
                }
            }
        }

        Self {
            pos: IVec3::ZERO,
            voxels,
            entities: Vec::new(),
        }
    }
    pub fn with_ground_layer() -> Self {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        voxels[0][0][0] = VoxelType::TestTreePicture;
        Self {
            pos: IVec3::ZERO,
            voxels,
            entities: Vec::new(),
        }
    }
    pub fn create_faces(&self) -> [ChunkFaces; 6] {
        let mut faces = [ChunkFaces([[0; 32]; 32]); 6];
        // 0 = -x
        // 1 = +x
        // 2 = -y
        // 3 = +y
        // 4 = -z
        // 5 = +z

        let mut x_aligned = [[0; 32]; 32];
        let mut y_aligned = [[0; 32]; 32];
        let mut z_aligned = [[0; 32]; 32];

        // data setup
        for (x, plane) in self.voxels.iter().enumerate() {
            for (y, row) in plane.iter().enumerate() {
                for (z, voxel) in row.iter().enumerate() {
                    let voxel_is_solid_u32 = voxel.is_solid_u32();

                    if voxel_is_solid_u32 > 0 {
                        x_aligned[y][z] |= voxel_is_solid_u32 >> x;
                        y_aligned[z][x] |= voxel_is_solid_u32 >> y;
                        z_aligned[x][y] |= voxel_is_solid_u32 >> z;
                    }
                }
            }
        }

        for i in 0..32 {
            for j in 0..32 {
                faces[0].0[i][j] =
                    x_aligned[i][j] & !((x_aligned[i][j] >> 1)/* | (neighbor_row << 31) */);
                faces[1].0[i][j] =
                    x_aligned[i][j] & !((x_aligned[i][j] << 1)/* | (neighbor_row >> 31) */);
                faces[2].0[i][j] =
                    y_aligned[i][j] & !((y_aligned[i][j] >> 1)/* | (neighbor_row << 31) */);
                faces[3].0[i][j] =
                    y_aligned[i][j] & !((y_aligned[i][j] << 1)/* | (neighbor_row >> 31) */);
                faces[4].0[i][j] =
                    z_aligned[i][j] & !((z_aligned[i][j] >> 1)/* | (neighbor_row << 31) */);
                faces[5].0[i][j] =
                    z_aligned[i][j] & !((z_aligned[i][j] << 1)/* | (neighbor_row >> 31) */);
            }
        }
        println!(
            "\
            x:  {}\n\
            -x: {}\n\
            +x: {}\n\n\
            y:  {}\n\
            -y: {}\n\
            +y: {}\n\n\
            z:  {}\n\
            -z: {}\n\
            +z: {}\n\
            ",
            format(x_aligned[0][0]),
            format(faces[0].0[0][0]),
            format(faces[1].0[0][0]),
            format(y_aligned[0][0]),
            format(faces[2].0[0][0]),
            format(faces[3].0[0][0]),
            format(z_aligned[0][0]),
            format(faces[4].0[0][0]),
            format(faces[5].0[0][0])
        );
        fn format(num: u32) -> String {
            let mut out = String::new();
            for i in (0..32).rev() {
                out += match num >> i & 1 {
                    0 => "   ",
                    1 => "|#|",
                    _ => unreachable!(),
                }
            }
            out
        }
        // WARNING! additional step: add non solid blocks back in
        faces
    }
}

use crate::gpu::mesh::Mesh;
pub fn generate_mesh(chunk_pos: IVec3, faces: [ChunkFaces; 6]) -> Mesh {
    let mut mesh = Mesh {
        vertices: Vec::new(), // the points
        indices: Vec::new(),  // which points form a triangle
    };
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                let position = (chunk_pos + IVec3::new(x as i32, y as i32, z as i32)).as_vec3();

                let masks_bit = 1_u32 << 31;
                if faces[0].0[y][z] & (masks_bit >> x) > 0 {
                    mesh += Mesh::face_nx(position)
                }
                if faces[1].0[y][z] & (masks_bit >> x) > 0 {
                    mesh += Mesh::face_px(position)
                }
                if faces[2].0[z][x] & (masks_bit >> y) > 0 {
                    mesh += Mesh::face_ny(position)
                }
                if faces[3].0[z][x] & (masks_bit >> y) > 0 {
                    mesh += Mesh::face_py(position)
                }
                if faces[4].0[x][y] & (masks_bit >> z) > 0 {
                    mesh += Mesh::face_nz(position)
                }
                if faces[5].0[x][y] & (masks_bit >> z) > 0 {
                    mesh += Mesh::face_pz(position)
                }
            }
        }
    }
    println!("{:?}", mesh);

    mesh
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
