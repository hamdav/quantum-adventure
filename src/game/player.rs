use bevy::prelude::*;
use num::complex;
use super::coords::*;
type c32 = complex::Complex32;

#[derive(Component)]
pub struct Superposition{
    pub factor: c32
}
#[derive(Component)]
pub struct PhaseIndicator;
#[derive(Component)]
pub struct MagnitudeIndicator;

pub fn spawn_superposition(commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    gp: GridPos,
    factor: c32
    ) {
    /* 
     * Spawns a new superposition at gp
     */

    // Position in world coordinates
    let world_pos = grid_to_world_coordinates(&gp);
    // Barlength
    let bar_length = (factor.norm() * 46.).ceil();

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("sprites/player_front.png"),
        transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.),
        ..Default::default()
    })
    .insert(Superposition{ factor })
    .insert(gp)
    .with_children(|parent| {
        // Spawn bar background
        parent.spawn_bundle(SpriteBundle{
            texture: asset_server.load("sprites/bar.png"),
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        });
        // Spawn bar
        parent.spawn_bundle(SpriteBundle{
                sprite: Sprite {
                    color: Color::rgb(0.7, 0.0, 0.0),
                    custom_size: Some(Vec2::new(bar_length, 4.)),
                    ..Default::default()
                },
                // -32 because it starts from the middle of the tile
                // + 23/2 because the anchor is in the middle of the bar
                // + 9 because the bar should be 9 pixels left of the boundry
                transform: Transform::from_xyz(bar_length/2. - 32. + 9., 
                                               4./2. - 32. + 5., 2.),
                ..Default::default()
        })
        .insert(MagnitudeIndicator);
        // Spawn arrow
        parent.spawn_bundle(SpriteBundle{
                texture: asset_server.load("sprites/arrow.png"),
                transform: Transform::from_xyz(0., 18., 2.)
                    .with_rotation(Quat::from_rotation_z(factor.arg())),
                ..Default::default()
        })
        .insert(PhaseIndicator);
    });
}

pub fn update_superposition_indicators(
    superposition_query: Query<(&Children, &Superposition),
        Changed<Superposition>>,
    mut phase_ind_q: Query<&mut Transform,
        (With<PhaseIndicator>, Without<MagnitudeIndicator>)>,
    mut magn_ind_q: Query<(&mut Transform, &mut Sprite),
        With<MagnitudeIndicator>>,
    ) {
    /*
     * Update the phase and magnitude indicators on superpositions
     */

    for (children, sp) in superposition_query.iter() {
        for &child in children.iter() {
            if let Ok(mut arrow_transform) = phase_ind_q.get_mut(child) {
                *arrow_transform = Transform{
                    translation: arrow_transform.translation,
                    rotation: Quat::from_rotation_z(sp.factor.arg()),
                    scale: arrow_transform.scale
                };
            }
            // Barlength
            let bar_length = (sp.factor.norm() * 46.).ceil();
            if let Ok((mut bar_transform, mut bar_sprite)) = magn_ind_q.get_mut(child) {
                *bar_transform = Transform::from_xyz(bar_length/2. - 32. + 9., 
                    4./2. - 32. + 5., 2.);
                bar_sprite.custom_size = Some(Vec2::new(bar_length, 4.));
            }
        }
    }
}
