// Module declarations
mod player;
mod operations;
mod camera;
mod texture;
mod coords;
mod measurer;

// Imports
use std::collections::HashMap;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use num::complex;
use crate::AppState;
use coords::*;
use player::QState;

type c32 = complex::Complex32;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
           .add_event::<operations::SwitchEvent>()
           .add_event::<operations::MixEvent>()
           .add_event::<operations::ClearSelectionEvent>()
           .add_system_set(SystemSet::on_enter(AppState::InGame)
                           .with_system(setup))
            .add_system_set(SystemSet::on_update(AppState::InGame)
                            .with_system(camera::movement)
                            .with_system(texture::set_texture_filters_to_nearest)
                            .with_system(operations::select_positions)
                            .with_system(operations::switcher)
                            .with_system(operations::mixer)
                            .with_system(operations::action_system)
                            .with_system(player::update_superpositions)
                            .with_system(player::update_superposition_indicators)
                            .with_system(operations::clear_selection)
                            .with_system(update_transforms)) //TODO: run in posupdate stage?

            .add_system_set(SystemSet::on_exit(AppState::InGame)
                            .with_system(teardown));
    }
}


#[derive(Component)]
struct MainCamera;


fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut map_query: MapQuery) {
    // Spawn the camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);

    // ====  Create the tile map =========
    // Load texture
    let texture_handle = asset_server.load("sprites/grass_tile.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(8, 8),
            TileSize(64.0, 64.0),
            TextureSize(64.0, 64.0),
        ),
        0u16, // <-- Map ID
        0u16, // <-- Layer ID
    );

    layer_builder.set_all(TileBundle::default());

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, texture_handle);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());

    // ====  Spawn Player ======

    let mut map = HashMap::new();
    map.insert(GridPos::new(0, 0), c32::new(1., 0.));
    player::spawn_player(&mut commands, &asset_server, QState{map});

    // ==== Spawn measure ====

    let mut map = HashMap::new();
    map.insert(GridPos::new(3, 3), c32::new(1./2_f32.sqrt(), 0.));
    map.insert(GridPos::new(5, 3), c32::new(1./2_f32.sqrt(), 0.));
    measurer::spawn_measurement_device(
        &mut commands, &asset_server, QState{map});
}



fn update_transforms(mut superposition_query: Query<(&GridPos, &mut Transform), Changed<GridPos>>) {
    /*
     * Updates anything with a gridpos and a transform
     * whose gridpos component changed
     */
    for (gp, mut transform) in superposition_query.iter_mut() {
        let world_pos = grid_to_world_coordinates(gp);
        *transform = Transform::from_xyz(world_pos.x, world_pos.y,
                                        transform.translation.z);
    }
}


// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
