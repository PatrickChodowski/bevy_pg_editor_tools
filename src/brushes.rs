use bevy::color::palettes::tailwind::BLUE_500;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy_enhanced_input::prelude::Cancel;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use bevy_enhanced_input::prelude::*;

use crate::prelude::WorldPos;


pub struct PGEditorBrushSelectPlugin;

impl Plugin for PGEditorBrushSelectPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_input_context::<BrushSelectController>()
        .insert_resource(BrushSettings::default())
        .add_observer(start_brush)
        .add_observer(update_brush)
        .add_observer(end_brush)
        ;
    }
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
    brush_settings:    Res<BrushSettings>,
    // brushes:            Query<Entity, With<BrushStroke>>,
){
    // if brushes.iter().len() > 0 {
    //     for entity in brushes.iter(){
    //         commands.entity(entity).despawn();
    //     }
    // }

    let Some(world_pos) = input_data.get() else {return;};
    // if ghosts.iter().len() > 0 {
    //     return;
    // }
    let loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(brush_settings.radius))),
        MeshMaterial3d(materials.add(Color::from(BLUE_500).with_alpha(0.4))),
        Transform::from_xyz(world_pos.x, world_pos.y + 1.0, world_pos.z)
                  .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        // BrushStroke::new(brush_settings.typ.clone(), brush_settings.radius),
        Brush{loc},
        NotShadowCaster,
        NotShadowReceiver
    ));

    commands.trigger(BrushStart);

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
    _trigger:   On<Fire<BrushSelectUpdate>>,
    input_data: Res<WorldPos>,
    query:      Single<(&mut Transform, &mut Brush)>
){
    let Some(world_pos) = input_data.get() else {return;};
    let (mut transform, mut brush) = query.into_inner();
    if world_pos.xz() != brush.loc.xz(){
        brush.loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
        transform.translation = brush.loc;
    }
}

fn end_brush(
    _trigger:       On<Cancel<BrushSelectUpdate>>,
    mut commands:   Commands,
    query:          Single<(Entity, &Brush)>
){
    let (brush_entity, brush) = query.into_inner();
    commands.trigger(BrushFinal);
    commands.entity(brush_entity).despawn();
}

#[derive(Event)]
pub struct BrushFinal;

#[derive(Event)]
pub struct BrushStart;

#[derive(Component)]
pub struct Brush {
    loc: Vec3
}


#[derive(Resource)]
pub struct BrushSettings {
    pub radius: f32
}

impl Default for BrushSettings {
    fn default() -> Self {
        BrushSettings {
            radius: 30.0
        }
    }
}

pub fn brush_changed(
    query: Query<Entity, Changed<Brush>>
) -> bool {
    !query.is_empty()
}
