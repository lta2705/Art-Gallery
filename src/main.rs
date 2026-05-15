mod read_bin;

use bevy::input::mouse::MouseMotion;
use bevy::pbr::PointLightShadowMap;
use bevy::prelude::*;
use bevy::window::{close_on_esc, CursorGrabMode};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_stl::StlPlugin;
use read_bin::print_mesh_dimensions;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Configure AssetPlugin to use the current directory '.' instead of 'assets'.
            // This allows us to load both "Models/..." and "assets/..." natively.
            file_path: ".".to_string(),
            ..default()
        }))
        .add_plugins(StlPlugin)
        .add_plugins(ObjPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(Msaa::Sample4)
        .insert_resource(LightingSettings {
            spotlight_on: true,
            point_lights_on: true,
            spotlight_intensity: 50000.0,
            point_light_intensity: 7000.0,
        })
        .insert_resource(PointLightShadowMap { size: 1024 })
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                player_movement,
                camera_toggle,
                setup_colliders,
                mouse_look,
                lighting_control,
                update_shadows_by_distance,
                close_on_esc,
                print_mesh_dimensions,
            ),
        )
        .run();
}

/// Marker component for meshes that need a Trimesh collider generated once loaded
#[derive(Component)]
struct NeedsCollider;

/// Marker component for the Player
#[derive(Component)]
struct Player;

/// Marker component for First Person Camera
#[derive(Component)]
struct FpvCamera;

/// Marker component for CCTV Camera
#[derive(Component)]
struct CctvCamera;

/// Resource to track lighting state
#[derive(Resource)]
struct LightingSettings {
    spotlight_on: bool,
    point_lights_on: bool,
    spotlight_intensity: f32,
    point_light_intensity: f32,
}

/// Marker for the main spotlight
#[derive(Component)]
struct GallerySpotlight;

/// Marker for the corner accent lights
#[derive(Component)]
struct GalleryPointLight;

#[derive(Component)]
pub struct UnprocessedMesh;

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_windows: Query<&mut Window>,
) {
    // 1. Cấu hình con trỏ chuột (Khóa chuột vào giữa màn hình và ẩn đi)
    if let Ok(mut window) = q_windows.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    // 2. Ánh sáng môi trường (Giảm để bóng đổ từ đèn trần trông thật hơn)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 40.0,
    });

    // Mặt sàn cố định (Fix cứng để không bị rơi tự do)
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(4.75, -0.05, -3.25)),
        RigidBody::Fixed,
        Collider::cuboid(4.75, 0.05, 3.25),
        Name::new("Fixed-Floor"),
    ));

    // 3. Tải phòng trưng bày (The Art Gallery - STL)
    commands.spawn((
        PbrBundle {
            mesh: asset_server.load("Models/art_gallery.stl"),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            }),
            // Xoay trục từ Z-up (hệ CAD) sang Y-up (hệ Bevy)
            transform: Transform::from_xyz(0.0, 0.0, -6.5)
                .with_rotation(
                    Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                )
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Name::new("Art-Gallery"),
        NeedsCollider, // Hệ thống update sẽ đọc và tạo Trimesh Collider chống xuyên tường
        UnprocessedMesh, // Dùng để in ra kích thước thật 1 lần duy nhất
    ));

    // 4. Tải nội thất (Desk PC - STL)
    let furniture_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.2, 0.0), // Màu nâu gỗ đậm
        metallic: 0.2,                         // Độ kim loại
        perceptual_roughness: 0.1,             // Độ nhám (càng thấp càng bóng)
        reflectance: 0.5,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: asset_server.load("Models/Desk_PC.stl"),
            material: furniture_material,
            // Di chuyển về góc p6(0, -6.5), tăng kích thước và xoay hướng ra hành lang
            transform: Transform::from_xyz(0.5, 0.1, -6.0)
                .with_rotation(
                    Quat::from_rotation_y(std::f32::consts::PI)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                )
                .with_scale(Vec3::splat(0.1)),
            ..default()
        },
        Name::new("Desk-PC"),
        NeedsCollider,
        UnprocessedMesh,
    ));

    // 5. Tải ảnh tranh vẽ và gắn vào các khối Quad (Kèm khung)
    let art1_texture = asset_server.load("src/assets/art1.jpg");
    let art2_texture = asset_server.load("src/assets/art2.jpg");
    let art3_texture = asset_server.load("src/assets/art3.jpg");

    // Bức tranh 1: Tường trái
    spawn_painting_with_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        art1_texture,
        Transform::from_xyz(0.01, 1.5, -1.5)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
    );

    // Bức tranh 2: Tường phải
    spawn_painting_with_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        art2_texture,
        Transform::from_xyz(6.5, 1.5, -3.49),
        // .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
    );

    // Bức tranh 3: Tường sau
    spawn_painting_with_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        art3_texture,
        Transform::from_xyz(1.75, 1.5, -6.49),
    );

    // 6. Hệ thống đèn trần (Phân bổ 3 bóng đều dọc hành lang chính)
    let bulb_mesh = asset_server.load("Models/eb_ceiling_light_01.obj");

    // Config vật liệu Thủy tinh/Kính cho Bulb
    let bulb_mat = materials.add(StandardMaterial {
            // Màu gốc của kính (A = 0.2 để gần như trong suốt)
            base_color: Color::rgba(0.9, 0.9, 1.0, 0.2), 
            // Emissive tạo hiệu ứng ánh sáng rực lên từ bên trong kính
            emissive: Color::rgb(15.0, 15.0, 8.0),
            // Độ nhám thấp (0.05) để kính có độ phản chiếu bề mặt bóng loáng
            perceptual_roughness: 0.05,
            metallic: 0.0,
            // Độ truyền sáng (Bevy 0.13+) giúp vật liệu thực sự trông giống thủy tinh
            specular_transmission: 1.0,
            thickness: 0.05, // Độ dày thành kính
            ior: 1.5,       // Chỉ số khúc xạ của thủy tinh
            // QUAN TRỌNG: Phải có chế độ Blend để render xuyên thấu
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

    let hallway_light_positions = vec![
        Vec3::new(1.5, 3.1, -1.5),
        Vec3::new(4.75, 3.1, -1.5),
        Vec3::new(8.0, 3.1, -1.5),
        Vec3::new(1.75, 3.1, -4.75),
    ];

    for (i, pos) in hallway_light_positions.into_iter().enumerate() {
        commands
            .spawn((
                PointLightBundle {
                    point_light: PointLight {
                        intensity: 7000.0,
                        shadows_enabled: true,
                        range: 15.0,
                        radius: 0.15, // Tạo bóng đổ mềm (Soft Shadows)
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                GalleryPointLight,
                Name::new(format!("Hallway-Light-{}", i + 1)),
            ))
            .with_children(|p| {
                p.spawn((
                    PbrBundle {
                        mesh: bulb_mesh.clone(),
                        material: bulb_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.1, 0.0)
                            .with_scale(Vec3::splat(0.01)),
                        ..default()
                    },
                    Name::new(format!("Bulb-Mesh-{}", i + 1)),
                    UnprocessedMesh,
                ));
            });
    }

    // 7. Đèn spotlight chính (togglable bằng phím L, điều chỉnh bằng [ / ])
    commands.spawn((
        SpotLightBundle {
            spot_light: SpotLight {
                intensity: 50000.0,
                shadows_enabled: true,
                range: 20.0,
                radius: 0.2,
                outer_angle: 1.2,
                inner_angle: 0.8,
                ..default()
            },
            transform: Transform::from_xyz(4.75, 3.2, -1.5)
                .looking_at(Vec3::new(4.75, 0.0, -1.5), Vec3::Y),
            ..default()
        },
        GallerySpotlight,
        Name::new("Main-Spotlight"),
    ));

    // 9. Player (đầu người) - spawn ngay vì Fixed-Floor collider đã có sẵn
    let player_id = commands
        .spawn((
            Player,
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.4)),
                material: materials.add(Color::rgb(0.8, 0.7, 0.9)),
                transform: Transform::from_xyz(4.0, 0.5, -1.5),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::ball(0.4),
            bevy_rapier3d::prelude::Ccd::enabled(),
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
        ))
        .id();

    commands.entity(player_id).with_children(|parent| {
        parent.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.2, 0.0),
                camera: Camera {
                    is_active: true,
                    ..default()
                },
                ..default()
            },
            FpvCamera,
        ));
    });

    // 10. Camera an ninh (CCTV Camera)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.75, 3.0, -1.5)
                .looking_at(Vec3::new(4.75, 0.0, -3.25), Vec3::Y),
            camera: Camera {
                is_active: false, // Mặc định tắt, chỉ bật khi nhấn phím 'C'
                ..default()
            },
            ..default()
        },
        CctvCamera,
        Name::new("CCTV-Camera"),
    ));
}

/// Helper function to spawn a painting with a procedural frame
fn spawn_painting_with_frame(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    texture: Handle<Image>,
    transform: Transform,
) {
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.1, 0.05),
        perceptual_roughness: 0.7,
        ..default()
    });

    // Painting Quad
    commands.spawn(PbrBundle {
        mesh: meshes.add(Rectangle::new(1.2, 1.2)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            ..default()
        }),
        transform,
        ..default()
    });

    // Procedural Frame (4 pieces)
    let w = 1.3;
    let t = 0.05;
    let offset = 0.6;
    let frame_mesh = meshes.add(Cuboid::new(w, t, t));
    let frame_mesh_v = meshes.add(Cuboid::new(t, w, t));

    // Top
    commands.spawn(PbrBundle {
        mesh: frame_mesh.clone(),
        material: frame_mat.clone(),
        transform: transform * Transform::from_xyz(0.0, offset, 0.0),
        ..default()
    });
    // Bottom
    commands.spawn(PbrBundle {
        mesh: frame_mesh.clone(),
        material: frame_mat.clone(),
        transform: transform * Transform::from_xyz(0.0, -offset, 0.0),
        ..default()
    });
    // Left
    commands.spawn(PbrBundle {
        mesh: frame_mesh_v.clone(),
        material: frame_mat.clone(),
        transform: transform * Transform::from_xyz(-offset, 0.0, 0.0),
        ..default()
    });
    // Right
    commands.spawn(PbrBundle {
        mesh: frame_mesh_v.clone(),
        material: frame_mat.clone(),
        transform: transform * Transform::from_xyz(offset, 0.0, 0.0),
        ..default()
    });
}

/// System to handle asynchronous generation of Colliders from meshes
/// Polls `Assets<Mesh>` until STL/OBJ meshes are loaded,
/// then generates colliders for collision detection.
/// - Room (L-shape): Trimesh for precise wall collision
/// - Furniture (Desk_PC): AABB Cuboid for performance
fn setup_colliders(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>, Option<&Name>), With<NeedsCollider>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh_handle, name) in query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            let is_furniture = name
                .and_then(|n| Some(n.as_str() == "Desk-PC"))
                .unwrap_or(false);

            if is_furniture {
                if let Some(half) = mesh_aabb_half_extents(mesh) {
                    commands
                        .entity(entity)
                        .insert(Collider::cuboid(half.x, half.y, half.z))
                        .insert(RigidBody::Fixed)
                        .remove::<NeedsCollider>();
                }
            } else if let Some(collider) =
                Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh)
            {
                commands
                    .entity(entity)
                    .insert(collider)
                    .insert(RigidBody::Fixed)
                    .remove::<NeedsCollider>();
            }
        }
    }
}

/// Compute AABB half-extents from mesh vertex positions.
fn mesh_aabb_half_extents(mesh: &Mesh) -> Option<Vec3> {
    use bevy::render::mesh::VertexAttributeValues;
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for pos in positions {
            let p = Vec3::from_slice(pos);
            min = min.min(p);
            max = max.max(p);
        }
        Some((max - min) / 2.0)
    } else {
        None
    }
}

/// System to handle WASD Movement relative to player's facing direction
/// Resets player to start position if they fall below threshold.
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = q_player.get_single_mut() {
        const RESET_Y: f32 = -5.0;
        if transform.translation.y < RESET_Y {
            transform.translation = Vec3::new(4.0, 0.5, -1.5);
            velocity.linvel = Vec3::ZERO;
            println!("Player fell out of bounds — reset to start position");
            return;
        }

        let mut dir = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            dir.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }

        if dir.length_squared() > 0.0 {
            dir = transform.rotation * dir.normalize();
        }

        let speed = 5.0;
        velocity.linvel.x = dir.x * speed;
        velocity.linvel.z = dir.z * speed;
        // Notice: We don't overwrite Y velocity, allowing gravity to pull the player down.
    }
}

/// System to handle Mouse Look
fn mouse_look(
    mut q_camera: Query<(&Camera, &mut Transform), (With<FpvCamera>, Without<Player>)>,
    mut q_player: Query<&mut Transform, (With<Player>, Without<FpvCamera>)>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    if let Ok((camera, mut camera_transform)) = q_camera.get_single_mut() {
        if !camera.is_active {
            return;
        } // Don't move mouse if CCTV is active

        let sensitivity = 0.002;

        if let Ok(mut player_transform) = q_player.get_single_mut() {
            player_transform.rotate_y(-delta.x * sensitivity);
        }

        let (mut _yaw, mut pitch, mut _roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        pitch -= delta.y * sensitivity;
        pitch = pitch.clamp(-1.5, 1.5);
        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
    }
}

/// System to handle keyboard lighting controls
/// Keys: L=toggle spotlight, P=toggle point lights,
///       [=decrease spotlight intensity, ]=increase spotlight intensity
fn lighting_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<LightingSettings>,
    mut q_spot: Query<&mut SpotLight, With<GallerySpotlight>>,
    mut q_point: Query<&mut PointLight, With<GalleryPointLight>>,
) {
    const MIN_INTENSITY: f32 = 0.0;
    const MAX_INTENSITY: f32 = 200000.0;

    if keyboard.just_pressed(KeyCode::KeyL) {
        settings.spotlight_on = !settings.spotlight_on;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        settings.point_lights_on = !settings.point_lights_on;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        settings.spotlight_intensity =
            (settings.spotlight_intensity * 0.95).max(MIN_INTENSITY);
        settings.point_light_intensity =
            (settings.point_light_intensity * 0.95).max(MIN_INTENSITY);
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        settings.spotlight_intensity =
            (settings.spotlight_intensity * 1.05).min(MAX_INTENSITY);
        settings.point_light_intensity =
            (settings.point_light_intensity * 1.05).min(MAX_INTENSITY);
    }
    if keyboard.pressed(KeyCode::BracketRight) {
        settings.spotlight_intensity = (settings.spotlight_intensity * 1.05).min(MAX_INTENSITY);
    }

    for mut light in q_spot.iter_mut() {
        light.intensity = if settings.spotlight_on {
            settings.spotlight_intensity
        } else {
            0.0
        };
    }
    for mut light in q_point.iter_mut() {
        light.intensity = if settings.point_lights_on {
            settings.point_light_intensity
        } else {
            0.0
        };
    }
}

/// System to auto-enable/disable shadows based on distance to player.
/// Lights within 8m get shadows enabled; farther lights skip shadow rendering for performance.
fn update_shadows_by_distance(
    player: Query<&Transform, With<Player>>,
    mut q_point: Query<(&Transform, &mut PointLight), With<GalleryPointLight>>,
    mut q_spot: Query<(&Transform, &mut SpotLight), With<GallerySpotlight>>,
) {
    let Ok(player_tf) = player.get_single() else {
        return;
    };

    let dist_threshold = 8.0;

    for (light_tf, mut light) in &mut q_point {
        let dist = player_tf.translation.distance(light_tf.translation);
        light.shadows_enabled = dist < dist_threshold;
    }

    for (light_tf, mut light) in &mut q_spot {
        let dist = player_tf.translation.distance(light_tf.translation);
        light.shadows_enabled = dist < dist_threshold;
    }
}

/// System to toggle between FPV and CCTV Cameras
/// Math / Logic:
/// For the camera toggle, we utilize the `is_active` boolean on the Camera component.
/// By applying logical NOT (`!`), we invert their states.
/// - The FPV Camera inherits its transform via hierarchical math: M_final = M_player * M_fpv_local.
/// - The CCTV Camera calculates its transform via LookAt math: M_cctv = Translation(Corner) * LookRotation(Target - Corner).
fn camera_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q_fpv: Query<&mut Camera, (With<FpvCamera>, Without<CctvCamera>)>,
    mut q_cctv: Query<&mut Camera, (With<CctvCamera>, Without<FpvCamera>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let (Ok(mut fpv), Ok(mut cctv)) = (q_fpv.get_single_mut(), q_cctv.get_single_mut()) {
            fpv.is_active = !fpv.is_active;
            cctv.is_active = !cctv.is_active;
        }
    }
}
