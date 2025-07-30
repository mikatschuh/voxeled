use super::Chunks;

use super::voxel::VoxelType;
use glam::{IVec3, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub pos: IVec3,
    pub(super) voxels: [[[VoxelType; 32]; 32]; 32],
    pub occlusion_map: [ChunkFaces; 6],
    pub(super) entities: Vec<Entity>,
    pub is_empty: bool,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Entity {}

/// Speichert eine Bitmap wo ein Bit gesetzt ist wenn ein Face von einer Richtung nicht verdeckt ist.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkFaces(pub [[u32; 32]; 32]);
impl ChunkFaces {
    fn new() -> Self {
        Self([[0; 32]; 32])
    }
}

impl Chunk {
    const NUM_OF_SOLID_BLOCKS: usize = 5;
    const NUM_OF_NON_SOLID_BLOCKS: usize = 5;
}
fn get_axis_aligned_solid_maps(voxels: &[[[VoxelType; 32]; 32]; 32]) -> [[[u32; 32]; 32]; 3] {
    let mut x_aligned = [[0; 32]; 32];
    let mut y_aligned = [[0; 32]; 32];
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in voxels.iter().enumerate() {
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
    [x_aligned, y_aligned, z_aligned]
}
fn get_x_aligned_solid_map(voxels: &[[[VoxelType; 32]; 32]; 32]) -> [[u32; 32]; 32] {
    let mut x_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in voxels.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    x_aligned[y][z] |= voxel_is_solid_u32 >> x;
                }
            }
        }
    }
    x_aligned
}
fn get_y_aligned_solid_map(voxels: &[[[VoxelType; 32]; 32]; 32]) -> [[u32; 32]; 32] {
    let mut y_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in voxels.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    y_aligned[z][x] |= voxel_is_solid_u32 >> y;
                }
            }
        }
    }
    y_aligned
}
fn get_z_aligned_solid_map(voxels: &[[[VoxelType; 32]; 32]; 32]) -> [[u32; 32]; 32] {
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in voxels.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    z_aligned[x][y] |= voxel_is_solid_u32 >> z;
                }
            }
        }
    }
    z_aligned
}
pub fn map_visible(
    voxels: &[[[VoxelType; 32]; 32]; 32],
    pos: IVec3,
    chunks: &Chunks,
) -> [ChunkFaces; 6] {
    let mut faces = [ChunkFaces([[0; 32]; 32]); 6];
    // 0 = -x
    // 1 = +x
    // 2 = -y
    // 3 = +y
    // 4 = -z
    // 5 = +z

    let [x_aligned, y_aligned, z_aligned] = get_axis_aligned_solid_maps(voxels);

    fn get_x(chunks: &Chunks, pos: IVec3) -> [[u32; 32]; 32] {
        if let Some(voxels) = chunks
            .get(pos)
            .and_then(|chunk| Some(chunk.read().unwrap().voxels))
        {
            get_x_aligned_solid_map(&voxels)
        } else {
            [[0; 32]; 32]
        }
    }
    fn get_y(chunks: &Chunks, pos: IVec3) -> [[u32; 32]; 32] {
        if let Some(voxels) = chunks
            .get(pos)
            .and_then(|chunk| Some(chunk.read().unwrap().voxels))
        {
            get_y_aligned_solid_map(&voxels)
        } else {
            [[0; 32]; 32]
        }
    }
    fn get_z(chunks: &Chunks, pos: IVec3) -> [[u32; 32]; 32] {
        if let Some(voxels) = chunks
            .get(pos)
            .and_then(|chunk| Some(chunk.read().unwrap().voxels))
        {
            get_z_aligned_solid_map(&voxels)
        } else {
            [[0; 32]; 32]
        }
    }
    let (px, nx, py, ny, pz, nz) = (
        get_x(&chunks, pos + IVec3::new(1, 0, 0)),
        get_x(&chunks, pos + IVec3::new(-1, 0, 0)),
        get_y(&chunks, pos + IVec3::new(0, 1, 0)),
        get_y(&chunks, pos + IVec3::new(0, -1, 0)),
        get_z(&chunks, pos + IVec3::new(0, 0, 1)),
        get_z(&chunks, pos + IVec3::new(0, 0, -1)),
    );
    for i in 0..32 {
        for j in 0..32 {
            faces[0].0[i][j] = x_aligned[i][j] & !((x_aligned[i][j] >> 1) | (nx[i][j] << 31));
            faces[1].0[i][j] = x_aligned[i][j] & !((x_aligned[i][j] << 1) | (px[i][j] >> 31));
            faces[2].0[i][j] = y_aligned[i][j] & !((y_aligned[i][j] >> 1) | (ny[i][j] << 31));
            faces[3].0[i][j] = y_aligned[i][j] & !((y_aligned[i][j] << 1) | (py[i][j] >> 31));
            faces[4].0[i][j] = z_aligned[i][j] & !((z_aligned[i][j] >> 1) | (nz[i][j] << 31));
            faces[5].0[i][j] = z_aligned[i][j] & !((z_aligned[i][j] << 1) | (pz[i][j] >> 31));
        }
    }
    /*
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
    */
    // WARNING! additional step: add non solid blocks back in
    faces
}

use crate::gpu::mesh::Mesh;
pub fn generate_mesh(
    cam_pos: Vec3,
    voxels: &[[[VoxelType; 32]; 32]; 32],
    chunk_pos: IVec3,
    faces: [ChunkFaces; 6],
) -> Mesh {
    let chunk_pos = chunk_pos << 5;
    let mut mesh = Mesh::with_capacity(100);
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                let position: IVec3 = chunk_pos + IVec3::new(x as i32, y as i32, z as i32);

                const MASK_BIT: u32 = 1_u32 << 31;
                if faces[0].0[y][z] & (MASK_BIT >> x) > 0 && cam_pos.x < position.x as f32 {
                    mesh.add_nx(position, voxels[x][y][z].texture())
                }
                if faces[1].0[y][z] & (MASK_BIT >> x) > 0 && cam_pos.x > position.x as f32 + 0.9 {
                    mesh.add_px(position, voxels[x][y][z].texture())
                }
                if faces[2].0[z][x] & (MASK_BIT >> y) > 0 && cam_pos.y < position.y as f32 {
                    mesh.add_ny(position, voxels[x][y][z].texture())
                }
                if faces[3].0[z][x] & (MASK_BIT >> y) > 0 && cam_pos.y > position.y as f32 + 0.9 {
                    mesh.add_py(position, voxels[x][y][z].texture())
                }
                if faces[4].0[x][y] & (MASK_BIT >> z) > 0 && cam_pos.z < position.z as f32 {
                    mesh.add_nz(position, voxels[x][y][z].texture())
                }
                if faces[5].0[x][y] & (MASK_BIT >> z) > 0 && cam_pos.z > position.z as f32 + 0.9 {
                    mesh.add_pz(position, voxels[x][y][z].texture())
                }
            }
        }
    }
    mesh
}
pub fn generate_mesh_without_cam_occ(
    voxels: &[[[VoxelType; 32]; 32]; 32],
    chunk_pos: IVec3,
    faces: [ChunkFaces; 6],
) -> Mesh {
    let chunk_pos = chunk_pos << 5;
    let mut mesh = Mesh::with_capacity(100);
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                let position: IVec3 = chunk_pos + IVec3::new(x as i32, y as i32, z as i32);

                const MASK_BIT: u32 = 1_u32 << 31;
                if faces[0].0[y][z] & (MASK_BIT >> x) > 0 {
                    mesh.add_nx(position, voxels[x][y][z].texture())
                }
                if faces[1].0[y][z] & (MASK_BIT >> x) > 0 {
                    mesh.add_px(position, voxels[x][y][z].texture())
                }
                if faces[2].0[z][x] & (MASK_BIT >> y) > 0 {
                    mesh.add_ny(position, voxels[x][y][z].texture())
                }
                if faces[3].0[z][x] & (MASK_BIT >> y) > 0 {
                    mesh.add_py(position, voxels[x][y][z].texture())
                }
                if faces[4].0[x][y] & (MASK_BIT >> z) > 0 {
                    mesh.add_nz(position, voxels[x][y][z].texture())
                }
                if faces[5].0[x][y] & (MASK_BIT >> z) > 0 {
                    mesh.add_pz(position, voxels[x][y][z].texture())
                }
            }
        }
    }
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
