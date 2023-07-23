use bevy::prelude::*;

pub type OnlyCollider = (With<Collider>, Without<Ball>);
pub type OnlyBall = (With<Ball>, Without<Collider>);

#[derive(Component)]
pub struct PedalLeft;

#[derive(Component)]
pub struct PedalRight;

#[derive(Component)]
pub struct Pedal;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub enum Collider {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Component)]
pub struct Dimensions(pub Vec2);

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn zero() -> Velocity {
        Velocity { x: 0., y: 0. }
    }
}
