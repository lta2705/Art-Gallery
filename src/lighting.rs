//! Module C – Lighting
//!
//! Định nghĩa PointLight và hàm set uniforms xuống shader.

use glam::Vec3;
use crate::shader::ShaderProgram;

/// Ánh đèn điểm (Point Light) với hệ số Attenuation
#[derive(Clone, Copy)]
pub struct PointLight {
    pub position:  Vec3,
    pub color:     Vec3,
    /// Hệ số suy giảm – dùng công thức 1/(c + l·d + q·d²)
    pub constant:  f32,
    pub linear:    f32,
    pub quadratic: f32,
}

impl PointLight {
    /// Đèn sáng (bright): ánh sáng trắng vàng, chiếu xa
    pub fn bright_warm(position: Vec3) -> Self {
        Self {
            position,
            color:     Vec3::new(1.0, 0.95, 0.7),
            constant:  1.0,
            linear:    0.07,   // giảm suy hao → sáng hơn
            quadratic: 0.017,
        }
    }
}

/// Set toàn bộ mảng đèn vào uniform của shader.
/// Shader cần struct PointLight với các trường: position, color, constant, linear, quadratic
pub unsafe fn upload_lights(
    gl:      &glow::Context,
    program: &ShaderProgram,
    lights:  &[PointLight],
) {
    program.use_program(gl);
    program.set_int(gl, "u_num_lights", lights.len() as i32);

    for (i, light) in lights.iter().enumerate() {
        let base = format!("u_lights[{}]", i);
        program.set_vec3(gl,  &format!("{}.position",  base), light.position);
        program.set_vec3(gl,  &format!("{}.color",     base), light.color);
        program.set_float(gl, &format!("{}.constant",  base), light.constant);
        program.set_float(gl, &format!("{}.linear",    base), light.linear);
        program.set_float(gl, &format!("{}.quadratic", base), light.quadratic);
    }
}

/// Tạo bố cục đèn cho phòng chữ L: 2 đèn accent tại góc nhánh
/// Main ceiling light is controlled separately via spotlight in main.rs
pub fn default_hallway_lights() -> Vec<PointLight> {
    vec![
        // Accent light at branch corner (inner corner p4 area)
        PointLight::bright_warm(Vec3::new(3.5, 2.9, -3.0)),
        // Accent light at far end of main hallway
        PointLight::bright_warm(Vec3::new(9.0, 2.9, -1.5)),
    ]
}
