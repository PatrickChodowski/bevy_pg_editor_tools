use bevy::color::palettes::tailwind::GRAY_500;
use bevy::prelude::*;

use crate::brushes::{NothingBrush, BrushType};

#[derive(Resource, Clone)]
pub struct EditorSettings {
    pub color: Color,
    pub snap_nav: bool,
    // pub multi_ghost: bool,
    pub show_spawners: bool,
    pub show_markers: bool,
    pub mode: EditorMode,
    pub brush_mapping: fn(commands: &mut Commands, brush_id: usize, editor_settings: &ResMut<EditorSettings>) -> Box<dyn BrushType>,
    pub brush_id_labels: Vec<(usize, &'static str)>,
    pub brush_id: usize,
    pub brush_radius: f32,
    pub brush_typ: Box<dyn BrushType>,
    pub plane_width: f32,
    pub plane_height: f32,
    pub plane_subdivisions: u32,
    pub plane_save_chunks: u32,
    pub plane_loc:  Vec3
}
impl EditorSettings {
    pub fn new(
        brush_mapping: fn(commands: &mut Commands, brush_id: usize, editor_settings: &ResMut<EditorSettings>) -> Box<dyn BrushType>,
        brush_id_labels: Vec<(usize, &'static str)>
    ) -> Self {
        Self {
            color: Color::from(GRAY_500),
            snap_nav: true,
            // multi_ghost: false,
            show_spawners: false,
            show_markers: false,
            mode: EditorMode::Scene,
            brush_mapping,
            brush_id_labels,
            brush_id: 0,
            brush_radius: 10.0,
            brush_typ: Box::new(NothingBrush),
            plane_height: 50.0,
            plane_width: 50.0,
            plane_subdivisions: 1,
            plane_save_chunks: 1,
            plane_loc: Vec3::ZERO
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Scene,
    Brushes,
    Plane
}