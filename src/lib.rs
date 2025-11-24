use bevy::prelude::*;
use bevy_pg_scenes::prelude::{Spawner, Marker};

pub mod assets_panel;
pub mod box_select;
pub mod brushes;
pub mod controller;
pub mod ghost;
pub mod tracker;
pub mod thumbnails;
pub mod ui_controls;
pub mod noises;
pub mod planes;
pub mod vertex;
pub mod terrain_brushes;


use assets_panel::PGEditorAssetsPanelPlugin;
use brushes::PGEditorBrushSelectPlugin;
use box_select::PGEditorBoxSelectPlugin;
use controller::PGEditorControllerPlugin;
use ghost::PGEditorGhostPlugin;
use thumbnails::PGEditorThumbnailsPlugin;
use tracker::PGEditorTrackerPlugin;
use ui_controls::PGEditorControlsDisplayPlugin;
use vertex::PGEditorVertexPlugin;


pub struct PGEditorPlugin{
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, option: Option<String>) -> Spawner,
    pub vertex_radius: f32
}

impl Plugin for PGEditorPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(
            (
                PGEditorTrackerPlugin,
                PGEditorBrushSelectPlugin,
                PGEditorBoxSelectPlugin,
                PGEditorAssetsPanelPlugin,
                PGEditorThumbnailsPlugin,
                PGEditorControllerPlugin,
                PGEditorControlsDisplayPlugin,
                PGEditorVertexPlugin{
                    vertex_radius: self.vertex_radius
                },
                PGEditorGhostPlugin{
                    spawner_mesh: self.spawner_mesh,
                    marker_mesh: self.marker_mesh,
                    markers_mapping: self.markers_mapping,
                    spawners_mapping: self.spawners_mapping
                }
            )
        )
        ;
    }
}



pub mod prelude {
    pub use crate::assets_panel::PGEditorAssetsPanelPlugin;
    pub use crate::box_select::{BoxSelectController, box_select_controller, box_select_changed, BoxSelectFinal, BoxSelect, PGEditorBoxSelectPlugin};
    pub use crate::brushes::{BrushSelectController, brush_select_controller, brush_changed, BrushDone, BrushStart, Brush, PGEditorBrushSelectPlugin, BrushType, BrushSettings};
    pub use crate::controller::PGEditorControllerPlugin;
    pub use crate::ghost::PGEditorGhostPlugin;
    pub use crate::thumbnails::PGEditorThumbnailsPlugin;
    pub use crate::tracker::{PGEditorTrackerPlugin, Undo, Redo, UndoMessage, RedoMessage, Changes, Change, ChangesSet};
    pub use crate::ui_controls::PGEditorControlsDisplayPlugin;
    pub use crate::planes::{PlaneToEdit, plane_mesh};
    pub use crate::vertex::{SpawnVertices, SelectedVertex, PlaneVertex, PGEditorVertexPlugin, TerrainVertexController, VertexRefs, terrain_vertex_controller};
    pub use crate::terrain_brushes::{TerrainHeightBrush, TerrainColorBrush, HeightBrushType, ColorBrushType};
    pub use crate::noises::{NoiseType, Noise};

    pub use crate::PGEditorPlugin;
}