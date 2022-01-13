use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::AppState;

use super::helpers;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
           .add_system_set(SystemSet::on_enter(AppState::InGame)
                           .with_system(setup))
            .add_system_set(SystemSet::on_update(AppState::InGame)
                            .with_system(helpers::camera::movement)
                            .with_system(helpers::texture::set_texture_filters_to_nearest)
                            .with_system(mark_tile_and_player))
            .add_system_set(SystemSet::on_exit(AppState::InGame)
                            .with_system(teardown));
    }
}

#[derive(Component, PartialEq, Eq)]
struct GridPos{
    x: i32, 
    y: i32
}

impl PartialEq<TilePos> for GridPos {
    fn eq(&self, other: &TilePos) -> bool {
        if self.x >= 0 && self.y >= 0 && 
                self.x as u32 == other.0 &&
                self.y as u32 == other.1 {
            true
        } else {
            false
        }
    }
}
impl PartialEq<GridPos> for TilePos {
    fn eq(&self, other: &GridPos) -> bool {
        *other == *self
    }
}

#[derive(Component)]
struct Superposition{
    phase: f32,
    magnitude: f32,
}
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct Blocking;



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
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("sprites/player_front.png"),
        transform: Transform::from_xyz(32., 32., 1.),
        ..Default::default()
    })
    .insert(Superposition{ phase: 0., magnitude: 1. })
    .insert(GridPos{ x: 0, y: 0 });
}

fn mark_tile_and_player(mut commands: Commands,
        windows: Res<Windows>,
        asset_server: Res<AssetServer>,
        mouse_button_input: Res<Input<MouseButton>>,
        superposition_query: Query<&GridPos, With<Superposition>>,
        tile_query: Query<&TilePos, (With<Tile>, Without<Blocking>)>,
        camera_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>) {

    if mouse_button_input.just_released(MouseButton::Left) {
    
        // get the primary window
        let wnd = windows.get_primary().unwrap();

        // check if the cursor is in the primary window
        let world_pos = if let Some(screen_pos) = wnd.cursor_position() {

            // get the size of the window
            let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // assuming there is exactly one main camera entity, so this is OK
            let (camera_transform, ortho_proj) = camera_query.single();

            // Screen coordinates are from bottom left position of screen
            // But the default orthographic projection camera starts with (0,0)
            // at the center of the screen. So subtract half of the screen size
            let mut p = screen_pos - size / 2.0;

            // The camera may also have a different scale meaning that 
            // one screen pixel is larger / smaller than one world pixel
            // as seen by the camera.
            p *= ortho_proj.scale;

            // And finaly, the camera may have shifted away from pointing at (0,0)
            // So apply the same transformation that the camera has done to the
            // computed value
            camera_transform.compute_matrix() * p.extend(0.0).extend(1.0)
        } else {return;};

        println!("Click at: {:?}", world_pos);

        let grid_pos = world_to_grid_coordinates(&Vec2::new(world_pos.x, world_pos.y));
        let mut found = false;
        for tp in tile_query.iter() {
            if *tp == grid_pos {
                // There is a non-blocking tile on the position clicked
                found = true;
            }
        }
        if !found { return; }

        let world_pos_corner = grid_to_world_coordinates(&grid_pos);
        commands.spawn_bundle(SpriteBundle {
            texture: asset_server.load("sprites/select.png"),
            transform: Transform::from_xyz(world_pos_corner.x,
                                           world_pos_corner.y,
                                           1.),
            ..Default::default()
        });

        for sp_grid_pos in superposition_query.iter() {
            // If the clicked grid_pos is the position of the superposition
            // spawn a marker on that tile
            if *sp_grid_pos == grid_pos {
                println!("MATCH");
            }
        }
    }
}

fn world_to_grid_coordinates(wc: &Vec2) -> GridPos {
    GridPos{x: (wc.x / 64.).floor() as i32,
            y: (wc.y / 64.).floor() as i32}
}
fn grid_to_world_coordinates(gc: &GridPos) -> Vec2 {
    Vec2::new(32. + (gc.x * 64) as f32,
              32. + (gc.y * 64) as f32)
}

// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
