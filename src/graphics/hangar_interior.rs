//! Dimensions and platform palette only. Architecture is the Blender-authored GLB.
//! No procedural room fallback: a missing interior must be reported when loading.
use crate::gameplay::station::Station;
use raylib::prelude::*;

pub struct InteriorStyle {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub exit_width: f32,
    pub exit_height: f32,
    pub ring_radius: f32,
    pub graphite: Color,
    pub titanium: Color,
    pub silver: Color,
    pub cyan: Color,
    pub ice: Color,
    pub window: Color,
}
impl Default for InteriorStyle {
    fn default() -> Self {
        Self {
            width: 36.0,
            height: 16.0,
            depth: 56.0,
            exit_width: 20.0,
            exit_height: 12.0,
            ring_radius: 168.0,
            graphite: Color::new(13, 20, 30, 255),
            titanium: Color::new(45, 57, 76, 255),
            silver: Color::new(78, 94, 116, 255),
            cyan: Color::new(49, 139, 180, 255),
            ice: Color::new(155, 202, 222, 255),
            window: Color::new(190, 153, 98, 255),
        }
    }
}
impl InteriorStyle {
    pub fn configure(&mut self, station: &Station) {
        let dock = &station.dock;
        self.ring_radius = (dock.local_position[1].powi(2) + dock.local_position[2].powi(2)).sqrt()
            * station.length;
        self.width = dock.room_width;
        self.height = dock.room_height;
        self.depth = dock.room_depth;
        self.exit_width = dock.width;
        self.exit_height = dock.height;
    }
}
