use bevy::prelude::*;

use super::player::*;
use super::coords::*;
use num::complex;
type c32 = complex::Complex32;

#[derive(Component)]
pub struct MeasurementDevice;

pub fn spawn_measurement_device(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    state: QState,
    ) {
    /*
     * Spawn a new measurement device
     */

    // Spawn children
    let children: Vec<Entity> = state.map
        .iter()
        .map(|(gp, factor)| spawn_measurement_indicator(
            commands, asset_server, *gp, *factor))
        .collect();

    // Spawn measurement device entity
    commands.spawn()
        .insert(state)
        .insert(MeasurementDevice)
        // The transform and global transform are unused in this
        // case but they are needed because child transforms
        // *have* to be relative to their parent transforms,
        // and thus the parents *have* to have a transform.
        // See https://github.com/bevyengine/bevy/issues/2730
        .insert(Transform::identity())
        .insert(GlobalTransform::identity())
        .push_children(&children);
}
pub fn spawn_measurement_indicator(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    gp: GridPos,
    factor: c32
    ) -> Entity {

    // bar = 5 x 24
    // Position in world coordinates
    let world_pos = grid_to_world_coordinates(&gp);
    // Barlength
    let bar_length = (factor.norm() * 24.).ceil();

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("sprites/measuring_device.png"),
        transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.),
        ..Default::default()
    })
    .insert(Superposition{ factor })
    .insert(gp)
    .with_children(|parent| {
        // Spawn bar
        parent.spawn_bundle(SpriteBundle{
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.87, 0.08),
                    custom_size: Some(Vec2::new(bar_length, 5.)),
                    ..Default::default()
                },
                // -32 because it starts from the middle of the tile
                // + bar_length/2 because the anchor is in the middle of the bar
                // + 10 because the bar should be 9 pixels left of the boundry
                transform: Transform::from_xyz(bar_length/2. - 32. + 10., 
                                               -5./2. + 32. - 21., 1.),
                ..Default::default()
        });
        //.insert(MagnitudeIndicator);
        // Spawn arrow
        parent.spawn_bundle(SpriteBundle{
                texture: asset_server.load("sprites/green_arrow.png"),
                transform: Transform::from_xyz(-14.+3.5, 18., 2.)
                    .with_rotation(Quat::from_rotation_z(factor.arg())),
                ..Default::default()
        });
        //.insert(PhaseIndicator);
    }).id()
}
