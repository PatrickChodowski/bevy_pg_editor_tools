use bevy::color::palettes::tailwind::BLUE_500;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy_enhanced_input::prelude::Cancel;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use bevy_enhanced_input::prelude::*;
use dyn_clone::DynClone;

use crate::prelude::WorldPos;


pub struct PGEditorBrushSelectPlugin;

impl Plugin for PGEditorBrushSelectPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BrushSettings::default())
        .add_message::<BrushStart>()
        .add_message::<BrushDone>()
        .add_observer(start_brush)
        .add_observer(update_brush)
        .add_observer(end_brush)
        .add_systems(Update, 
            (
                brush_started.run_if(on_message::<BrushStart>),
                brush_apply.run_if(resource_exists::<Brush>),
                brush_final.run_if(on_message::<BrushDone>)
            )
        )
        .add_systems(Update, brush_apply.run_if(brush_changed))
        ;
    }
}

#[derive(Resource)]
pub struct BrushSettings {
    pub radius: f32,
    pub typ: Box<dyn BrushType>
}
impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            radius: 10.0,
            typ: Box::new(NothingBrush)            
        }
    }
}


fn brush_started(
    world:   &mut World,
){
    world.resource_scope(|_world: &mut World, mut brush: Mut<Brush>| {
        brush.typ.started(_world);
    });
}

fn brush_apply(
    world:     &mut World,
){
    world.resource_scope(|_world: &mut World, mut brush: Mut<Brush>| {
        let radius = brush.radius;
        let loc = brush.loc;
        brush.typ.apply(_world, loc, radius);
    });
}

fn brush_final(
    world:     &mut World,
){
    world.resource_scope(|_world: &mut World, mut brush: Mut<Brush>| {
        brush.typ.done(_world);
    });

    world.remove_resource::<Brush>();
}


#[derive(Component, Reflect)]
pub struct BrushSelectController;

pub fn brush_select_controller() -> impl Bundle {
    return (
        BrushSelectController,
        Actions::<BrushSelectController>::spawn(
            SpawnWith(|context: &mut ActionSpawner<_>| {
            let member1 = context
                .spawn((Action::<BrushSelectUpdate1>::new(), Down::default(), bindings![KeyCode::KeyB]))
                .id();
            let member2 = context
                .spawn((Action::<BrushSelectUpdate2>::new(), Down::default(), bindings![MouseButton::Left]))
                .id();
            context.spawn((Action::<BrushSelectUpdate>::new(), Chord::new([member1, member2])));

        })) 
    );
}


#[derive(InputAction)]
#[action_output(bool)]
struct BrushSelectUpdate1;

#[derive(InputAction)]
#[action_output(bool)]
struct BrushSelectUpdate2;

#[derive(InputAction)]
#[action_output(bool)]
struct BrushSelectUpdate;


fn start_brush(
    _trigger:          On<Start<BrushSelectUpdate>>,
    input_data:        Res<WorldPos>,
    mut commands:      Commands,
    mut meshes:        ResMut<Assets<Mesh>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    brush_settings:    Res<BrushSettings>
){
    let Some(world_pos) = input_data.get() else {return;};
    let loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
    let brush = Brush{
        loc, 
        radius: brush_settings.radius, 
        typ: brush_settings.typ.clone()
    };
    commands.insert_resource(brush);
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(brush_settings.radius))),
        MeshMaterial3d(materials.add(Color::from(BLUE_500).with_alpha(0.4))),
        Transform::from_xyz(world_pos.x, world_pos.y + 1.0, world_pos.z)
                  .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        BrushMarker,
        NotShadowCaster,
        NotShadowReceiver
    ));
    commands.write_message(BrushStart);

    // Stuck on Assets as Entities
    // for (entity, terrain_material) in textures.iter_mut(){
    //     if let Some(material) = materials.get(&terrain_material.0){
    //         if let Some(ref base_color_texture) = material.base_color_texture {
    //             if let Some(image) = images.get_mut(base_color_texture){
    //                 let v = image.data.as_ref().unwrap()[0];
    //                 let count = image.data.as_ref().unwrap().len();
    //                 image.data = Some(vec![255; count]);
    //             }
    //             break;
    //         }
    //     }
    // }
}

fn update_brush(
    _trigger:               On<Ongoing<BrushSelectUpdate>>,
    input_data:             Res<WorldPos>,
    mut brush_transform:    Single<&mut Transform, With<BrushMarker>>,
    mut brush:              ResMut<Brush>
){
    let Some(world_pos) = input_data.get() else {return;};
    if world_pos.xz() != brush.loc.xz(){
        brush.loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
        brush_transform.translation = brush.loc;
    }
}

fn end_brush(
    _trigger:       On<Cancel<BrushSelectUpdate>>,
    mut commands:   Commands,
    brush_entity:   Single<Entity, With<BrushMarker>>,
){
    commands.write_message(BrushDone);
    commands.entity(*brush_entity).despawn();
}

#[derive(Message)]
pub struct BrushDone;

#[derive(Message)]
pub struct BrushStart;


#[derive(Component)]
pub struct BrushMarker;

#[derive(Resource)]
pub struct Brush {
    loc: Vec3,
    radius: f32,
    typ: Box<dyn BrushType>
}


pub fn brush_changed(
    maybe_brush: Option<Res<Brush>>
) -> bool {
    if let Some(res_brush) = maybe_brush {
        return res_brush.is_changed();
    } else{
        return false;
    }
}


pub trait BrushType:  Send + Sync + DynClone + 'static {
    fn started(&mut self, world:&mut World){}
    fn apply(&mut self, world: &mut World, loc: Vec3, radius: f32){}
    fn done(&mut self, world: &mut World){}
}
dyn_clone::clone_trait_object!(BrushType);

#[derive(Clone)]
pub struct NothingBrush;

impl BrushType for NothingBrush {
    fn started(&mut self, world:&mut World) {
        // info!("Started nothingbrush");
    }
    fn apply(&mut self, world: &mut World, loc: Vec3, radius: f32) {
        // info!("apply nothingbrush");
    }

    fn done(&mut self, world: &mut World) {
        // info!("Done nothingbrush");
    }
}