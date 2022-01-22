use rand;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use num::complex;
use super::player::*;
use super::coords::*;
use super::measurer::*;

#[allow(non_camel_case_types)]
type c32 = complex::Complex32;

/*
 * Components
 */
#[derive(Component)]
pub struct Blocking;
#[derive(Component)]
pub struct SelectedPos;

/*
 * Events
 */
pub struct SwitchEvent{
    gp1: GridPos,
    gp2: GridPos,
}
pub struct MixEvent{
    gp1: GridPos,
    gp2: GridPos,
}
pub struct MeasureEvent{
    entity: Entity,
}
pub struct MeasureSuccessEvent{
    pub entity: Entity,
}

pub struct ClearSelectionEvent;

/*
* Systems
*/
pub fn switcher(mut switche_reader: EventReader<SwitchEvent>,
    mut player_state_query: Query<&mut QState, With<Player>>,
    ) {
    // The without superposition is so that bevy knows that the queries 
    // are disjoint since we access the first one mutably

    for switch_event in switche_reader.iter() {
        // Switch the superpositions
        let mut state = player_state_query.single_mut();

        let a_i = *state.map.get(&switch_event.gp1)
            .unwrap_or(&c32::new(0., 0.));
        let b_i = *state.map.get(&switch_event.gp2)
            .unwrap_or(&c32::new(0., 0.));
        let a_f = b_i;
        let b_f = a_i;

        if a_f == c32::new(0., 0.) {
            // Removes value if there, does nothing if not
            state.map.remove(&switch_event.gp1);
        } else {
            // Replaces value if already there and creates new if not
            state.map.insert(switch_event.gp1, a_f);
        }
        if b_f == c32::new(0., 0.) {
            // Removes value if there, does nothing if not
            state.map.remove(&switch_event.gp2);
        } else {
            // Replaces value if already there and creates new if not
            state.map.insert(switch_event.gp2, b_f);
        }
    }
}

pub fn mixer(
    mut mixe_reader: EventReader<MixEvent>,
    mut player_state_query: Query<&mut QState, With<Player>>,
    ) {
    for mix_event in mixe_reader.iter() {

        let mut state = player_state_query.single_mut();

        let a_i = *state.map.get(&mix_event.gp1)
            .unwrap_or(&c32::new(0., 0.));
        let b_i = *state.map.get(&mix_event.gp2)
            .unwrap_or(&c32::new(0., 0.));
        let a_f = (a_i - b_i)/2_f32.sqrt();
        let b_f = (a_i + b_i)/2_f32.sqrt();
        println!("a_f: {}, b_f: {}", a_f, b_f);

        if a_f == c32::new(0., 0.) {
            // Removes value if there, does nothing if not
            state.map.remove(&mix_event.gp1);
        } else {
            // Replaces value if already there and creates new if not
            state.map.insert(mix_event.gp1, a_f);
        }
        if b_f == c32::new(0., 0.) {
            // Removes value if there, does nothing if not
            state.map.remove(&mix_event.gp2);
        } else {
            // Replaces value if already there and creates new if not
            state.map.insert(mix_event.gp2, b_f);
        }
    }
}

pub fn measure(
    mut measurement_event_reader: EventReader<MeasureEvent>,
    mut success_event_writer: EventWriter<MeasureSuccessEvent>,
    measurement_state_query: Query<&QState, With<MeasurementDevice>>,
    mut player_state_query: Query<&mut QState, (With<Player>, Without<MeasurementDevice>)>,
    ) {

    for meas_event in measurement_event_reader.iter() {
        let success_state = measurement_state_query.get(meas_event.entity)
            .unwrap();
        let mut player_state = player_state_query.single_mut();
        let scal_prod = player_state.scal_prod(success_state);
        println!("Prob of success = {}", scal_prod.norm_sqr());
        if rand::random::<f32>() < scal_prod.norm_sqr() {
            *player_state = (*success_state).clone() * scal_prod.conj() / scal_prod.norm();
            success_event_writer.send(MeasureSuccessEvent{ entity: meas_event.entity });
        } else {
            *player_state = ((*player_state).clone()
                - scal_prod.conj() * (*success_state).clone() )
                / (1. - scal_prod.norm_sqr()).sqrt()
        }
    }
}

pub fn select_positions(mut commands: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mouse_button_input: Res<Input<MouseButton>>,
    selected_tiles: Query<(Entity, &GridPos), With<SelectedPos>>,
    tile_query: Query<&TilePos, (With<Tile>, Without<Blocking>)>,
    blocking_query: Query<&GridPos, With<Blocking>>,
    camera_query: Query<(&Transform, &OrthographicProjection)>,
    ) {

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

        let grid_pos = world_to_grid_coordinates(&Vec2::new(world_pos.x, world_pos.y));
        // Check that there is a tile there that is selectable
        // Otherwise the square cannot be selected.
        let mut found_selectable_tile = false;
        for tp in tile_query.iter() {
            if *tp == grid_pos {
                // There is a non-blocking tile on the position clicked
                found_selectable_tile = true;
            }
        }
        for gp in blocking_query.iter() {
            if *gp == grid_pos {
                // There is some blocking element on top of the tile
                found_selectable_tile = false;
            }
        }
        if !found_selectable_tile { return; }

        // If there are previously selected squares,
        // the newly selected square must be a neighbour of one of them
        // and cannot already be selected.
        if !selected_tiles.is_empty() {
            // Make true if tile is a neighbour of some selected tile
            let mut tile_is_neighbour = false;
            for (entity, selected_tile_gridpos) in selected_tiles.iter() {
                // Tile is already selected
                // TODO: Deselect it in a better way, now the player could
                // select 1, 2, 3, and the deselct 2 and the selection would no
                // longer be only neighbours.
                if grid_pos == *selected_tile_gridpos {
                    println!("Despawning");
                    commands.entity(entity).despawn();
                    return;
                }
                if are_neighbours(&grid_pos, selected_tile_gridpos) {
                    tile_is_neighbour = true;
                }
            }
            if !tile_is_neighbour {
                return;
            }
        }
        // You cannot have more than two squares selected at once
        if selected_tiles.iter().count() == 2 {
            return;
        }

        let world_pos_corner = grid_to_world_coordinates(&grid_pos);
        commands.spawn_bundle(SpriteBundle {
                texture: asset_server.load("sprites/select.png"),
                transform: Transform::from_xyz(world_pos_corner.x,
                                               world_pos_corner.y,
                                               20.),
                ..Default::default()
            })
            .insert(SelectedPos)
            .insert(grid_pos);


    }
}
pub fn action_system(keys: Res<Input<KeyCode>>,
    selected_tiles: Query<&GridPos, With<SelectedPos>>,
    measurement_devices: Query<(Entity, &QState), With<MeasurementDevice>>,
    mut switche_writer: EventWriter<SwitchEvent>,
    mut mixe_writer: EventWriter<MixEvent>,
    mut mease_writer: EventWriter<MeasureEvent>,
    mut clear_selection_event_writer: EventWriter<ClearSelectionEvent>
    ) {
    if keys.just_pressed(KeyCode::P) {
        // Check that only two tiles are selected
        if selected_tiles.iter().count() != 2 {
            return;
        }
        let mut it = selected_tiles.iter();
        switche_writer.send(SwitchEvent{ gp1: *it.next().unwrap(), gp2: *it.next().unwrap() });
        clear_selection_event_writer.send(ClearSelectionEvent);
    }
    if keys.just_pressed(KeyCode::O) {
        // Check that only two tiles are selected
        if selected_tiles.iter().count() != 2 {
            return;
        }
        let mut it = selected_tiles.iter();
        mixe_writer.send(MixEvent{ gp1: *it.next().unwrap(), gp2: *it.next().unwrap() });
        clear_selection_event_writer.send(ClearSelectionEvent);
    }
    if keys.just_pressed(KeyCode::I) {
        if let Ok(gp) = selected_tiles.get_single() {
            for (entity, state) in measurement_devices.iter() {
                if state.map.contains_key(gp) {
                    mease_writer.send(MeasureEvent{ entity });
                }
            }
            clear_selection_event_writer.send(ClearSelectionEvent);
        }
    }
}

pub fn clear_selection(mut commands: Commands,
    selected_query: Query<Entity, With<SelectedPos>>,
    mut clear_selection_ev: EventReader<ClearSelectionEvent>
    ) {
    for _ in clear_selection_ev.iter() {
        for entity in selected_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
