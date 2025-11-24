use bevy::prelude::*;

pub mod box_select;
pub mod brushes;
pub mod tracker;

use tracker::PGEditorTrackerPlugin;
use brushes::PGEditorBrushSelectPlugin;
use box_select::PGEditorBoxSelectPlugin;

pub struct PGEditorToolsPlugin;

impl Plugin for PGEditorToolsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(
            (
                PGEditorTrackerPlugin,
                PGEditorBrushSelectPlugin,
                PGEditorBoxSelectPlugin  
            )
        );
    }
}



pub mod prelude {
    pub use crate::tracker::{PGEditorTrackerPlugin, Undo, Redo, UndoMessage, RedoMessage, Changes, Change, ChangesSet};
    pub use crate::box_select::{BoxSelectController, box_select_controller, box_select_changed, BoxSelectFinal, BoxSelect, PGEditorBoxSelectPlugin};
    pub use crate::brushes::{BrushSelectController, brush_select_controller, brush_changed, BrushDone, BrushStart, Brush, PGEditorBrushSelectPlugin, BrushType, BrushSettings};
    pub use crate::PGEditorToolsPlugin;
}