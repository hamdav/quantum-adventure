use bevy::prelude::*;
use bevy::app::AppExit;

use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::MainMenu)
                           .with_system(setup))
            .add_system_set(SystemSet::on_update(AppState::MainMenu)
                            .with_system(button_system))
            .add_system_set(SystemSet::on_exit(AppState::MainMenu)
                            .with_system(teardown));
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
enum ButtonAction {
    Play,
    Quit,
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &mut Style, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut exit: EventWriter<AppExit>,
    mut state: ResMut<State<AppState>>
) {
    for (interaction, mut color, mut style, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                match *action {
                    ButtonAction::Quit => exit.send(AppExit),
                    ButtonAction::Play => state.set(AppState::InGame).unwrap(),
                };
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                style.size = Size::new(Val::Px(160.0), Val::Px(75.0));
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                style.size = Size::new(Val::Px(150.0), Val::Px(65.0));
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/Evolventa.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ButtonAction::Play);

    // Quit button
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Quit",
                    TextStyle {
                        font: asset_server.load("fonts/Evolventa.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ButtonAction::Quit);
}

// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
