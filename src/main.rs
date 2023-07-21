use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
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

const BORDER_LENGTH: f32 = 1560.;
const BORDER_THICKNESS: f32 = 5.;
const TOP_BORDER_Y: f32 = 330.;
const BOTTOM_BORDER_Y: f32 = -330.;

const PEDAL_WIDTH: f32 = 20.;
const PEDAL_LENGTH: f32 = 150.;
const PEDAL_GUTTER: f32 = 20.;
const MAX_PEDAL_VELOCITY: f32 = 10.;
const PEDAL_VELOCITY_INCREMENTS: f32 = 4.;

const BALL_RADIUS: f32 = 20.;
const BALL_COLOR: Color = Color::YELLOW_GREEN;
const BALL_X: f32 = 0.;

const BALL_START_SPEED_X: f32 = -5.;
const BALL_START_SPEED_Y: f32 = 3.;
const BOUNCE_SPEED_BONUS: f32 = 0.75;
const MAX_BALL_SPEED: f32 = 15.;

const LEFT_PEDAL_COLOR: Color = Color::CYAN;
const RIGHT_PEDAL_COLOR: Color = Color::BISQUE;

const PEDAL_BOUND: f32 = WINDOW_LENGTH / 2. - PEDAL_GUTTER - PEDAL_LENGTH / 2.;
const LEFT_PEDAL_X: f32 = -(WINDOW_WIDTH / 2. - PEDAL_GUTTER - PEDAL_WIDTH / 2.);
const RIGHT_PEDAL_X: f32 = WINDOW_WIDTH / 2. - PEDAL_GUTTER - PEDAL_WIDTH / 2.;

struct TheGame;

impl Plugin for TheGame {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_systems(Startup, spawn_entities)
            .add_systems(OnEnter(GameState::Stop), reset_ball)
            .add_systems(OnEnter(GameState::Play), set_ball_velocity)
            .add_systems(Update, play_game.run_if(in_state(GameState::Stop)))
            .add_systems(
                Update,
                (
                    move_entities,
                    keyboard_event_arrow,
                    keyboard_event_ws,
                    ball_collision,
                    game_over,
                )
                    .run_if(in_state(GameState::Play)),
            );
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
        Collider,
        Velocity::zero(),
        Dimensions(Vec2::new(PEDAL_WIDTH, PEDAL_LENGTH)),
    ));

    commands.spawn((
        pedal_sprite(RIGHT_PEDAL_COLOR, RIGHT_PEDAL_X),
        PedalRight,
        Pedal,
        Collider,
        Velocity::zero(),
        Dimensions(Vec2::new(PEDAL_WIDTH, PEDAL_LENGTH)),
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(BALL_X, 0., 0.)),
            ..default()
        },
        Ball,
        Velocity::zero(),
        Dimensions(Vec2::new(BALL_RADIUS, BALL_RADIUS)),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(BORDER_LENGTH, BORDER_THICKNESS)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., TOP_BORDER_Y, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        Dimensions(Vec2::new(BORDER_LENGTH, BORDER_THICKNESS)),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(BORDER_LENGTH, BORDER_THICKNESS)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., BOTTOM_BORDER_Y, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        Dimensions(Vec2::new(BORDER_LENGTH, BORDER_THICKNESS)),
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

fn game_over(
    mut query: Query<&Transform, With<Ball>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(transform) = query.get_single_mut() {
        if transform.translation.x < -WINDOW_WIDTH / 2.
            || transform.translation.x > WINDOW_WIDTH / 2.
        {
            next_state.set(GameState::Stop);
        }
    }
}

fn reset_ball(mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>) {
    if let Ok((mut velocity, mut transform)) = query.get_single_mut() {
        transform.translation = Vec3::new(BALL_X, 0., 0.);
        velocity.x = 0.;
        velocity.y = 0.;
    }
}

fn play_game(key_code: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if key_code.pressed(KeyCode::Space) {
        next_state.set(GameState::Play);
    }
}

fn set_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    if let Ok(mut velocity) = query.get_single_mut() {
        let default_velocity = Velocity::default();
        velocity.x = default_velocity.x;
        velocity.y = default_velocity.y;
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
        (velocity + PEDAL_VELOCITY_INCREMENTS).min(MAX_PEDAL_VELOCITY)
    } else if decelerate {
        (velocity - PEDAL_VELOCITY_INCREMENTS).max(-MAX_PEDAL_VELOCITY)
    } else {
        0.
    }
}

fn ball_collision(
    colliders: Query<(&Transform, &Dimensions, Option<&Velocity>), (With<Collider>, Without<Ball>)>,
    mut ball: Query<(&mut Velocity, &Transform, &Dimensions), (With<Ball>, Without<Collider>)>,
) {
    if let Ok((mut ball_velocity, ball_transform, ball_dimensions)) = ball.get_single_mut() {
        for (collider_transform, collider_dimensions, maybe_velocity) in colliders.iter() {
            let collider_velocity = maybe_velocity.unwrap_or(&Velocity { x: 0., y: 0. });
            let relative_velocity_y = ball_velocity.y - collider_velocity.y;
            match collide(
                collider_transform.translation,
                collider_dimensions.0,
                ball_transform.translation,
                ball_dimensions.0,
            ) {
                Some(Collision::Top) | Some(Collision::Bottom) => {
                    ball_velocity.y = -relative_velocity_y;
                }
                Some(Collision::Left) => {
                    ball_velocity.y += collider_velocity.y;
                    ball_velocity.x = -ball_velocity.x + BOUNCE_SPEED_BONUS;
                }
                Some(Collision::Right) => {
                    ball_velocity.y += collider_velocity.y;
                    ball_velocity.x = -ball_velocity.x - BOUNCE_SPEED_BONUS;
                }
                _ => (),
            }
            ball_velocity.x = ball_velocity.x.clamp(-MAX_BALL_SPEED, MAX_BALL_SPEED);
            ball_velocity.y = ball_velocity.y.clamp(-MAX_BALL_SPEED, MAX_BALL_SPEED);
        }
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
struct Collider;

#[derive(Component)]
struct Dimensions(Vec2);

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Velocity {
    fn default() -> Velocity {
        Velocity {
            x: BALL_START_SPEED_X,
            y: BALL_START_SPEED_Y,
        }
    }

    fn zero() -> Velocity {
        Velocity { x: 0., y: 0. }
    }
}

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum GameState {
    #[default]
    Stop,
    Play,
}
