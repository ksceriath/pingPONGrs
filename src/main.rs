use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(TheGame)
        .run();
}

fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Ping PONG".to_string(),
            resolution: (WINDOW_WIDTH, WINDOW_LENGTH).into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

const WINDOW_LENGTH: f32 = 700.;
const WINDOW_WIDTH: f32 = 1600.;

const PEDAL_WIDTH: f32 = 20.;
const PEDAL_LENGTH: f32 = 150.;
const PEDAL_GUTTER: f32 = 20.;
const MAX_PEDAL_VELOCITY: f32 = 5.;

const BALL_RADIUS: f32 = 20.;
const BALL_COLOR: Color = Color::YELLOW_GREEN;
const BALL_X: f32 = 0.;

const LEFT_PEDAL_COLOR: Color = Color::CYAN;
const RIGHT_PEDAL_COLOR: Color = Color::BISQUE;

const PEDAL_BOUND: f32 = WINDOW_LENGTH / 2. - PEDAL_GUTTER - PEDAL_LENGTH / 2.;
const LEFT_PEDAL_X: f32 = -(WINDOW_WIDTH / 2. - PEDAL_GUTTER - PEDAL_WIDTH / 2.);
const RIGHT_PEDAL_X: f32 = WINDOW_WIDTH / 2. - PEDAL_GUTTER - PEDAL_WIDTH / 2.;

struct TheGame;

impl Plugin for TheGame {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_entities)
            .add_systems(Update, move_entities)
            .add_systems(Update, keyboard_event_arrow)
            .add_systems(Update, keyboard_event_ws);
    }
}

fn spawn_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let pedal_sprite = |color: Color, x: f32| SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(PEDAL_WIDTH, PEDAL_LENGTH)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(x, 0., 0.),
            ..default()
        },
        ..default()
    };

    commands.spawn((
        pedal_sprite(LEFT_PEDAL_COLOR, LEFT_PEDAL_X),
        PedalLeft,
        Pedal,
        Velocity { x: 0., y: 0. },
    ));

    commands.spawn((
        pedal_sprite(RIGHT_PEDAL_COLOR, RIGHT_PEDAL_X),
        PedalRight,
        Pedal,
        Velocity { x: 0., y: 0. },
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(BALL_X, 0., 0.)),
            ..default()
        },
        Ball,
        Velocity::default(),
    ));
}

fn move_entities(mut query: Query<(&mut Transform, &Velocity, Option<&Pedal>)>) {
    for (mut transform, velocity, maybe_pedal) in query.iter_mut() {
        let displacement = Vec3::new(velocity.x, velocity.y, 0.);
        transform.translation += displacement;

        if maybe_pedal.is_some() {
            transform.translation.y = bound_check_pedals(transform.translation.y);
        }
    }
}

fn bound_check_pedals(pedal_location: f32) -> f32 {
    if pedal_location > PEDAL_BOUND {
        PEDAL_BOUND
    } else if pedal_location < -PEDAL_BOUND {
        -PEDAL_BOUND
    } else {
        pedal_location
    }
}

fn keyboard_event_arrow(
    key_code: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<PedalRight>>,
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.y = compute_velocity(
            velocity.y,
            key_code.pressed(KeyCode::Up),
            key_code.pressed(KeyCode::Down),
        );
    }
}

fn keyboard_event_ws(
    key_code: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<PedalLeft>>,
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.y = compute_velocity(
            velocity.y,
            key_code.pressed(KeyCode::W),
            key_code.pressed(KeyCode::S),
        );
    }
}

fn compute_velocity(velocity: f32, accelerate: bool, decelerate: bool) -> f32 {
    if accelerate {
        (velocity + 1.).min(MAX_PEDAL_VELOCITY)
    } else if decelerate {
        (velocity - 1.).max(-MAX_PEDAL_VELOCITY)
    } else {
        0.
    }
}

#[derive(Component)]
struct PedalLeft;

#[derive(Component)]
struct PedalRight;

#[derive(Component)]
struct Pedal;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Velocity {
    fn default() -> Velocity {
        Velocity { x: -5., y: -3. }
    }
}
