use std::collections::HashMap;
use bevy::prelude::*;
use num::complex;
use super::coords::*;
type c32 = complex::Complex32;

#[derive(Component)]
pub struct QState {
    pub map: HashMap<GridPos, c32>,
}
#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct Superposition{
    pub factor: c32
}
#[derive(Component)]
pub struct PhaseIndicator;
#[derive(Component)]
pub struct MagnitudeIndicator;

pub fn spawn_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    state: QState,
    ) {

    // Spawn children
    let children: Vec<Entity> = state.map
        .iter()
        .map(|(gp, factor)| spawn_superposition(
            commands, asset_server, *gp, *factor))
        .collect();

    // Spawn player entity
    commands.spawn()
        .insert(state)
        .insert(Player)
        // The transform and global transform are unused in this
        // case but they are needed because child transforms
        // *have* to be relative to their parent transforms,
        // and thus the parents *have* to have a transform.
        // See https://github.com/bevyengine/bevy/issues/2730
        .insert(Transform::identity())
        .insert(GlobalTransform::identity())
        .push_children(&children);
}

pub fn spawn_superposition(commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    gp: GridPos,
    factor: c32
    ) -> Entity {
    /* 
     * Spawns a new superposition at gp
     */
    println!("Spawning: ");

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
    }).id()
}

pub fn update_superpositions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &QState, &Children), (Changed<QState>, With<Player>)>,
    mut superposition_query: Query<(&GridPos, &mut Superposition)>
    ){
    /*
     * Update the factors in the superposition entities
     * whenever the state changes
     */
    
    for (entity, state, children) in player_query.iter() {
        println!("QState: {:?}", state.map);
        // Loop through children, despawn any that aren't in state
        // and make sure the factors match in those that are
        for child in children.iter() {
            let (child_gp, mut child_sp) = 
                superposition_query.get_mut(*child).unwrap();

            println!("{:?}", state.map.get(child_gp));
            if let Some(factor) = state.map.get(child_gp) {
                if child_sp.factor != *factor {
                    child_sp.factor = *factor;
                }
            } else {
                // Superposition is no longer part of the state and should
                // be despawned. I think this removes it from the parent
                // children list aswell.
                println!("Despawning");
                commands.entity(*child).despawn_recursive();
            }
        }
        // Loop through state to see which entries do not have an
        // associated child and spawn one from them
        for (gp, factor) in state.map.iter() {
            let mut found_child = false;

            for child in children.iter() {
                let (child_gp, _child_sp) = 
                    superposition_query.get_mut(*child).unwrap();
                if *child_gp == *gp {
                    found_child = true;
                    break;
                }
            }
            
            if !found_child {
                let id = spawn_superposition(&mut commands,
                                             &asset_server,
                                             *gp,
                                             *factor);
                commands.entity(entity)
                    .add_child(id);
            }
        }
    }
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
