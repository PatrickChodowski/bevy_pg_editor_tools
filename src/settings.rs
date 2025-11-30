use bevy::color::palettes::tailwind::GRAY_500;
use bevy::prelude::*;
use bevy_pg_scenes::prelude::TerrainChunk;

use crate::brushes::{NothingBrush, BrushType};
use crate::ghost::{GhostTransformAxis, GhostTransformMode};
use crate::planes::PlaneToEdit;

#[derive(Resource)]
pub struct EditorSettings {
    pub ghost_transform_mode: GhostTransformMode,
    pub ghost_transform_axis: GhostTransformAxis,
    pub change_value_scale: f32,
    pub color: Color,
    pub snap_nav: bool,
    pub multi_ghost: bool,
    pub show_spawners: bool,
    pub show_markers: bool,
    pub mode: EditorMode,
    pub brush_mapping: fn(commands: &mut Commands, terrain_chunks: &Query<Entity, (With<TerrainChunk>, With<PlaneToEdit>)>, brush_id: usize) -> Box<dyn BrushType>,
    pub brush_id_labels: Vec<(usize, &'static str)>,
    pub brush_id: usize,
    pub brush_radius: f32,
    pub brush_typ: Box<dyn BrushType>
}
impl EditorSettings {
    pub fn new(
        brush_mapping: fn(commands: &mut Commands, terrain_chunks: &Query<Entity, (With<TerrainChunk>, With<PlaneToEdit>)>, brush_id: usize) -> Box<dyn BrushType>,
        brush_id_labels: Vec<(usize, &'static str)>
    ) -> Self {
        Self {
            ghost_transform_mode: GhostTransformMode::default(),
            ghost_transform_axis: GhostTransformAxis::default(),
            change_value_scale: 1.0,
            color: Color::from(GRAY_500),
            snap_nav: true,
            multi_ghost: false,
            show_spawners: false,
            show_markers: false,
            mode: EditorMode::Scene,
            brush_mapping,
            brush_id_labels,
            brush_id: 0,
            brush_radius: 10.0,
            brush_typ: Box::new(NothingBrush)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Scene,
    Brushes,
    Plane
}