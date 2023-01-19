use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct FoodTimer(Timer);

#[derive(Component)]
struct BackgroundTile;

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_movement_system)
        .add_system(spawn_food)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bg_tile: Handle<Image> = asset_server.load("background-tile.png");
    let snake_handle = asset_server.load("red_circle.png");
    commands.insert_resource(FoodTimer(Timer::new(Duration::from_secs(2), TimerMode::Repeating)));
    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands.spawn((
        SpriteBundle {
            texture: snake_handle.clone(),
            ..default()
        },
        Player {
            movement_speed: 200.0,
            rotation_speed: f32::to_radians(180.0),
        },
    ));
    commands.spawn((
        SpriteBundle {
            texture: bg_tile.clone(),
            ..default()
        },
        BackgroundTile
    ));
}

fn player_movement_system(
    time: Res<Time>,
    windows: Res<Windows>,
    mut q_camera: Query<(&Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
    mut q_player: Query<(&Player, &mut Transform), Without<MainCamera>>,
) {
    let delta = time.delta_seconds();
    let (minhoca, mut transform) = q_player.single_mut();
    let window = windows.get_primary().unwrap();
    let (camera, mut cam_transform, cam_global_transform) = q_camera.single_mut();

    let rotation_sign = if let Some(screen_pos) = window.cursor_position() {
        let win_size = Vec2::new(window.width() as f32, window.height() as f32);
        let ndc = (screen_pos / win_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = cam_global_transform.compute_matrix() * camera.projection_matrix().inverse();
        let pos = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate();

        let movement_direction = (transform.rotation * Vec3::Y).truncate();
        let mouse = Vec2::new(pos.x, pos.y);
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

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
    cam_transform.translation.x = transform.translation.x;
    cam_transform.translation.y = transform.translation.y;
}

fn spawn_food(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodTimer>,
    asset_server: Res<AssetServer>,
) {
    let food_handle = asset_server.load("food.png");

    let mut rng = rand::thread_rng();

    if timer.0.tick(time.delta()).just_finished() {
        let x = rng.gen_range(-BOUNDS.x..BOUNDS.x) / 2.0;
        let y = rng.gen_range(-BOUNDS.y..BOUNDS.y) / 2.0;
        println!("Food spawned at {x}, {y}!");
        commands.spawn((
            SpriteBundle {
                texture: food_handle.clone(),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            Food,
        ));
    }
}
