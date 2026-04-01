use bevy::prelude::*;
use bevy_pg_core::prelude::{GameStatePlay, MainCamera};
// use bevy::window::PrimaryWindow;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;

use crate::planes::PlaneToEdit;

pub struct PGEditorPointer;

/* Editor needs something not dependent on navmesh. For Brushes and spawning assets */

impl Plugin for PGEditorPointer {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EditorPointer::default())
            .add_systems(PreUpdate, update_pointer.run_if(in_state(GameStatePlay::Editor)))
        ;
    }
}


#[derive(Resource)]
pub struct EditorPointer {
    pub loc: Option<Vec3>,
    pub plane_entity: Option<Entity>
}
impl Default for EditorPointer {
    fn default() -> Self {
        EditorPointer {
            loc: None,
            plane_entity: None
        }
    }
}

impl EditorPointer {
    fn reset(&mut self) {
        self.loc = None;
        self.plane_entity = None;
    }
}



fn update_pointer(
    mut editor_pointer: ResMut<EditorPointer>,
    hovermap:           Res<HoverMap>,
    // primary:            Single<&Window, With<PrimaryWindow>>,
    camera_entity:      Single<Entity, With<MainCamera>>,
    planes:             Query<(Entity, &Transform), With<PlaneToEdit>>,
    nodes:              Query<Entity, With<Node>>,
){
    editor_pointer.reset();
    let mouse_hit_data = hovermap.0.get(&PointerId::Mouse).unwrap();
    for (entity, hit_data) in mouse_hit_data.iter(){
        if hit_data.camera != *camera_entity {
            continue;
        }
        let Some(hit_position) = hit_data.position else {continue};
        if let Ok((plane_entity, plane_transform)) = planes.get(*entity){
            editor_pointer.plane_entity = Some(plane_entity);
            editor_pointer.loc = Some(plane_transform.translation + hit_position);
        }
        if let Ok(_node_entity) = nodes.get(*entity){
            editor_pointer.reset();
            return;
        }
    }
    // if let Some(editor_pointer_loc) = editor_pointer.loc {
    //     info!("Editor pointer loc: {}", editor_pointer_loc);
    // }
}

