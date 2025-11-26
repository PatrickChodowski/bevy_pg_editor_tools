use bevy::prelude::*;
use crate::ghost::{GhostTransformAxis, GhostTransformMode};

#[derive(Resource, Debug)]
pub struct EditorSettings {
    pub mode: GhostTransformMode,
    pub axis: GhostTransformAxis,
    pub change_value_scale: f32,
    pub snap_nav: bool,
    pub multi_ghost: bool,
    pub show_spawners_markers: bool
}
impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            mode: GhostTransformMode::default(),
            axis: GhostTransformAxis::default(),
            change_value_scale: 1.0,
            snap_nav: true,
            multi_ghost: false,
            show_spawners_markers: false
        }
    }
}