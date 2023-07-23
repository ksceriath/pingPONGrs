use bevy::prelude::*;
use bevy::window::WindowResolution;
use rand::Rng;

#[derive(Resource)]
pub struct Config {
    pub window_width: f32,

    pub border_length: f32,
    pub border_thickness: f32,
    pub top_border_y: f32,
    pub bottom_border_y: f32,

    pub pedal_width: f32,
    pub pedal_length: f32,
    pub pedal_bound: f32,
    pub left_pedal_x: f32,
    pub right_pedal_x: f32,
    pub max_pedal_velocity: f32,
    pub pedal_velocity_increments: f32,

    pub ball_radius: f32,
    pub ball_color: Color,
    pub ball_x: f32,

    pub bounce_speed_bonus: f32,
    pub max_ball_speed_x: f32,
    pub max_ball_speed_y: f32,

    pub left_pedal_color: Color,
    pub right_pedal_color: Color,
}

impl Config {
    fn new(resolution: &WindowResolution) -> Config {
        let window_length = resolution.height();
        let window_width = resolution.width();

        let pedal_width = 20.;
        let pedal_length = 150.;
        let pedal_gutter = 20.;

        let pedal_bound = window_length / 2. - pedal_gutter - pedal_length / 2.;
        let left_pedal_x = -(window_width / 2. - pedal_gutter - pedal_width / 2.);
        let right_pedal_x = window_width / 2. - pedal_gutter - pedal_width / 2.;

        Config {
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

            bounce_speed_bonus: 0.75,
            max_ball_speed_x: 15.,
            max_ball_speed_y: 10.,

            left_pedal_color: Color::CYAN,
            right_pedal_color: Color::BISQUE,
        }
    }

    pub fn ball_start_speed_x(&self) -> f32 {
        rand::thread_rng().gen_range(-5. ..-3.)
    }

    pub fn ball_start_speed_y(&self) -> f32 {
        rand::thread_rng().gen_range(-7. ..7.)
    }

    pub fn init_game_config(mut commands: Commands, query: Query<&Window>) {
        commands.insert_resource(Config::new(&query.single().resolution));
    }
}
