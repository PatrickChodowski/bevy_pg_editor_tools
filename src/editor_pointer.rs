use bevy::prelude::*;
use bevy_pg_core::prelude::{GameStatePlay, MainCamera};
use bevy::window::PrimaryWindow;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy_pg_nav::prelude::TerrainRayMeshData;

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
    pub plane_loc: Option<Vec3>,
    pub center_screen_plane_pos: Option<Vec3>, 
    pub plane_entity: Option<Entity>,
    pub y0_pos: Option<Vec3> 
}
impl Default for EditorPointer {
    fn default() -> Self {
        EditorPointer {
            plane_loc: None,
            center_screen_plane_pos: None,
            plane_entity: None,
            y0_pos: None,
        }
    }
}

impl EditorPointer {
    fn reset(&mut self) {
        self.y0_pos = None;
        self.plane_loc = None;
        self.plane_entity = None;
        self.center_screen_plane_pos = None;
    }
    fn reset_click(&mut self) {
        self.plane_loc = None;
        self.plane_entity = None;
    }

}



fn update_pointer(
    mut editor_pointer: ResMut<EditorPointer>,
    hovermap:           Res<HoverMap>,
    main_camera:        Single<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
    primary:            Single<&Window, With<PrimaryWindow>>,
    planes:             Query<(&TerrainRayMeshData, &PlaneToEdit)>,
    nodes:              Query<Entity, With<Node>>,
){
    editor_pointer.reset();
    let mouse_hit_data = hovermap.0.get(&PointerId::Mouse).unwrap();
    let (camera_entity, camera, camera_transform) = main_camera.into_inner();
    let center_position = primary.size()*0.5;

    if let Some(cursor_position) = primary.cursor_position(){
        if let Ok(cursor_ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if cursor_ray.direction.y.abs() > f32::EPSILON {
                let distance = -cursor_ray.origin.y / cursor_ray.direction.y;
                if distance > 0.0 {
                    let intersection_point = cursor_ray.get_point(distance);
                    let target_x = intersection_point.x;
                    let target_z = intersection_point.z;
                    editor_pointer.y0_pos = Some(Vec3::new(target_x, 0.0, target_z));
                }
            }
        }
    }
    
    if let Ok(center_ray) = camera.viewport_to_world(camera_transform, center_position) {
        for (trmd, _plane) in planes.iter(){
            if let Some(intersection) = trmd.ray_intersection(&center_ray.origin, &center_ray.direction){
                editor_pointer.center_screen_plane_pos = Some(intersection.position);
                // info!("screen center plane pos: {:?}", editor_pointer.center_screen_plane_pos);
                break;
            }
        }
    }

    for (entity, hit_data) in mouse_hit_data.iter(){
        if hit_data.camera != camera_entity {
            continue;
        }
        let Some(hit_position) = hit_data.position else {continue};
        if planes.contains(*entity){
            editor_pointer.plane_entity = Some(*entity);
            editor_pointer.plane_loc = Some(hit_position);
        }
        if let Ok(_node_entity) = nodes.get(*entity){
            editor_pointer.reset_click();
            return;
        }
    }
}

