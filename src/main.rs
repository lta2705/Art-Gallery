//! main.rs – 3D Art Gallery (Rust + OpenGL/glow)
//!
//! Module A: geometry.rs  – Hallway, Head, OBJ loader
//! Module B: camera.rs    – WASD, Mouse Look, POV/CCTV toggle
//! Module C: lighting.rs  – Multiple dim PointLights, Phong shaders

mod camera;
mod geometry;
mod lighting;
mod shader;
mod texture;

use camera::{Camera, CameraMode, InputState};
use geometry::{build_framed_painting, build_l_room, build_sphere, load_obj, load_stl_mesh, Mesh};
use lighting::{default_hallway_lights, upload_lights};
use shader::ShaderProgram;
use texture::{bind_texture, load_texture};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use std::time::Instant;

// winit 0.29 imports (correct API)
use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    // ── Window + OpenGL context ──────────────────────────────────────
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_builder = WindowBuilder::new()
        .with_title("3D Art Gallery")
        .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 720u32));

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_multisampling(4);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|a, b| {
                    if a.num_samples() > b.num_samples() {
                        a
                    } else {
                        b
                    }
                })
                .unwrap()
        })
        .unwrap();

    let window = window.unwrap();
    window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .ok();
    window.set_cursor_visible(false);

    let raw_handle = window.raw_window_handle();
    let ctx_attrs = ContextAttributesBuilder::new().build(Some(raw_handle));
    let not_current = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &ctx_attrs)
            .unwrap()
    };

    let inner_size = window.inner_size();
    let surf_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_handle,
        inner_size.width.try_into().unwrap(),
        inner_size.height.try_into().unwrap(),
    );
    let gl_surface: Surface<WindowSurface> = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surf_attrs)
            .unwrap()
    };

    let gl_ctx = not_current.make_current(&gl_surface).unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            gl_config
                .display()
                .get_proc_address(&std::ffi::CString::new(s).unwrap())
        })
    };

    // ── Shaders ──────────────────────────────────────────────────────
    let room_vert = include_str!("../shaders/room.vert");
    let room_frag = include_str!("../shaders/room.frag");
    let sph_vert = include_str!("../shaders/sphere.vert");
    let sph_frag = include_str!("../shaders/sphere.frag");
    let emi_vert = include_str!("../shaders/emissive.vert");
    let emi_frag = include_str!("../shaders/emissive.frag");
    let depth_vert = include_str!("../shaders/depth.vert");
    let depth_frag = include_str!("../shaders/depth.frag");

    let room_shader = unsafe { ShaderProgram::new(&gl, room_vert, room_frag) };
    let sphere_shader = unsafe { ShaderProgram::new(&gl, sph_vert, sph_frag) };
    let emissive_shader = unsafe { ShaderProgram::new(&gl, emi_vert, emi_frag) };
    let depth_shader = unsafe { ShaderProgram::new(&gl, depth_vert, depth_frag) };

    // ── Geometry ─────────────────────────────────────────────────────
    let l_room = build_l_room();
    let (sv, si) = build_sphere(32, 32, 0.4);

    let floor_mesh = unsafe { Mesh::new(&gl, &l_room.floor.0, &l_room.floor.1) };
    let ceiling_mesh = unsafe { Mesh::new(&gl, &l_room.ceiling.0, &l_room.ceiling.1) };

    let mut wall_meshes = Vec::new();
    for (v, i) in &l_room.walls {
        wall_meshes.push(unsafe { Mesh::new(&gl, v, i) });
    }

    let sphere_mesh = unsafe { Mesh::new(&gl, &sv, &si) };

    // Bonus: bóng đèn từ OBJ
    let bulb_mesh_opt: Option<Mesh> = {
        let path = "Models/3d-model.obj";
        if std::path::Path::new(path).exists() {
            let (bv, bi) = load_obj(path);
            Some(unsafe { Mesh::new(&gl, &bv, &bi) })
        } else {
            None
        }
    };

    // Load STL meshes
    let stl_room_opt = load_stl_mesh("../Models/The art gallery.stl");
    let stl_room_mesh = stl_room_opt.map(|(v, i)| unsafe { Mesh::new(&gl, &v, &i) });

    let furniture_opt = load_stl_mesh("../Models/furniture.stl");
    let furniture_mesh = furniture_opt.map(|(v, i)| unsafe { Mesh::new(&gl, &v, &i) });

    // Khung tranh (Framed Paintings)
    let painting_geo = build_framed_painting(1.2, 1.2, 0.05);
    let frame_mesh = unsafe { Mesh::new(&gl, &painting_geo.frame.0, &painting_geo.frame.1) };
    let art_mesh = unsafe { Mesh::new(&gl, &painting_geo.art.0, &painting_geo.art.1) };

    // Vị trí 3 bức ảnh trong phòng chữ L
    let painting_transforms = vec![
        // Art 1: Tường trái, giữa hành lang
        Mat4::from_rotation_translation(
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::new(-1.98, 1.5, -4.0),
        ),
        // Art 2: Tường phải, giữa hành lang
        Mat4::from_rotation_translation(
            Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            Vec3::new(1.98, 1.5, -4.0),
        ),
        // Art 3: Tường cuối hành lang ngang
        Mat4::from_rotation_translation(
            Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            Vec3::new(9.98, 1.5, 4.0),
        ),
    ];

    // ── Textures – 3 bức tranh ───────────────────────────────────────
    let tex_left = unsafe { load_texture(&gl, "assets/art1.jpg") };
    let tex_right = unsafe { load_texture(&gl, "assets/art2.jpg") };
    let tex_end = unsafe { load_texture(&gl, "assets/art3.jpg") };
    let textures = vec![tex_left, tex_right, tex_end];

    // ── Lighting ─────────────────────────────────────────────────────
    let lights = default_hallway_lights();

    // ── Camera & State ───────────────────────────────────────────────
    let mut camera = Camera::new(inner_size.width as f32 / inner_size.height.max(1) as f32);
    let mut input = InputState::default();
    let mut last_frame = Instant::now();

    // ── OpenGL state ─────────────────────────────────────────────────
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);
        gl.enable(glow::CULL_FACE); // Bật culling để tối ưu
        gl.cull_face(glow::BACK);
        gl.clear_color(0.05, 0.05, 0.06, 1.0); // Tăng sáng nền một chút
        gl.enable(glow::MULTISAMPLE); // Bật MSAA
    }

    // ── Shadow Map Setup ─────────────────────────────────────────────
    let shadow_width = 2048;
    let shadow_height = 2048;
    let depth_map_fbo = unsafe { gl.create_framebuffer().unwrap() };
    let depth_map = unsafe { gl.create_texture().unwrap() };
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(depth_map));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::DEPTH_COMPONENT as i32,
            shadow_width,
            shadow_height,
            0,
            glow::DEPTH_COMPONENT,
            glow::FLOAT,
            None,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(depth_map_fbo));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::DEPTH_ATTACHMENT,
            glow::TEXTURE_2D,
            Some(depth_map),
            0,
        );
        gl.draw_buffer(glow::NONE);
        gl.read_buffer(glow::NONE);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    }

    // ──────────────────────── Event loop ─────────────────────────────
    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                // ── Keyboard ─────────────────────────────────────────
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(key),
                                    state,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    let pressed = state == ElementState::Pressed;
                    match key {
                        KeyCode::KeyW | KeyCode::ArrowUp => input.forward = pressed,
                        KeyCode::KeyS | KeyCode::ArrowDown => input.backward = pressed,
                        KeyCode::KeyA | KeyCode::ArrowLeft => input.left = pressed,
                        KeyCode::KeyD | KeyCode::ArrowRight => input.right = pressed,
                        KeyCode::KeyC if pressed => camera.toggle_mode(),
                        KeyCode::Escape if pressed => elwt.exit(),
                        _ => {}
                    }
                }

                // ── Mouse Look ───────────────────────────────────────
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                    ..
                } => {
                    camera.rotate_mouse(dx as f32, dy as f32);
                }

                // ── Resize ───────────────────────────────────────────
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    unsafe { gl.viewport(0, 0, size.width as i32, size.height as i32) };
                    gl_surface.resize(
                        &gl_ctx,
                        size.width.try_into().unwrap(),
                        size.height.try_into().unwrap(),
                    );
                    camera.update_aspect(size.width, size.height);
                }

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => elwt.exit(),

                // ── Render ───────────────────────────────────────────
                Event::AboutToWait => {
                    let now = Instant::now();
                    let dt = now.duration_since(last_frame).as_secs_f32();
                    last_frame = now;

                    // Calculate next position for collision
                    let (fwd, right) = (input.fwd_axis(), input.right_axis());
                    if fwd != 0.0 || right != 0.0 {
                        let dir = (camera.forward() * fwd + camera.right() * right).normalize();
                        let speed = 5.0 * dt;
                        let next_pos = camera.head_pos + dir * speed;

                        // Simple AABB Collision for the L-shaped room walls
                        // Main hallway: x in [-2, 2], z in [-10, 6]
                        // Branch hallway: x in [2, 10], z in [2, 6]
                        let r = 0.4; // head radius
                        let in_main = next_pos.x > -2.0 + r
                            && next_pos.x < 2.0 - r
                            && next_pos.z > -10.0 + r
                            && next_pos.z < 6.0 - r;
                        let in_branch = next_pos.x > 2.0 - r
                            && next_pos.x < 10.0 - r
                            && next_pos.z > 2.0 + r
                            && next_pos.z < 6.0 - r;

                        // Check if current position is already outside (to prevent getting stuck)
                        let curr_in_main = camera.head_pos.x > -2.0 + r
                            && camera.head_pos.x < 2.0 - r
                            && camera.head_pos.z > -10.0 + r
                            && camera.head_pos.z < 6.0 - r;
                        let curr_in_branch = camera.head_pos.x > 2.0 - r
                            && camera.head_pos.x < 10.0 - r
                            && camera.head_pos.z > 2.0 + r
                            && camera.head_pos.z < 6.0 - r;

                        if in_main || in_branch || (!curr_in_main && !curr_in_branch) {
                            camera.move_head(fwd, right, dt);
                        }
                    }

                    let view = camera.view_matrix();
                    let proj = camera.projection_matrix();
                    let eye = match camera.mode {
                        CameraMode::POV => camera.head_pos + Vec3::Y * 0.3,
                        CameraMode::CCTV => camera.cctv_pos,
                    };

                    // Spotlight setup
                    let spot_pos = Vec3::new(0.0, 2.9, 0.0);
                    let spot_dir = Vec3::new(0.0, -1.0, 0.0).normalize();
                    let light_projection =
                        Mat4::perspective_rh_gl(90.0_f32.to_radians(), 1.0, 0.1, 20.0);
                    let light_view =
                        Mat4::look_at_rh(spot_pos, spot_pos + spot_dir, Vec3::new(0.0, 0.0, -1.0));
                    let light_space_matrix = light_projection * light_view;

                    let identity = Mat4::IDENTITY;

                    // Draw function helper
                    let draw_scene = |gl: &glow::Context,
                                      shader: &ShaderProgram,
                                      is_depth_pass: bool| {
                        unsafe {
                            // Floor/Ceiling/Walls or STL Room
                            shader.set_mat4(gl, "u_model", &identity);
                            if let Some(ref stl) = stl_room_mesh {
                                if !is_depth_pass {
                                    shader.set_bool(gl, "u_use_texture", false);
                                    shader.set_vec3(gl, "u_base_color", Vec3::new(0.8, 0.8, 0.8));
                                }
                                stl.draw(gl);
                            } else {
                                if !is_depth_pass {
                                    shader.set_bool(gl, "u_use_texture", false);
                                    shader.set_vec3(gl, "u_base_color", Vec3::new(0.3, 0.3, 0.3));
                                }
                                floor_mesh.draw(gl);
                                if !is_depth_pass {
                                    shader.set_vec3(
                                        gl,
                                        "u_base_color",
                                        Vec3::new(0.15, 0.15, 0.15),
                                    );
                                }
                                ceiling_mesh.draw(gl);
                                if !is_depth_pass {
                                    shader.set_vec3(gl, "u_base_color", Vec3::new(1.0, 1.0, 1.0));
                                }
                                for wm in &wall_meshes {
                                    wm.draw(gl);
                                }
                            }

                            // Furniture
                            if let Some(ref furn) = furniture_mesh {
                                let f_model = Mat4::from_translation(Vec3::new(2.0, 0.0, 4.0))
                                    * Mat4::from_scale(Vec3::splat(0.5));
                                shader.set_mat4(gl, "u_model", &f_model);
                                if !is_depth_pass {
                                    shader.set_bool(gl, "u_use_texture", false);
                                    shader.set_vec3(gl, "u_base_color", Vec3::new(0.6, 0.3, 0.1));
                                }
                                furn.draw(gl);
                            }

                            // Framed Paintings
                            for (i, transform) in painting_transforms.iter().enumerate() {
                                shader.set_mat4(gl, "u_model", transform);
                                if !is_depth_pass {
                                    shader.set_bool(gl, "u_use_texture", false);
                                    shader.set_vec3(gl, "u_base_color", Vec3::new(0.2, 0.1, 0.05));
                                }
                                frame_mesh.draw(gl);

                                if !is_depth_pass {
                                    shader.set_bool(gl, "u_use_texture", true);
                                    shader.set_int(gl, "u_texture", 0);
                                    bind_texture(gl, textures[i], 0);
                                }
                                art_mesh.draw(gl);
                            }
                        }
                    };

                    unsafe {
                        // 1. Render depth map
                        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(depth_map_fbo));
                        gl.viewport(0, 0, shadow_width, shadow_height);
                        gl.clear(glow::DEPTH_BUFFER_BIT);
                        gl.cull_face(glow::FRONT); // Peter panning fix

                        depth_shader.use_program(&gl);
                        depth_shader.set_mat4(&gl, "u_light_space_matrix", &light_space_matrix);

                        draw_scene(&gl, &depth_shader, true);

                        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                        gl.cull_face(glow::BACK); // Restore culling

                        // 2. Render main scene
                        gl.viewport(0, 0, inner_size.width as i32, inner_size.height as i32);
                        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                        // ---- Room ----
                        room_shader.use_program(&gl);
                        room_shader.set_mat4(&gl, "u_view", &view);
                        room_shader.set_mat4(&gl, "u_projection", &proj);
                        room_shader.set_vec3(&gl, "u_view_pos", eye);
                        room_shader.set_mat4(&gl, "u_light_space_matrix", &light_space_matrix);

                        // Setup Spotlight
                        room_shader.set_bool(&gl, "u_use_spotlight", true);
                        room_shader.set_vec3(&gl, "u_spot_light.position", spot_pos);
                        room_shader.set_vec3(&gl, "u_spot_light.direction", spot_dir);
                        room_shader.set_vec3(&gl, "u_spot_light.color", Vec3::new(2.5, 2.5, 2.3)); // Intense light
                        room_shader.set_float(
                            &gl,
                            "u_spot_light.cutOff",
                            25.0_f32.to_radians().cos(),
                        );
                        room_shader.set_float(
                            &gl,
                            "u_spot_light.outerCutOff",
                            35.0_f32.to_radians().cos(),
                        );
                        room_shader.set_float(&gl, "u_spot_light.constant", 1.0);
                        room_shader.set_float(&gl, "u_spot_light.linear", 0.045);
                        room_shader.set_float(&gl, "u_spot_light.quadratic", 0.0075);

                        upload_lights(&gl, &room_shader, &lights);

                        // Bind shadow map
                        gl.active_texture(glow::TEXTURE1);
                        gl.bind_texture(glow::TEXTURE_2D, Some(depth_map));
                        room_shader.set_int(&gl, "u_shadow_map", 1);
                        gl.active_texture(glow::TEXTURE0);

                        draw_scene(&gl, &room_shader, false);

                        // ---- The Head (Chỉ vẽ khi ở chế độ CCTV) ----
                        if camera.mode == CameraMode::CCTV {
                            sphere_shader.use_program(&gl);
                            sphere_shader.set_mat4(&gl, "u_view", &view);
                            sphere_shader.set_mat4(&gl, "u_projection", &proj);
                            sphere_shader.set_vec3(&gl, "u_view_pos", eye);
                            sphere_shader.set_vec3(&gl, "u_sphere_color", Vec3::new(0.8, 0.7, 0.9));
                            let head_model = Mat4::from_translation(camera.head_pos);
                            sphere_shader.set_mat4(&gl, "u_model", &head_model);
                            sphere_mesh.draw(&gl);
                        }

                        // ---- Bulb (emissive) ----
                        emissive_shader.use_program(&gl);
                        emissive_shader.set_mat4(&gl, "u_view", &view);
                        emissive_shader.set_mat4(&gl, "u_projection", &proj);
                        emissive_shader.set_vec3(
                            &gl,
                            "u_emissive_color",
                            Vec3::new(1.0, 0.95, 0.7),
                        );

                        // Draw spotlight bulb
                        let spot_model =
                            Mat4::from_translation(spot_pos) * Mat4::from_scale(Vec3::splat(0.1));
                        emissive_shader.set_mat4(&gl, "u_model", &spot_model);
                        if let Some(ref bm) = bulb_mesh_opt {
                            bm.draw(&gl);
                        } else {
                            sphere_mesh.draw(&gl);
                        }

                        for light in &lights {
                            let bulb_model = Mat4::from_translation(light.position)
                                * Mat4::from_scale(Vec3::splat(0.08));
                            emissive_shader.set_mat4(&gl, "u_model", &bulb_model);
                            if let Some(ref bm) = bulb_mesh_opt {
                                bm.draw(&gl);
                            } else {
                                sphere_mesh.draw(&gl); // fallback
                            }
                        }
                    }
                    gl_surface.swap_buffers(&gl_ctx).unwrap();
                }

                _ => {}
            }
        })
        .unwrap();
}
