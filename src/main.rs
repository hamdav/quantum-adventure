use bevy::prelude::*;
use bevy::window::WindowMode;

mod menu;
mod game;
mod helpers;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Quantum Adventure".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(AppState::InGame)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .run();
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum AppState {
    MainMenu,
    InGame,
    Paused,
}
