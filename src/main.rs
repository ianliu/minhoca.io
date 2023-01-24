use bevy::{prelude::*, sprite::MaterialMesh2dBundle, sprite::Mesh2dHandle};

use std::f32::consts::PI;

const BOUNDS: f32 = 1024.0;
const SEGMENT_DIST: f32 = 32.0;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct BackgroundTile;

#[derive(Component)]
struct MinhocaHead {
    movement_speed: f32,
    rotation_speed: f32,
}

#[derive(Component)]
struct MinhocaSegment;

#[derive(Resource, Default, Deref, DerefMut)]
struct MinhocaSegments(Vec<Entity>);

#[derive(Component)]
struct Positions;

#[derive(Resource, Default)]
struct MousePosition(Option<Vec2>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_world)
        .add_startup_system(setup_minhoca)
        .add_startup_system(setup_bounds)
        .insert_resource(MinhocaSegments::default())
        .insert_resource(MousePosition::default())
        .add_system(mouse_position_system.before(player_movement_system))
        .add_system(player_movement_system.before(camera_movement_system))
        .add_system(camera_movement_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bg_tile: Handle<Image> = asset_server.load("background-tile.png");
    commands.spawn((Camera2dBundle::default(), MainCamera));

    // TODO: get width and height from bg_tile
    let width = 512.0;
    let height = 591.0;
    for i in -5..5 {
        for j in -5..5 {
            commands.spawn((
                SpriteBundle {
                    texture: bg_tile.clone(),
                    transform: Transform::from_xyz((i as f32) * width, (j as f32) * height, 1.0),
                    ..default()
                },
                BackgroundTile,
            ));
        }
    }
}

fn setup_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Torus is a 3d shape, so we need to ratate it to face the camera.
    let torus_mesh = Mesh::from(shape::Torus {
        radius: BOUNDS + 16.,
        ring_radius: 16.,
        subdivisions_segments: 128,
        subdivisions_sides: 4,
    });
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle::from(meshes.add(torus_mesh)),
        material: materials.add(ColorMaterial::from(Color::PINK)),
        transform: Transform::from_xyz(0., 0., 1.0).with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    });
}

fn setup_minhoca(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut segments: ResMut<MinhocaSegments>,
) {
    *segments = MinhocaSegments(vec![
        commands
            .spawn((
                SpriteBundle {
                    texture: asset_server.load("minhoca_head.png"),
                    transform: Transform::from_xyz(0.0, 0.0, 2.0),
                    ..default()
                },
                MinhocaHead {
                    movement_speed: 200.0,
                    rotation_speed: f32::to_radians(180.0),
                },
                MinhocaSegment,
            ))
            .id(),
        spawn_segment(&mut commands, &asset_server, 0., -1. * SEGMENT_DIST),
        spawn_segment(&mut commands, &asset_server, 0., -2. * SEGMENT_DIST),
        spawn_segment(&mut commands, &asset_server, 0., -3. * SEGMENT_DIST),
        spawn_segment(&mut commands, &asset_server, 0., -4. * SEGMENT_DIST),
        spawn_segment(&mut commands, &asset_server, 0., -5. * SEGMENT_DIST),
    ]);
}

fn spawn_segment(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("minhoca_segment.png"),
                transform: Transform::from_xyz(x, y, 2.0),
                ..default()
            },
            MinhocaSegment,
        ))
        .id()
}

fn mouse_position_system(
    windows: Res<Windows>,
    query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_pos: ResMut<MousePosition>,
) {
    let window = windows.get_primary().unwrap();
    let (camera, cam_global_transform) = query.single();
    *mouse_pos = if let Some(screen_pos) = window.cursor_position() {
        let win_size = Vec2::new(window.width(), window.height());
        let ndc = (screen_pos / win_size) * 2.0 - Vec2::ONE;
        let ndc_to_world =
            cam_global_transform.compute_matrix() * camera.projection_matrix().inverse();
        let pos = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate();
        MousePosition(Some(Vec2::new(pos.x, pos.y)))
    } else {
        MousePosition(None)
    };
}

fn player_movement_system(
    time: Res<Time>,
    head: Query<&MinhocaHead>,
    mut minhoca: Query<(&mut Transform, &MinhocaSegment)>,
    mouse_pos: Res<MousePosition>,
) {
    let delta = time.delta_seconds();
    let mut segments = minhoca.iter_mut();
    let (mut head_transform, _) = segments.next().unwrap();

    let rotation_sign = if let Some(mouse) = mouse_pos.0 {
        let movement_direction = (head_transform.rotation * Vec3::Y).truncate();
        let mouse_direction = mouse - head_transform.translation.truncate();
        let side = movement_direction.perp().dot(mouse_direction);
        side.signum()
    } else {
        0.0
    };

    let minhoca_head = head.single();
    head_transform.rotate_z(rotation_sign * minhoca_head.rotation_speed * delta);
    let dx = minhoca_head.movement_speed * delta;
    let dv = head_transform.rotation * Vec3::Y;
    head_transform.translation += dx * dv;

    let mut head_pos = head_transform.translation;
    for (mut transform, _) in segments {
        let old_pos = transform.translation;
        let direction = old_pos - head_pos;
        let new_pos = head_pos + direction.normalize() * f32::min(SEGMENT_DIST, direction.length());
        transform.translation = new_pos;
        head_pos = new_pos;
    }
}

fn camera_movement_system(
    q_player: Query<(&Transform, With<MinhocaHead>), Without<MainCamera>>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
) {
    let transform = q_player.single().0;
    let mut cam_transform = q_camera.single_mut();
    cam_transform.translation.x = transform.translation.x;
    cam_transform.translation.y = transform.translation.y;
}
