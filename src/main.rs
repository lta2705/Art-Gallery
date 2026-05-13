use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{close_on_esc, CursorGrabMode};
use bevy_rapier3d::prelude::*;
use bevy_stl::StlPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Configure AssetPlugin to use the current directory '.' instead of 'assets'.
            // This allows us to load both "Models/..." and "assets/..." natively.
            file_path: ".".to_string(),
            ..default()
        }))
        .add_plugins(StlPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(Msaa::Sample4)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                player_movement,
                camera_toggle,
                setup_colliders,
                mouse_look,
                close_on_esc,
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

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_windows: Query<&mut Window>,
) {
    // Cursor grab
    if let Ok(mut window) = q_windows.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    // 1. Ambient Light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 150.0,
    });

    // 2. Load STL Room (The Art Gallery)
    commands.spawn((
        PbrBundle {
            mesh: asset_server.load("Models/art_gallery.stl"),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            }),
            // The original loaded mesh needs to be rotated -90 around X (to make Z up become Y up)
            // Then rotated -90 around Y, and translated.
            transform: Transform::from_xyz(0.0, 0.0, -6.5)
                .with_rotation(
                    Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
                        * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                )
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        NeedsCollider, // Will attach Trimesh collider in update system
    ));

    // 3. Load Furniture
    let furniture_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.2, 0.0), // Màu nâu gỗ đậm
        metallic: 0.2,                         // Độ kim loại (0.0 đến 1.0)
        perceptual_roughness: 0.1,             // Độ nhám (càng thấp càng bóng)
        reflectance: 0.5,                      // Độ phản chiếu ánh sáng
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("Models/Desk_PC.stl"),
        material: furniture_material,
        transform: Transform::from_xyz(1.5, 0.5, -5.5) // Nâng Y lên 0.5 để không lún sàn
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.1)),
        ..default()
    });

    // 4. Load Painting Images and map to Quads (Rectangles)
    let art1_texture = asset_server.load("src/assets/art1.jpg");
    let art2_texture = asset_server.load("src/assets/art2.jpg");
    let art3_texture = asset_server.load("src/assets/art3.jpg");

    let painting_transforms = vec![
        // Art 1: Left wall of main hallway
        (
            art1_texture,
            Transform::from_xyz(0.1, 1.5, -1.5)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ),
        // Art 2: Inner wall
        (art2_texture, Transform::from_xyz(6.5, 1.5, -2.9)),
        // Art 3: Back wall of branch hallway
        (art3_texture, Transform::from_xyz(1.75, 1.5, -6.4)),
    ];

    let quad_mesh = meshes.add(Rectangle::new(1.2, 1.2));

    for (texture, transform) in painting_transforms {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: false,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: quad_mesh.clone(),
            material,
            transform,
            ..default()
        });
    }

    // 5. Spotlight pointing at the painting
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            intensity: 50000.0, // High intensity gallery spotlight
            shadows_enabled: true,
            range: 20.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 3.0, -1.0)
            .looking_at(Vec3::new(4.0, 1.5, -2.9), Vec3::Y),
        ..default()
    });

    // CCTV Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.75, 3.0, -1.5)
                .looking_at(Vec3::new(4.75, 0.0, -3.25), Vec3::Y),
            camera: Camera {
                is_active: false, // Inactive by default
                ..default()
            },
            ..default()
        },
        CctvCamera,
    ));
}

/// System to handle asynchronous generation of Colliders from meshes
/// Math / Logic:
/// Since STL loading is asynchronous, we poll the `Assets<Mesh>` resource.
/// Once the raw vertices and indices are available, `Collider::from_bevy_mesh`
/// uses the exact geometry data to compute a precise `Trimesh` for collision detection.
/// This ensures the player's Sphere collider interacts perfectly with the L-shaped room walls.
fn setup_colliders(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>), With<NeedsCollider>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(), With<Player>>,
) {
    for (entity, mesh_handle) in query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            // Generate Trimesh from STL
            if let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh)
            {
                commands
                    .entity(entity)
                    .insert(collider)
                    .insert(RigidBody::Fixed) // Ensure the room is static
                    .remove::<NeedsCollider>();

                // Spawn player ONLY after the floor collider is ready to prevent falling into the void
                if player_query.is_empty() {
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
                }
            }
        }
    }
}

/// System to handle WASD Movement relative to player's facing direction
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((transform, mut velocity)) = q_player.get_single_mut() {
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
