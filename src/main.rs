use bevy::{prelude::*, time::FixedTimestep};
use rand::Rng;

const TIME_STEP: f32 = 1.0 / 120.0;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_non_send_resource(ElapsedTime(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(player_movement_system)
                .with_system(spawn_food_system),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

/// player component
#[derive(Component)]
struct Player {
    /// linear speed in meters per second
    movement_speed: f32,
    /// rotation speed in radians per second
    rotation_speed: f32,
}

#[derive(Component)]
struct Food {
    amount: f32,
}

#[derive(Resource)]
struct ElapsedTime(Timer);

/// Add the game's entities to our world and creates an orthographic camera for 2D rendering.
///
/// The Bevy coordinate system is the same for 2D and 3D, in terms of 2D this means that:
///
/// * `X` axis goes from left to right (`+X` points right)
/// * `Y` axis goes from bottom to top (`+Y` point up)
/// * `Z` axis goes from far to near (`+Z` points towards you, out of the screen)
///
/// The origin is at the center of the screen.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let snake_handle = asset_server.load("red_circle.png");

    // 2D orthographic camera
    commands.spawn(Camera2dBundle::default());

    // player controlled snake
    commands.spawn((
        SpriteBundle {
            texture: snake_handle.clone(),
            ..default()
        },
        Player {
            movement_speed: 500.0,                  // metres per second
            rotation_speed: f32::to_radians(360.0), // degrees per second
        },
    ));
}

/// Demonstrates applying rotation and movement based on keyboard input.
fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.single_mut();

    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.5;

    if keyboard_input.pressed(KeyCode::Left) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement_factor += 0.5;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        movement_factor -= 1.0;
    }

    // update the ship rotation around the Z axis (perpendicular to the 2D plane of the screen)
    transform.rotate_z(rotation_factor * ship.rotation_speed * TIME_STEP);

    // get the ship's forward vector by applying the current rotation to the ships initial facing vector
    let movement_direction = transform.rotation * Vec3::Y;
    // get the distance the ship will move based on direction, the ship's movement speed and delta time
    let movement_distance = movement_factor * ship.movement_speed * TIME_STEP;
    // create the change in translation using the new movement direction and distance
    let translation_delta = movement_direction * movement_distance;
    // update the ship translation with our new translation delta
    transform.translation += translation_delta;

    // bound the ship within the invisible level bounds
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn spawn_food_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ElapsedTime>,
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
            Food { amount: 1.0 },
        ));
    }
}
