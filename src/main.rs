use bevy::{prelude::*, sprite::MaterialMesh2dBundle, sprite::Mesh2dHandle};

use std::f32::consts::PI;

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

#[derive(Resource, Default)]
struct MinhocaSegments(Vec<Entity>);

#[derive(Resource, Default)]
struct MousePosition(Option<Vec2>);

const BOUNDS: f32 = 1024.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_world)
        .add_startup_system(setup_minhoca)
        .insert_resource(MinhocaSegments::default())
        .insert_resource(MousePosition::default())
        .add_system(mouse_position_system.before(player_movement_system))
        .add_system(player_movement_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
        transform: Transform::from_translation(Vec3::new(0., 0., 1.0))
            .with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    });
}

fn setup_minhoca(mut commands: Commands, asset_server: Res<AssetServer>, mut segments: ResMut<MinhocaSegments>) {
    *segments = MinhocaSegments(vec![
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("minhoca_head.png"),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
                ..default()
            },
            MinhocaHead {
                movement_speed: 200.0,
                rotation_speed: f32::to_radians(180.0),
            },
        )).id(),
        spawn_segment(&mut commands, &asset_server),
        spawn_segment(&mut commands, &asset_server),
    ]);
}

fn spawn_segment(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("minhoca_segment.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        MinhocaSegment
    )).id()
}

fn mouse_position_system(
    windows: Res<Windows>,
    query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_pos: ResMut<MousePosition>
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
    mut q_player: Query<(&MinhocaHead, &mut Transform), Without<MainCamera>>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    mouse_pos: Res<MousePosition>,
) {
    let delta = time.delta_seconds();
    let (minhoca, mut transform) = q_player.single_mut();

    let rotation_sign = if let Some(mouse) = mouse_pos.0 {
        let movement_direction = (transform.rotation * Vec3::Y).truncate();
        let mouse_direction = mouse - transform.translation.truncate();
        let side = movement_direction.perp().dot(mouse_direction);
        side.signum()
    } else {
        0.0
    };

    transform.rotate_z(rotation_sign * minhoca.rotation_speed * delta);
    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = minhoca.movement_speed * delta;
    transform.translation += movement_direction * movement_distance;

    if transform.translation.length() > BOUNDS {
        transform.translation = transform.translation
            - (transform.translation.length() - BOUNDS) * transform.translation.normalize();
    }

    let mut cam_transform = q_camera.single_mut();
    cam_transform.translation.x = transform.translation.x;
    cam_transform.translation.y = transform.translation.y;
}
