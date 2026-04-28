//! Module A – Geometry & Environment
//!
//! Định nghĩa Vertex, tạo dữ liệu phòng tranh (hành lang dài),
//! hình cầu (The Head), và upload dữ liệu lên GPU.

use glow::HasContext;

/// Cấu trúc đỉnh gồm position (xyz), normal (xyz), tex_coord (uv)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], tex_coord: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            tex_coord,
        }
    }
}

/// Kích thước byte của một Vertex
pub const VERTEX_SIZE: i32 = std::mem::size_of::<Vertex>() as i32;

/// Mesh được upload lên GPU
pub struct Mesh {
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
    pub ebo: glow::Buffer,
    pub index_count: i32,
}

impl Mesh {
    /// Upload vertices + indices lên GPU, thiết lập VAO attributes
    pub unsafe fn new(gl: &glow::Context, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();
        let ebo = gl.create_buffer().unwrap();

        gl.bind_vertex_array(Some(vao));

        // VBO
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck_vertex(vertices),
            glow::STATIC_DRAW,
        );

        // EBO
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck_u32(indices),
            glow::STATIC_DRAW,
        );

        let stride = VERTEX_SIZE;
        // location 0 – position
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        // location 1 – normal
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);
        // location 2 – tex_coord
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, 24);

        gl.bind_vertex_array(None);

        Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
        }
    }

    pub unsafe fn draw(&self, gl: &glow::Context) {
        gl.bind_vertex_array(Some(self.vao));
        gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
        gl.bind_vertex_array(None);
    }
}

// ──────────────────────────────────────────────────────────────
//  Hành lang dài (Long & Rectangular Hallway)
//  Kích thước: rộng 4, cao 3, dài 20 (đơn vị OpenGL)
// ──────────────────────────────────────────────────────────────
pub struct LRoomMeshes {
    pub floor: (Vec<Vertex>, Vec<u32>),
    pub ceiling: (Vec<Vertex>, Vec<u32>),
    pub walls: Vec<(Vec<Vertex>, Vec<u32>)>,
}

/// Tạo dữ liệu đỉnh cho căn phòng hình chữ L
/// Room dimensions: Main Hallway 9.5m × 3m, Branch 3.5m × 3.5m, Height 3.2m
pub fn build_l_room() -> LRoomMeshes {
    const H: f32 = 3.2; // Ceiling height: 3.2m

    // Floor Plan Vertices (X, Z) - Counter-clockwise starting from bottom-left
    // Total Width: 9.5m | Total Depth: 6.5m
    // p1(0,0) → p2(9.5,0) → p3(9.5,-3) → p4(3.5,-3) → p5(3.5,-6.5) → p6(0,-6.5)
    let p1 = [0.0, 0.0];    // Bottom-left (origin)
    let p2 = [9.5, 0.0];    // Bottom-right (end of main hallway)
    let p3 = [9.5, -3.0];   // Main top-right
    let p4 = [3.5, -3.0];   // Inner corner
    let p5 = [3.5, -6.5];   // Branch top-right
    let p6 = [0.0, -6.5];   // Branch top-left

    // Helper tạo tường từ 2 điểm A, B trên XZ
    let create_wall = |a: [f32; 2], b: [f32; 2], normal: [f32; 3]| -> (Vec<Vertex>, Vec<u32>) {
        make_quad(
            [a[0], 0.0, a[1]],
            [b[0], 0.0, b[1]],
            [b[0], H, b[1]],
            [a[0], H, a[1]],
            normal,
            true, // Vẫn giữ UV để sau này có thể dán tranh
        )
    };

    // 5 mảng tường bao quanh (Đã bỏ tường phải p2->p3 để làm cửa)
    let mut walls = Vec::new();
    walls.push(create_wall(p1, p2, [0.0, 0.0, -1.0])); // Đáy (z=0)
    // Đoạn p2 -> p3 (Tường phải chính x=9.5) bị bỏ trống để làm CỬA (Door)
    walls.push(create_wall(p3, p4, [0.0, 0.0, 1.0])); // Bụng chữ L (z=-3)
    walls.push(create_wall(p4, p5, [-1.0, 0.0, 0.0])); // Phải nhánh (x=3.5)
    walls.push(create_wall(p5, p6, [0.0, 0.0, 1.0])); // Đỉnh nhánh (z=-6.5)
    walls.push(create_wall(p6, p1, [1.0, 0.0, 0.0])); // Trái dài (x=0)
    let mut floor_v = Vec::new();
    let mut floor_i = Vec::new();

    // Sàn 1 (Main Hallway): x:[0, 9.5], z:[-3.0, 0.0]
    let (v1, i1) = make_quad(
        [0.0, 0.0, -3.0],
        [9.5, 0.0, -3.0],
        [9.5, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        false,
    );
    // Sàn 2 (Branch Hallway): x:[0, 3.5], z:[-6.5, -3.0]
    let (v2, i2) = make_quad(
        [0.0, 0.0, -6.5],
        [3.5, 0.0, -6.5],
        [3.5, 0.0, -3.0],
        [0.0, 0.0, -3.0],
        [0.0, 1.0, 0.0],
        false,
    );

    // Ghép vertices và indices
    floor_v.extend(v1);
    floor_i.extend(i1);
    let offset = floor_v.len() as u32;
    floor_v.extend(v2);
    floor_i.extend(i2.iter().map(|idx| idx + offset));

    // Trần (Lật từ sàn)
    let mut ceil_v = Vec::new();
    for v in &floor_v {
        let mut cv = v.clone();
        cv.position[1] = H;
        cv.normal = [0.0, -1.0, 0.0];
        ceil_v.push(cv);
    }
    let ceil_i = floor_i.clone();

    LRoomMeshes {
        floor: (floor_v, floor_i),
        ceiling: (ceil_v, ceil_i),
        walls,
    }
}

/// Tạo một mặt phẳng (quad = 2 tam giác) từ 4 điểm góc (counter-clockwise)
fn make_quad(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    p3: [f32; 3],
    normal: [f32; 3],
    texture: bool,
) -> (Vec<Vertex>, Vec<u32>) {
    // UV đơn giản: phủ toàn bộ mặt (0,0)→(1,1)
    let uvs: [[f32; 2]; 4] = if texture {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    } else {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    };
    let verts = vec![
        Vertex {
            position: p0,
            normal,
            tex_coord: uvs[0],
        },
        Vertex {
            position: p1,
            normal,
            tex_coord: uvs[1],
        },
        Vertex {
            position: p2,
            normal,
            tex_coord: uvs[2],
        },
        Vertex {
            position: p3,
            normal,
            tex_coord: uvs[3],
        },
    ];
    let idxs = vec![0, 1, 2, 0, 2, 3];
    (verts, idxs)
}

// ──────────────────────────────────────────────────────────────
//  The Head – UV Sphere (high-fidelity)
// ──────────────────────────────────────────────────────────────
pub fn build_sphere(stacks: u32, slices: u32, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    use std::f32::consts::PI;
    let mut verts = Vec::new();
    let mut idxs = Vec::new();

    for i in 0..=stacks {
        let phi = PI * (i as f32) / (stacks as f32); // 0 → π
        for j in 0..=slices {
            let theta = 2.0 * PI * (j as f32) / (slices as f32); // 0 → 2π

            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();

            let position = [radius * x, radius * y, radius * z];
            let normal = [x, y, z];
            let tex_coord = [j as f32 / slices as f32, i as f32 / stacks as f32];

            verts.push(Vertex {
                position,
                normal,
                tex_coord,
            });
        }
    }

    for i in 0..stacks {
        for j in 0..slices {
            let row0 = i * (slices + 1) + j;
            let row1 = (i + 1) * (slices + 1) + j;
            idxs.extend_from_slice(&[row0, row1, row0 + 1]);
            idxs.extend_from_slice(&[row0 + 1, row1, row1 + 1]);
        }
    }

    (verts, idxs)
}

// ──────────────────────────────────────────────────────────────
//  OBJ Loader (bonus – load bóng đèn)
// ──────────────────────────────────────────────────────────────
/// Load một file .obj đơn giản dùng `tobj`.
/// Trả về (vertices, indices) từ mesh đầu tiên.
pub fn load_obj(path: &str) -> (Vec<Vertex>, Vec<u32>) {
    let (models, _) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .expect("Failed to load OBJ");

    let mesh = &models[0].mesh;
    let n = mesh.positions.len() / 3;
    let mut verts = Vec::with_capacity(n);

    for i in 0..n {
        let px = mesh.positions[3 * i];
        let py = mesh.positions[3 * i + 1];
        let pz = mesh.positions[3 * i + 2];

        let (nx, ny, nz) = if mesh.normals.len() == mesh.positions.len() {
            (
                mesh.normals[3 * i],
                mesh.normals[3 * i + 1],
                mesh.normals[3 * i + 2],
            )
        } else {
            (0.0, 1.0, 0.0)
        };

        let (u, v) = if mesh.texcoords.len() / 2 == n {
            (mesh.texcoords[2 * i], mesh.texcoords[2 * i + 1])
        } else {
            (0.0, 0.0)
        };

        verts.push(Vertex {
            position: [px, py, pz],
            normal: [nx, ny, nz],
            tex_coord: [u, v],
        });
    }

    (verts, mesh.indices.clone())
}

pub struct FramedPainting {
    pub frame: (Vec<Vertex>, Vec<u32>),
    pub art: (Vec<Vertex>, Vec<u32>),
}

/// Tạo một bộ khung tranh 3D và tấm ảnh bên trong
pub fn build_framed_painting(w: f32, h: f32, thick: f32) -> FramedPainting {
    let border = 0.1; // Độ rộng của viền khung
    let mut f_verts = Vec::new();
    let mut f_idxs = Vec::new();

    // Helper để thêm một box vào mảng vertices/indices
    let mut add_box = |min: [f32; 3], max: [f32; 3]| {
        let offset = f_verts.len() as u32;
        let (v, i) = build_box(min, max);
        f_verts.extend(v);
        f_idxs.extend(i.iter().map(|idx| idx + offset));
    };

    // 4 thanh của khung tranh
    // Thanh dưới
    add_box(
        [-w / 2.0 - border, -h / 2.0 - border, 0.0],
        [w / 2.0 + border, -h / 2.0, thick],
    );
    // Thanh trên
    add_box(
        [-w / 2.0 - border, h / 2.0, 0.0],
        [w / 2.0 + border, h / 2.0 + border, thick],
    );
    // Thanh trái
    add_box(
        [-w / 2.0 - border, -h / 2.0, 0.0],
        [-w / 2.0, h / 2.0, thick],
    );
    // Thanh phải
    add_box([w / 2.0, -h / 2.0, 0.0], [w / 2.0 + border, h / 2.0, thick]);

    // Tấm nền tranh (Art) - hơi thụt vào trong khung một chút
    let art = make_quad(
        [-w / 2.0, -h / 2.0, 0.01],
        [w / 2.0, -h / 2.0, 0.01],
        [w / 2.0, h / 2.0, 0.01],
        [-w / 2.0, h / 2.0, 0.01],
        [0.0, 0.0, 1.0],
        true,
    );

    FramedPainting {
        frame: (f_verts, f_idxs),
        art,
    }
}

/// Tạo một hình hộp (6 mặt)
fn build_box(min: [f32; 3], max: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let mut v = Vec::new();
    let mut i = Vec::new();

    let p = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];

    // Front, Back, Left, Right, Top, Bottom
    let faces = [
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),  // Front
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]), // Back
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]), // Left
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),  // Right
        ([3, 7, 6, 2], [0.0, 1.0, 0.0]),  // Top
        ([0, 1, 5, 4], [0.0, -1.0, 0.0]), // Bottom
    ];

    for (face_indices, norm) in faces {
        let offset = v.len() as u32;
        v.push(Vertex::new(p[face_indices[0]], norm, [0.0, 0.0]));
        v.push(Vertex::new(p[face_indices[1]], norm, [1.0, 0.0]));
        v.push(Vertex::new(p[face_indices[2]], norm, [1.0, 1.0]));
        v.push(Vertex::new(p[face_indices[3]], norm, [0.0, 1.0]));
        i.extend_from_slice(&[
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
        ]);
    }

    (v, i)
}

// ──────────────────────────────────────────────────────────────
//  Helper: cast byte slices (tránh unsafe transmute)
// ──────────────────────────────────────────────────────────────
fn bytemuck_vertex(data: &[Vertex]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<Vertex>(),
        )
    }
}

fn bytemuck_u32(data: &[u32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<u32>(),
        )
    }
}

// ──────────────────────────────────────────────────────────────
//  STL Loader (Using mesh-loader)
// ──────────────────────────────────────────────────────────────
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

pub fn load_stl_mesh(path: &str, scale: f32) -> Option<(Vec<Vertex>, Vec<u32>, AABB)> {

    let p = std::path::Path::new(path);
    if !p.exists() {
        println!("Warning: STL file not found at {}. Returning None.", path);
        return None;
    }

    let loader = mesh_loader::Loader::default().merge_meshes(true);
    let scene = match loader.load(p) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load STL {}: {:?}", path, e);
            return None;
        }
    };

    if scene.meshes.is_empty() {
        return None;
    }

    let mesh = &scene.meshes[0];
    let n = mesh.vertices.len();
    let mut verts = Vec::with_capacity(n);
    let mut idxs = Vec::with_capacity(mesh.faces.len() * 3);

    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    let mut computed_normals = vec![[0.0_f32; 3]; n];
    if mesh.normals.is_empty() {
        for face in &mesh.faces {
            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;
            let v0 = glam::Vec3::from(mesh.vertices[i0]);
            let v1 = glam::Vec3::from(mesh.vertices[i1]);
            let v2 = glam::Vec3::from(mesh.vertices[i2]);
            let normal = (v1 - v0).cross(v2 - v0).try_normalize().unwrap_or(glam::Vec3::Z);
            for &idx in &[i0, i1, i2] {
                computed_normals[idx][0] += normal.x;
                computed_normals[idx][1] += normal.y;
                computed_normals[idx][2] += normal.z;
            }
        }
        for norm in &mut computed_normals {
            let n_vec = glam::Vec3::from(*norm).try_normalize().unwrap_or(glam::Vec3::Z);
            *norm = n_vec.into();
        }
    }

    for i in 0..n {
        let v_orig = mesh.vertices[i];
        // Rotate Z-up to Y-up: (x, y, z) -> (x, z, -y)
        let mut v = [
            v_orig[0] * scale,
            v_orig[2] * scale,
            -v_orig[1] * scale
        ];
        
        // Update AABB
        for d in 0..3 {
            if v[d] < min[d] { min[d] = v[d]; }
            if v[d] > max[d] { max[d] = v[d]; }
        }

        let n_orig = if !mesh.normals.is_empty() {
            mesh.normals[i]
        } else {
            computed_normals[i]
        };
        // Rotate normal as well
        let n_vec = [n_orig[0], n_orig[2], -n_orig[1]];

        verts.push(Vertex {
            position: v,
            normal: n_vec,
            tex_coord: [0.0, 0.0],
        });
    }

    for face in &mesh.faces {
        idxs.push(face[0]);
        idxs.push(face[1]);
        idxs.push(face[2]);
    }

    Some((verts, idxs, AABB { min, max }))
}

