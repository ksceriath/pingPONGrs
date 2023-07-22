use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::{WindowMode, WindowResolution};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .add_systems(PreStartup, GameConfig::init_game_config)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(TheGame)
        .run();
}

fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Ping PONG".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[derive(Resource)]
struct GameConfig {
    window_width: f32,

    border_length: f32,
    border_thickness: f32,
    top_border_y: f32,
    bottom_border_y: f32,

    pedal_width: f32,
    pedal_length: f32,
    pedal_bound: f32,
    left_pedal_x: f32,
    right_pedal_x: f32,
    max_pedal_velocity: f32,
    pedal_velocity_increments: f32,

    ball_radius: f32,
    ball_color: Color,
    ball_x: f32,

    ball_start_speed_x: f32,
    ball_start_speed_y: f32,
    bounce_speed_bonus: f32,
    max_ball_speed_x: f32,
    max_ball_speed_y: f32,

    left_pedal_color: Color,
    right_pedal_color: Color,
}

impl GameConfig {
    fn new(resolution: &WindowResolution) -> GameConfig {
        let window_length = resolution.height();
        let window_width = resolution.width();

        let pedal_width = 20.;
        let pedal_length = 150.;
        let pedal_gutter = 20.;

        let pedal_bound = window_length / 2. - pedal_gutter - pedal_length / 2.;
        let left_pedal_x = -(window_width / 2. - pedal_gutter - pedal_width / 2.);
        let right_pedal_x = window_width / 2. - pedal_gutter - pedal_width / 2.;

        GameConfig {
            window_width,

            border_length: window_width - 40.,
            border_thickness: 5.,
            top_border_y: window_length / 2. - 20.,
            bottom_border_y: -(window_length / 2. - 20.),

            pedal_width,
            pedal_length,
            pedal_bound,
            left_pedal_x,
            right_pedal_x,
            max_pedal_velocity: 10.,
            pedal_velocity_increments: 4.,

            ball_radius: 20.,
            ball_color: Color::YELLOW_GREEN,
            ball_x: 0.,

            ball_start_speed_x: -5.,
            ball_start_speed_y: 3.,
            bounce_speed_bonus: 0.75,
            max_ball_speed_x: 15.,
            max_ball_speed_y: 10.,

            left_pedal_color: Color::CYAN,
            right_pedal_color: Color::BISQUE,
        }
    }

    fn init_game_config(mut commands: Commands, query: Query<&Window>) {
        commands.insert_resource(GameConfig::new(&query.single().resolution));
    }
}

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
    config: Res<GameConfig>,
) {
    commands.spawn(Camera2dBundle::default());

    let pedal_sprite = |color: Color, x: f32| SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(config.pedal_width, config.pedal_length)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(x, 0., 0.),
            ..default()
        },
        ..default()
    };

    commands.spawn((
        pedal_sprite(config.left_pedal_color, config.left_pedal_x),
        PedalLeft,
        Pedal,
        Collider,
        Velocity::zero(),
        Dimensions(Vec2::new(config.pedal_width, config.pedal_length)),
    ));

    commands.spawn((
        pedal_sprite(config.right_pedal_color, config.right_pedal_x),
        PedalRight,
        Pedal,
        Collider,
        Velocity::zero(),
        Dimensions(Vec2::new(config.pedal_width, config.pedal_length)),
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(config.ball_radius).into())
                .into(),
            material: materials.add(ColorMaterial::from(config.ball_color)),
            transform: Transform::from_translation(Vec3::new(config.ball_x, 0., 0.)),
            ..default()
        },
        Ball,
        Velocity::zero(),
        Dimensions(Vec2::new(config.ball_radius, config.ball_radius)),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(config.border_length, config.border_thickness)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., config.top_border_y, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        Dimensions(Vec2::new(config.border_length, config.border_thickness)),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(config.border_length, config.border_thickness)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., config.bottom_border_y, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        Dimensions(Vec2::new(config.border_length, config.border_thickness)),
    ));
}

fn move_entities(
    mut query: Query<(&mut Transform, &Velocity, Option<&Pedal>)>,
    config: Res<GameConfig>,
) {
    for (mut transform, velocity, maybe_pedal) in query.iter_mut() {
        let displacement = Vec3::new(velocity.x, velocity.y, 0.);
        transform.translation += displacement;

        if maybe_pedal.is_some() {
            transform.translation.y = transform
                .translation
                .y
                .clamp(-config.pedal_bound, config.pedal_bound);
        }
    }
}

fn game_over(
    mut query: Query<&Transform, With<Ball>>,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
) {
    if let Ok(transform) = query.get_single_mut() {
        if transform.translation.x < -config.window_width / 2.
            || transform.translation.x > config.window_width / 2.
        {
            next_state.set(GameState::Stop);
        }
    }
}

fn reset_ball(
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    config: Res<GameConfig>,
) {
    if let Ok((mut velocity, mut transform)) = query.get_single_mut() {
        transform.translation = Vec3::new(config.ball_x, 0., 0.);
        velocity.x = 0.;
        velocity.y = 0.;
    }
}

fn play_game(key_code: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if key_code.pressed(KeyCode::Space) {
        next_state.set(GameState::Play);
    }
}

fn set_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>, config: Res<GameConfig>) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.x = config.ball_start_speed_x;
        velocity.y = config.ball_start_speed_y;
    }
}

fn keyboard_event_arrow(
    key_code: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<PedalRight>>,
    config: Res<GameConfig>,
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.y = compute_velocity(
            velocity.y,
            key_code.pressed(KeyCode::Up),
            key_code.pressed(KeyCode::Down),
            config,
        );
    }
}

fn keyboard_event_ws(
    key_code: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<PedalLeft>>,
    config: Res<GameConfig>,
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.y = compute_velocity(
            velocity.y,
            key_code.pressed(KeyCode::W),
            key_code.pressed(KeyCode::S),
            config,
        );
    }
}

fn compute_velocity(
    velocity: f32,
    accelerate: bool,
    decelerate: bool,
    config: Res<GameConfig>,
) -> f32 {
    if accelerate {
        (velocity + config.pedal_velocity_increments).min(config.max_pedal_velocity)
    } else if decelerate {
        (velocity - config.pedal_velocity_increments).max(-config.max_pedal_velocity)
    } else {
        0.
    }
}

type OnlyCollider = (With<Collider>, Without<Ball>);
type OnlyBall = (With<Ball>, Without<Collider>);
fn ball_collision(
    colliders: Query<(&Transform, &Dimensions, Option<&Velocity>), OnlyCollider>,
    mut ball: Query<(&Transform, &Dimensions, &mut Velocity), OnlyBall>,
    config: Res<GameConfig>,
) {
    if let Ok((ball_transform, ball_dimensions, mut ball_velocity)) = ball.get_single_mut() {
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
                    ball_velocity.x = -ball_velocity.x + config.bounce_speed_bonus;
                }
                Some(Collision::Right) => {
                    ball_velocity.y += collider_velocity.y;
                    ball_velocity.x = -ball_velocity.x - config.bounce_speed_bonus;
                }
                _ => (),
            }
            let vx_max = config.max_ball_speed_x;
            let vy_max = config.max_ball_speed_y;
            ball_velocity.x = ball_velocity.x.clamp(-vx_max, vx_max);
            ball_velocity.y = ball_velocity.y.clamp(-vy_max, vy_max);
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
