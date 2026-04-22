//! Module – Texture loading
//!
//! Load ảnh từ file, upload lên OpenGL texture object.

use glow::HasContext;

/// Load ảnh bằng `image` crate, upload lên GPU, trả về Texture ID
pub unsafe fn load_texture(gl: &glow::Context, path: &str) -> glow::Texture {
    let img = image::open(path)
        .unwrap_or_else(|_| panic!("Cannot open texture: {}", path))
        .to_rgba8();

    let (width, height) = img.dimensions();
    let raw = img.into_raw();

    let tex = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(tex));

    // Cài đặt wrapping và filtering
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S,     glow::CLAMP_TO_EDGE as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T,     glow::CLAMP_TO_EDGE as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        Some(&raw),
    );
    gl.generate_mipmap(glow::TEXTURE_2D);
    gl.bind_texture(glow::TEXTURE_2D, None);

    tex
}

/// Bind texture lên texture unit (0, 1, 2, ...)
pub unsafe fn bind_texture(gl: &glow::Context, tex: glow::Texture, unit: u32) {
    gl.active_texture(glow::TEXTURE0 + unit);
    gl.bind_texture(glow::TEXTURE_2D, Some(tex));
}
