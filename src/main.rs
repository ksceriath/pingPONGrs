use bevy::prelude::*;
use bevy::window::WindowMode;
use game_plugin::game_config::Config;
use game_plugin::TheGame;

mod game_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .add_systems(PreStartup, Config::init_game_config)
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
