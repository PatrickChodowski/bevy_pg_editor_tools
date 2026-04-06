use bevy::{color::palettes::tailwind::{BLUE_400, ORANGE_700}, prelude::*};
use libm::fabsf;

use crate::editor_pointer::EditorPointer;


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
                display_boxselect
            ).chain()
        );
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
    editor_pointer:  Res<EditorPointer>,
    mut box_select:  ResMut<BoxSelect>,
    mouse:           Res<ButtonInput<MouseButton>>,
    keys:            Res<ButtonInput<KeyCode>>,
){
    
    // Start drag
    if mouse.just_pressed(MouseButton::Middle) && keys.pressed(KeyCode::KeyB) && !box_select.active {
        let Some(world_pos) = editor_pointer.loc else {return};
        box_select.active = true;
        box_select.start = world_pos;
        box_select.loc = world_pos;
        box_select.dims  = Vec2::ZERO;
        return;
    }

    // Dragging
    if box_select.active && mouse.pressed(MouseButton::Middle) && keys.pressed(KeyCode::KeyB) {
        let Some(world_pos) = editor_pointer.loc else {return};
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

#[derive(Event)]
pub struct BoxSelectFinal{
    aabb: AABB
}
impl BoxSelectFinal {
    pub fn has_point(&self, loc: Vec2) -> bool {
        self.aabb.has_point(loc)
    }
}


#[derive(Resource, Debug)]
pub struct BoxSelect {
    pub active:  bool,
    pub start:  Vec3,
    pub loc:    Vec3,
    pub dims:   Vec2
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct AABB {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

impl Default for AABB {
    fn default() -> Self {
        return AABB{
            min_x: 0.0, 
            max_x: 0.0,
            min_z: 0.0, 
            max_z: 0.0
        };
    }
}

impl AABB {
    pub fn from_loc_dims(loc: Vec2, dim: Vec2) -> AABB {
        AABB {
            min_x: loc.x - dim.x / 2.0,
            max_x: loc.x + dim.x / 2.0,
            min_z: loc.y - dim.y / 2.0,
            max_z: loc.y + dim.y / 2.0,
        }
    }

    pub fn has_point(&self, loc: Vec2) -> bool {
        loc.x >= self.min_x && loc.x <= self.max_x && loc.y >= self.min_z && loc.y <= self.max_z
    }
}