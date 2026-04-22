//! Module – Shader Program helper
//!
//! Biên dịch, link Vertex + Fragment shader, set uniforms.

use glow::{HasContext, NativeProgram};
use glam::{Mat4, Vec3};

pub struct ShaderProgram {
    pub id: NativeProgram,
}

impl ShaderProgram {
    /// Biên dịch và link vertex + fragment shader từ source strings
    pub unsafe fn new(gl: &glow::Context, vert_src: &str, frag_src: &str) -> Self {
        let vert = compile_shader(gl, glow::VERTEX_SHADER,   vert_src);
        let frag = compile_shader(gl, glow::FRAGMENT_SHADER, frag_src);

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vert);
        gl.attach_shader(program, frag);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            panic!("Shader link error: {}", gl.get_program_info_log(program));
        }

        gl.delete_shader(vert);
        gl.delete_shader(frag);

        Self { id: program }
    }

    pub unsafe fn use_program(&self, gl: &glow::Context) {
        gl.use_program(Some(self.id));
    }

    pub unsafe fn set_int(&self, gl: &glow::Context, name: &str, val: i32) {
        let loc = gl.get_uniform_location(self.id, name);
        gl.uniform_1_i32(loc.as_ref(), val);
    }

    pub unsafe fn set_bool(&self, gl: &glow::Context, name: &str, val: bool) {
        self.set_int(gl, name, val as i32);
    }

    pub unsafe fn set_mat4(&self, gl: &glow::Context, name: &str, mat: &Mat4) {
        let loc = gl.get_uniform_location(self.id, name);
        gl.uniform_matrix_4_f32_slice(loc.as_ref(), false, &mat.to_cols_array());
    }

    pub unsafe fn set_vec3(&self, gl: &glow::Context, name: &str, v: Vec3) {
        let loc = gl.get_uniform_location(self.id, name);
        gl.uniform_3_f32(loc.as_ref(), v.x, v.y, v.z);
    }

    pub unsafe fn set_float(&self, gl: &glow::Context, name: &str, v: f32) {
        let loc = gl.get_uniform_location(self.id, name);
        gl.uniform_1_f32(loc.as_ref(), v);
    }
}

unsafe fn compile_shader(gl: &glow::Context, kind: u32, src: &str) -> glow::Shader {
    let shader = gl.create_shader(kind).unwrap();
    gl.shader_source(shader, src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!("Shader compile error:\n{}", gl.get_shader_info_log(shader));
    }
    shader
}
