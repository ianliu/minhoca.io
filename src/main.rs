use bevy::{prelude::*, sprite::MaterialMesh2dBundle, sprite::Mesh2dHandle};
use rand::prelude::*;
use std::f32::consts::PI;

const BOUNDS: f32 = 1024.0;
const SEGMENT_SIZE: f32 = 32.0;
const MOVEMENT_SPEED: f32 = 200.0;
const TURN_SPEED: f32 = PI;

enum CollisionLayer {
    HEAD,
    SEGMENT,
    FOOD,
    BOUNDS,
}

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

#[derive(Component)]
struct Collider {
    layer: CollisionLayer,
    radius: f32,
}

#[derive(Component, Default)]
struct MinhocaSegments(Vec<Entity>);

#[derive(Component)]
struct Positions;

#[derive(Component)]
struct Food(i32);

#[derive(Resource, Default)]
struct MousePosition(Option<Vec2>);

#[derive(Resource)]
struct FoodTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_world)
        .add_startup_system(setup_minhoca)
        .add_startup_system(setup_bounds)
        // .insert_resource(MinhocaSegments::default())
        .insert_resource(MousePosition::default())
        .insert_resource(FoodTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_system(mouse_position_system.before(player_movement_system))
        .add_system(player_movement_system.before(camera_movement_system))
        .add_system(collision_system)
        .add_system(spawn_food_system)
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
                    transform: Transform::from_xyz((i as f32) * width, (j as f32) * height, 0.0),
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
    let torus_mesh = Mesh::from(shape::Torus {
        radius: BOUNDS + 32.,
        ring_radius: 16.,
        subdivisions_segments: 128,
        subdivisions_sides: 4,
    });
    // Torus is a 3d shape, so we need to ratate it to face the camera.
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle::from(meshes.add(torus_mesh)),
            material: materials.add(ColorMaterial::from(Color::PINK)),
            transform: Transform::from_xyz(0., 0., 1.0)
                .with_rotation(Quat::from_rotation_x(PI / 2.)),
            ..default()
        },
        Collider {
            layer: CollisionLayer::BOUNDS,
            radius: BOUNDS,
        },
    ));
}

fn setup_minhoca(mut commands: Commands, asset_server: Res<AssetServer>) {
    let initial_head_transform = Transform::from_xyz(0., 0., 2.);
    let mut minhoca = vec![commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("minhoca_head.png"),
                transform: initial_head_transform,
                ..default()
            },
            MinhocaHead {
                movement_speed: MOVEMENT_SPEED,
                rotation_speed: TURN_SPEED,
            },
            Collider {
                layer: CollisionLayer::HEAD,
                radius: SEGMENT_SIZE,
            },
            MinhocaSegment,
        ))
        .id()];
    for i in 1..=10 {
        minhoca.push(spawn_segment(
            &mut commands,
            &asset_server,
            SEGMENT_SIZE,
            Vec2::from((0., -(i as f32) * SEGMENT_SIZE)),
        ));
    }
    commands.spawn(MinhocaSegments(minhoca));
}

fn spawn_segment(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    size: f32,
    pos: Vec2,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("minhoca_segment.png"),
                transform: Transform::from_xyz(pos.x, pos.y, 100.0),
                ..default()
            },
            Collider {
                layer: CollisionLayer::SEGMENT,
                radius: size,
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
    mut segment_transforms: Query<&mut Transform, With<MinhocaSegment>>,
    minhoca_segments: Query<&MinhocaSegments>,
    mouse_pos: Res<MousePosition>,
) {
    let delta = time.delta_seconds();
    let mut entities = minhoca_segments.single().0.iter();

    let mut head_transform = segment_transforms
        .get_mut(*entities.next().unwrap())
        .unwrap();

    // get rotation direction
    let rotation_sign = if let Some(mouse) = mouse_pos.0 {
        let movement_direction = (head_transform.rotation * Vec3::Y).truncate();
        let mouse_direction = mouse - head_transform.translation.truncate();
        let side = movement_direction.perp().dot(mouse_direction);
        side.signum()
    } else {
        0.0
    };

    // Move the head
    let movement_speed = head.single().movement_speed;
    let rotation_speed = head.single().rotation_speed;
    let dx = movement_speed * delta;
    let dv = head_transform.rotation * Vec3::Y;

    head_transform.rotate_z(rotation_sign * rotation_speed * delta);
    head_transform.translation += dx * dv;

    // move the body
    let mut head_transform = head_transform.clone();
    entities.for_each(|e| {
        let mut seg_transform = segment_transforms.get_mut(*e).unwrap();
        let seg_direction = (seg_transform.rotation * Vec3::Y).truncate();
        let new_direction = seg_transform.translation - head_transform.translation;
        let new_translation = head_transform.translation
            + new_direction.normalize() * f32::min(SEGMENT_SIZE, new_direction.length());
        let mut new_transform =
            Transform::from_translation(new_translation).with_rotation(seg_transform.rotation);
        new_transform.rotate(Quat::from_rotation_z(
            seg_direction.angle_between(new_direction.truncate()),
        ));
        *seg_transform = new_transform;
        head_transform = new_transform;
    });
}

fn camera_movement_system(
    q_player: Query<(&Transform, &MinhocaHead), Without<MainCamera>>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
) {
    let head_transform = q_player.single().0;
    let mut cam_transform = q_camera.single_mut();
    cam_transform.translation.x = head_transform.translation.x;
    cam_transform.translation.y = head_transform.translation.y;
}

fn check_collision_circles(r1: f32, c1: Vec2, r2: f32, c2: Vec2) -> bool {
    let distance = (c1 - c2).dot(c1 - c2);
    if distance <= (r1 + r2).powi(2) {
        return true;
    }
    false
}

fn collision_system(
    heads: Query<(&Transform, &Collider), With<MinhocaHead>>,
    colliders: Query<(Entity, &Transform, &Collider), (With<Collider>, Without<MinhocaHead>)>,
    mut commands: Commands,
) {
    let (head_pos, head_collider) = heads.single();
    let r1 = head_collider.radius;
    let c1 = head_pos.translation.truncate();

    for (e, transform, collider) in colliders.iter() {
        let r2 = collider.radius;
        let c2 = transform.translation.truncate();
        match collider.layer {
            CollisionLayer::HEAD => (),
            CollisionLayer::BOUNDS => (),
            CollisionLayer::FOOD => {
                if check_collision_circles(r1, c1, r2, c2) {
                    commands.entity(e).despawn();
                    println!("Just ate some food!");
                }
            }
            CollisionLayer::SEGMENT => (),
        }
    }
}

fn spawn_food_system(
    time: Res<Time>,
    mut timer: ResMut<FoodTimer>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut x = thread_rng().gen_range(-BOUNDS..BOUNDS);
        let mut y = thread_rng().gen_range(-BOUNDS..BOUNDS);
        while x * x + y * y > BOUNDS * BOUNDS {
            x = thread_rng().gen_range(-BOUNDS..BOUNDS);
            y = thread_rng().gen_range(-BOUNDS..BOUNDS);
        }
        let food_texture: Handle<Image> = assets.load("food.png");
        commands.spawn((
            SpriteBundle {
                texture: food_texture,
                transform: Transform::from_xyz(x, y, 50.),
                ..default()
            },
            Collider {
                layer: CollisionLayer::FOOD,
                radius: 10.,
            },
            Food(1),
        ));
    }
}
