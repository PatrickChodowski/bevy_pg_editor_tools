use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Cancel;
use bevy_enhanced_input::prelude::*;
use bevy::color::palettes::tailwind::*;
use libm::fabsf;

use crate::world_pos::WorldPos;


pub struct PGEditorBoxSelectPlugin;

impl Plugin for PGEditorBoxSelectPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_input_context::<BoxSelectController>()
        .add_observer(start_boxselect)
        .add_observer(update_boxselect)
        .add_observer(end_boxselect)             
        ;
    }
}


#[derive(Component, Reflect)]
pub struct BoxSelectController;

pub fn box_select_controller() -> impl Bundle {
    return (
        BoxSelectController,
        Actions::<BoxSelectController>::spawn(
            SpawnWith(|context: &mut ActionSpawner<_>| {
            let member1 = context
                .spawn((Action::<BoxSelectUpdate1>::new(), Down::default(), bindings![KeyCode::KeyB]))
                .id();
            let member2 = context
                .spawn((Action::<BoxSelectUpdate2>::new(), Down::default(), bindings![MouseButton::Right]))
                .id();
            context.spawn((Action::<BoxSelectUpdate>::new(), Chord::new([member1, member2])));

            })) 
        );
}

#[derive(InputAction)]
#[action_output(bool)]
struct BoxSelectUpdate1;

#[derive(InputAction)]
#[action_output(bool)]
struct BoxSelectUpdate2;

#[derive(InputAction)]
#[action_output(bool)]
struct BoxSelectUpdate;

#[derive(Event)]
pub struct BoxSelectFinal{
    aabb: AABB
}
impl BoxSelectFinal {
    pub fn has_point(&self, loc: Vec2) -> bool {
        self.aabb.has_point(loc)
    }
}


#[derive(Component, Debug)]
pub struct BoxSelect {
    pub start:  Vec3,
    pub loc:    Vec3,
    pub dims:   Vec2
}

impl BoxSelect {
    fn new(loc: &Vec3) -> Self {
        BoxSelect {
            start: *loc,
            loc: *loc,
            dims: Vec2::ZERO
        }
    }
}

impl Default for BoxSelect {
    fn default() -> Self {
        BoxSelect {
            start: Vec3::ZERO,
            loc: Vec3::ZERO,
            dims: Vec2::ZERO
        }
    }
}


fn start_boxselect(
    _trigger:       On<Start<BoxSelectUpdate>>,
    mut commands:   Commands,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    input_data:     Res<WorldPos>
){
    let Some(world_pos) = input_data.get() else {return;};
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::from(ORANGE_600).with_alpha(0.4))),
        Transform::from_translation(world_pos),
        BoxSelect::new(&world_pos),
        NotShadowCaster,
        NotShadowReceiver
    ));
}


fn update_boxselect(
    _trigger:   On<Fire<BoxSelectUpdate>>,
    input_data: Res<WorldPos>,
    query:      Single<(&mut Transform, &mut BoxSelect)>
){
    let Some(world_pos) = input_data.get() else {return;};
    let (mut transform, mut box_select) = query.into_inner();

    let new_x = (world_pos.x + box_select.start.x) / 2.0;
    let new_z = (world_pos.z + box_select.start.z) / 2.0;
    let dim_x = fabsf(world_pos.x - box_select.start.x);
    let dim_z = fabsf(world_pos.z - box_select.start.z);
    let max_y = world_pos.y.max(box_select.start.y) + 0.1;  
    let dims = Vec2::new(dim_x, dim_z);
    let loc = Vec3A::new(new_x, max_y, new_z);
    box_select.loc = loc.into();
    box_select.dims = dims; 
    transform.translation = loc.into();
    transform.scale = Vec3::new(dims.x, 1.0, dims.y);
}

fn end_boxselect(
    _trigger:       On<Cancel<BoxSelectUpdate>>,
    mut commands:   Commands,
    query:          Single<(Entity, &BoxSelect)>
){
    let (bs_entity, box_select) = query.into_inner();
    let aabb = AABB::from_loc_dims(box_select.loc.xz(), box_select.dims);
    commands.trigger(BoxSelectFinal{aabb: aabb});
    commands.entity(bs_entity).despawn();
}

pub fn box_select_changed(
    query: Query<Entity, Changed<BoxSelect>>
) -> bool {
    !query.is_empty()
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