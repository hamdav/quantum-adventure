use bevy::prelude::*;

use super::coords::*;
use super::operations::{Blocking, MeasureSuccessEvent};
use num::complex;
#[allow(non_camel_case_types)]
type c32 = complex::Complex32;

#[derive(Component)]
pub struct OpenableByMeasurement{
    measurement_device_entity: Entity
}

#[derive(Component)]
pub struct AnimateOnce;



pub fn spawn_door(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    gp: GridPos,
    measurement_device_entity: Entity,
    ) {
    /*
     * Spawns a door that opens when successful measurement
     * by the given entity is done.
     */
    let world_pos = grid_to_world_coordinates(&gp);

    let texture_handle = asset_server.load("sprites/door_anim.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 10, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn_bundle(SpriteSheetBundle{
        texture_atlas: texture_atlas_handle,
        sprite: TextureAtlasSprite::new(0),
        transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.),
        ..Default::default()
    })
    .insert(OpenableByMeasurement{ measurement_device_entity })
    .insert(Blocking)
    .insert(AnimateOnce)
    .insert(gp);
}

pub fn door_opening_system(
    mut commands: Commands,
    mut measurement_success_reader: EventReader<MeasureSuccessEvent>,
    door_query: Query<(Entity, &OpenableByMeasurement)>,
    ) {
    /*
     * Opens all doors opened by a measurement success event
     */
    for event in measurement_success_reader.iter() {
        for (door_entity, openable) in door_query.iter() {
            if openable.measurement_device_entity == event.entity {
                commands.entity(door_entity)
                    .remove::<Blocking>()
                    .insert(Timer::from_seconds(0.1, true));
            }
        }
    }
}

pub fn sprite_animation(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(Entity, &mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>),
        With<AnimateOnce>>,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            if sprite.index + 1 == texture_atlas.textures.len() {
                commands.entity(entity)
                    .remove::<Timer>();
            } else {
                sprite.index = sprite.index + 1;
            }
        }
    }
}
