
use bevy::prelude::*;

use crate::prelude::WorldPos;

pub struct PGEditorToolsPlugin;

impl Plugin for PGEditorToolsPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(WorldPos::new())
        ;
    }
}

pub mod box_select;
pub mod brushes;
pub mod tracker;
pub mod world_pos;


pub mod prelude {
    pub use crate::tracker::{PGEditorTrackerPlugin, Undo, Redo, UndoMessage, RedoMessage, Changes, Change, ChangesSet};
    pub use crate::box_select::{BoxSelectController, box_select_controller, box_select_changed, BoxSelectFinal, BoxSelect, PGEditorBoxSelectPlugin};
    pub use crate::brushes::{BrushSelectController, brush_select_controller, brush_changed, BrushDone, BrushStart, Brush, PGEditorBrushSelectPlugin, BrushType, BrushSettings};
    pub use crate::world_pos::WorldPos;
    pub use crate::PGEditorToolsPlugin;
}