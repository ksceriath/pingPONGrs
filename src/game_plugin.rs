use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::sprite::MaterialMesh2dBundle;
use components::*;
use game_config::Config;
use states::GameState;

pub mod components;
pub mod game_config;
pub mod states;

pub struct TheGame;

impl Plugin for TheGame {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_systems(Startup, Self::spawn_entities)
            .add_systems(OnEnter(GameState::Stop), Self::reset_ball)
            .add_systems(OnEnter(GameState::Play), Self::set_ball_velocity)
            .add_systems(Update, Self::play_game.run_if(in_state(GameState::Stop)))
            .add_systems(
                Update,
                (
                    Self::move_entities,
                    Self::keyboard_event_arrow,
                    Self::keyboard_event_ws,
                    Self::ball_collision,
                    Self::game_over,
                )
                    .run_if(in_state(GameState::Play)),
            );
    }
}

impl TheGame {
    fn spawn_entities(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        config: Res<Config>,
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
            Collider::Left,
            Velocity::zero(),
            Dimensions(Vec2::new(config.pedal_width, config.pedal_length)),
        ));

        commands.spawn((
            pedal_sprite(config.right_pedal_color, config.right_pedal_x),
            PedalRight,
            Pedal,
            Collider::Right,
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
            Collider::Top,
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
            Collider::Bottom,
            Dimensions(Vec2::new(config.border_length, config.border_thickness)),
        ));
    }

    fn move_entities(
        mut query: Query<(&mut Transform, &Velocity, Option<&Pedal>)>,
        config: Res<Config>,
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
        config: Res<Config>,
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
        config: Res<Config>,
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

    fn set_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>, config: Res<Config>) {
        if let Ok(mut velocity) = query.get_single_mut() {
            velocity.x = config.ball_start_speed_x();
            velocity.y = config.ball_start_speed_y();
        }
    }

    fn keyboard_event_arrow(
        key_code: Res<Input<KeyCode>>,
        mut query: Query<&mut Velocity, With<PedalRight>>,
        config: Res<Config>,
    ) {
        if let Ok(mut velocity) = query.get_single_mut() {
            velocity.y = Self::compute_velocity(
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
        config: Res<Config>,
    ) {
        if let Ok(mut velocity) = query.get_single_mut() {
            velocity.y = Self::compute_velocity(
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
        config: Res<Config>,
    ) -> f32 {
        if accelerate {
            (velocity + config.pedal_velocity_increments).min(config.max_pedal_velocity)
        } else if decelerate {
            (velocity - config.pedal_velocity_increments).max(-config.max_pedal_velocity)
        } else {
            0.
        }
    }

    fn ball_collision(
        colliders: Query<(&Transform, &Dimensions, Option<&Velocity>, &Collider), OnlyCollider>,
        mut ball: Query<(&Transform, &Dimensions, &mut Velocity), OnlyBall>,
        config: Res<Config>,
    ) {
        if let Ok((ball_transform, ball_dimensions, mut ball_velocity)) = ball.get_single_mut() {
            for (collider_transform, collider_dimensions, maybe_velocity, collider_pos) in
                colliders.iter()
            {
                let collider_velocity = maybe_velocity.unwrap_or(&Velocity { x: 0., y: 0. });
                let relative_velocity_y = ball_velocity.y - collider_velocity.y;

                if collide(
                    collider_transform.translation,
                    collider_dimensions.0,
                    ball_transform.translation,
                    ball_dimensions.0,
                )
                .is_some()
                {
                    match collider_pos {
                        Collider::Top => {
                            ball_velocity.y = -relative_velocity_y;
                        }
                        Collider::Bottom => {
                            ball_velocity.y = -relative_velocity_y;
                        }
                        Collider::Left => {
                            ball_velocity.y += collider_velocity.y;
                            ball_velocity.x = -ball_velocity.x + config.bounce_speed_bonus;
                        }
                        Collider::Right => {
                            ball_velocity.y += collider_velocity.y;
                            ball_velocity.x = -ball_velocity.x - config.bounce_speed_bonus;
                        }
                    }
                }

                let vx_max = config.max_ball_speed_x;
                let vy_max = config.max_ball_speed_y;
                ball_velocity.x = ball_velocity.x.clamp(-vx_max, vx_max);
                ball_velocity.y = ball_velocity.y.clamp(-vy_max, vy_max);
            }
        }
    }
}
