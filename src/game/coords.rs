use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub struct GridPos{
    x: i32, 
    y: i32
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        GridPos{x, y}
    }
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

pub fn world_to_grid_coordinates(wc: &Vec2) -> GridPos {
    GridPos{x: (wc.x / 64.).floor() as i32,
            y: (wc.y / 64.).floor() as i32}
}
pub fn grid_to_world_coordinates(gc: &GridPos) -> Vec2 {
    Vec2::new(32. + (gc.x * 64) as f32,
              32. + (gc.y * 64) as f32)
}
pub fn are_neighbours(p1: &GridPos, p2: &GridPos) -> bool {
    (p1.x - p2.x).abs() <= 1 && (p1.y - p2.y).abs() <= 1
}
