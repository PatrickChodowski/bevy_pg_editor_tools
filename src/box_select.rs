use bevy::{color::palettes::tailwind::{ORANGE_700}, prelude::*};
use bevy_pg_core::prelude::AABB;
use libm::fabsf;

use crate::editor_pointer::EditorPointer;
use crate::ghost::{EditorAsset, Ghost, GhostMark};
use crate::EditorSettings;
use crate::prelude::EditorMode;
use crate::planes::PlaneToEdit;



pub struct PGEditorBoxSelectPlugin;

impl Plugin for PGEditorBoxSelectPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BoxSelect::default())
        .init_gizmo_group::<BoxSelectGizmos>()
        .add_systems(Startup, setup_gizmo_config)
        .add_systems(Update,
            (
                update_boxselect,
                display_boxselect,
                select_items.run_if(box_select_changed)
            ).chain()
        )
        .add_observer(on_box_select_final)
        ;
    }
}

fn on_box_select_final(
    _trigger:     On<BoxSelectFinal>,
    mut commands: Commands,
    ghost_marks:  Query<(Entity, &GhostMark)>
){
    for (entity, ghost_mark) in ghost_marks.iter(){
        commands.entity(entity).remove::<GhostMark>();
        commands.entity(entity).insert(Ghost{material_after: ghost_mark.material_after.clone()});
    }

}


fn select_items(
    mut commands:    Commands,
    box_select:      Res<BoxSelect>,
    editor_settings: Res<EditorSettings>,
    planes:          Query<(Entity, &Transform, &PlaneToEdit, &MeshMaterial3d<StandardMaterial>, Option<&GhostMark>)>,
    objects:         Query<(Entity, &Transform, &EditorAsset, &MeshMaterial3d<StandardMaterial>, Option<&GhostMark>)>,
){

    if !box_select.active {
        return;
    }
    let aabb = box_select.aabb();

    match editor_settings.mode {
        EditorMode::Brushes => {}
        EditorMode::Plane => {
            for (plane_entity, plane_transform, plane, material, maybe_ghost_mark) in planes.iter(){
                let plane_aabb = AABB::from_loc_dims(plane_transform.translation.xz(), plane.dims());
                if aabb.intercepts(&plane_aabb){
                    if maybe_ghost_mark.is_none(){
                        commands.entity(plane_entity).insert(GhostMark{material_after: material.0.clone()});
                    }
                } else {
                    if maybe_ghost_mark.is_some(){
                        commands.entity(plane_entity).remove::<GhostMark>();
                    }
                }
            }
        }
        EditorMode::Scene => {
            for (entity, transform, _asset, material, maybe_ghost_mark) in objects.iter(){
                if aabb.has_point(transform.translation.xz()){
                    if maybe_ghost_mark.is_none(){
                        commands.entity(entity).insert(GhostMark{material_after: material.0.clone()});
                    }
                } else {
                    if maybe_ghost_mark.is_some(){
                        commands.entity(entity).remove::<GhostMark>();
                    }
                }
            }
        }
    }


}



#[derive(Default, Reflect, GizmoConfigGroup)]
struct BoxSelectGizmos;

fn setup_gizmo_config(
    mut config_store: ResMut<GizmoConfigStore>
){
    let (config, _) = config_store.config_mut::<BoxSelectGizmos>();
    config.depth_bias = -1.0;  
}

fn update_boxselect(
    mut commands:    Commands,
    editor_pointer:  Res<EditorPointer>,
    mut box_select:  ResMut<BoxSelect>,
    mouse:           Res<ButtonInput<MouseButton>>,
    keys:            Res<ButtonInput<KeyCode>>,
    editor_settings: Res<EditorSettings>
){
    if editor_settings.mode == EditorMode::Brushes{
        return;
    }
    
    // Start drag
    if mouse.just_pressed(MouseButton::Middle) && keys.pressed(KeyCode::KeyB) && !box_select.active {
        let Some(world_pos) = editor_pointer.y0_pos else {return};
        box_select.active = true;
        box_select.start = world_pos;
        box_select.loc = world_pos;
        box_select.dims  = Vec2::ZERO;
        return;
    }

    // Dragging
    if box_select.active && mouse.pressed(MouseButton::Middle) && keys.pressed(KeyCode::KeyB) {
        let Some(world_pos) = editor_pointer.y0_pos else {return};

        if world_pos.distance(box_select.loc) <= 1.0 {
            // ignore as too little distance, too much noise
            return
        }

        let dim_x = fabsf(world_pos.x - box_select.start.x);
        let dim_z = fabsf(world_pos.z - box_select.start.z);
        let dims = Vec2::new(dim_x, dim_z);
        box_select.loc = world_pos;
        box_select.dims = dims; 
        return;
    }

    // End drag
    if box_select.active && (mouse.just_released(MouseButton::Middle) || keys.just_released(KeyCode::KeyB)) {
        // let Some(world_pos) = editor_pointer.loc else {return};
        *box_select = BoxSelect::default();
        commands.trigger(BoxSelectFinal);
        return;
    }
}


fn display_boxselect(
    box_select:  Res<BoxSelect>,
    mut gizmos:  Gizmos<BoxSelectGizmos>
){
    if !box_select.active {
        return;
    }

    let gizmo_color = Color::from(ORANGE_700);
    let corner1 = Vec3::new(box_select.start.x, box_select.start.y, box_select.loc.z);
    let corner2 = Vec3::new(box_select.loc.x, box_select.start.y, box_select.start.z);
    let aabb = box_select.aabb();

    gizmos.sphere(Isometry3d::from_xyz(aabb.min_x, 0.0, aabb.min_z), 0.5, gizmo_color);
    gizmos.sphere(Isometry3d::from_xyz(aabb.max_x, 0.0, aabb.max_z), 0.5, gizmo_color);
    gizmos.sphere(Isometry3d::from_xyz(aabb.min_x, 0.0, aabb.max_z), 0.5, gizmo_color);
    gizmos.sphere(Isometry3d::from_xyz(aabb.max_x, 0.0, aabb.min_z), 0.5, gizmo_color);

    gizmos.line(box_select.start, corner1, gizmo_color);
    gizmos.line(corner1, box_select.loc, gizmo_color);
    gizmos.line(box_select.loc, corner2, gizmo_color);
    gizmos.line(corner2, box_select.start, gizmo_color);


}

pub fn box_select_changed(
    box_select: Res<BoxSelect>
) -> bool {
    box_select.is_changed()
}

#[derive(Resource, Debug)]
pub struct BoxSelect {
    pub active:  bool,
    pub start:  Vec3,
    pub loc:    Vec3,
    pub dims:   Vec2
}

impl BoxSelect {
    fn aabb(&self) -> AABB {
        let center = (self.loc + self.start) * 0.5;
        AABB::from_loc_dims(center.xz(), self.dims)
    }
}


impl Default for BoxSelect {
    fn default() -> Self {
        BoxSelect {
            active: false,
            start: Vec3::ZERO,
            loc: Vec3::ZERO,
            dims: Vec2::ZERO
        }
    }
}


#[derive(Event)]
pub struct BoxSelectFinal;
