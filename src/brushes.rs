use bevy::platform::collections::HashMap;
use bevy::ecs::system::SystemState;
use bevy::color::palettes::tailwind::BLUE_500;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy_enhanced_input::prelude::Cancel;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use bevy_enhanced_input::prelude::*;
use bevy_pg_core::prelude::PointerData;
use dyn_clone::DynClone;
use rand::Rng;
use rand::seq::IndexedRandom;
use bevy_pg_nav::prelude::PGNavmesh;

use crate::prelude::{EditorMode, EditorSettings};
use crate::tracker::{Changes, Change, ChangesSet, ChangeSpawn};
use crate::ghost::{EditorAsset, EditorGhostSettings, Ghost, editor_asset_bundle};


pub struct PGEditorBrushSelectPlugin;

impl Plugin for PGEditorBrushSelectPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_input_context::<BrushSelectController>()
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
                 context.spawn((Action::<BrushSelectUpdate>::new(), Down::default(), bindings![MouseButton::Left]));
        })) 
    );
}

#[derive(InputAction)]
#[action_output(bool)]
struct BrushSelectUpdate;


fn start_brush(
    _trigger:          On<Start<BrushSelectUpdate>>,
    input_data:        Res<PointerData>,
    mut commands:      Commands,
    mut meshes:        ResMut<Assets<Mesh>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    editor_settings:   Res<EditorSettings>,
    brushes:           Query<Entity, With<BrushMarker>>
){
    if editor_settings.mode != EditorMode::Brushes {
        return;
    }

    for brush_entity in brushes.iter(){
        commands.entity(brush_entity).despawn();
    }

    let Some(world_pos) = input_data.world_pos else {return};
    let loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
    let brush = Brush{
        loc, 
        radius: editor_settings.brush_radius, 
        typ: editor_settings.brush_typ.clone()
    };

    commands.insert_resource(brush);
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(editor_settings.brush_radius))),
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
    _trigger:               On<Fire<BrushSelectUpdate>>,
    input_data:             Res<PointerData>,
    mut brush_transform:    Single<&mut Transform, With<BrushMarker>>,
    mut brush:              ResMut<Brush>,
    editor_settings:   Res<EditorSettings>
){
    if editor_settings.mode != EditorMode::Brushes {
        return;
    }

    let Some(world_pos) = input_data.world_pos else {return};
    if world_pos.xz() != brush.loc.xz(){
        brush.loc = Vec3::new(world_pos.x, world_pos.y + 1.0, world_pos.z);
        brush_transform.translation = brush.loc;
    }
}

fn end_brush(
    _trigger:       On<Cancel<BrushSelectUpdate>>,
    mut commands:   Commands,
    brush_entity:   Single<Entity, With<BrushMarker>>,
    editor_settings:   Res<EditorSettings>
){
    if editor_settings.mode != EditorMode::Brushes {
        return;
    }

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




/*  Different brush types */
#[derive(Clone)]
pub enum StrokeTest {
    Negative,
    Positive(ChangeSpawn)
}

#[derive(Clone)]
pub struct ScatterBrush {
    pub assets:       Vec<&'static str>,
    pub radius_inner: f32,
    pub chance:       f32,
    pub scale:        (f32, f32),
    pub rotation:     (f32, f32),
    pub nudges:       (f32, f32),
    pub data:         HashMap<(u32, u32), StrokeTest>,
    pub locs:         Vec<Vec2>
}
impl BrushType for ScatterBrush {
    fn started(&mut self, world:&mut World) {
        // info!("Started scatterbrush");
    }

    fn apply(&mut self, world: &mut World, loc: Vec3, radius: f32) {

        let locs: Vec<Vec2> = pack_circles(self.radius_inner, radius, loc.x, loc.z);
        let threshold = self.radius_inner*2.0*self.radius_inner*2.0;

        let mut system_state: SystemState<(
                ResMut<Assets<Mesh>>,
                ResMut<Assets<StandardMaterial>>,
                Res<AssetServer>,
                Commands,
                Query<&PGNavmesh>,
                Res<EditorGhostSettings>
            )> = SystemState::new(world);

        let (mut meshes, mut materials, ass, mut commands, navs, ghost_settings) = system_state.get_mut(world);

        for loc in locs.iter(){
            let uloc = (loc.x as u32, loc.y as u32);

            if self.data.contains_key(&uloc){
                continue;
            }
            if self.locs.iter().any(|&p| loc.distance_squared(p) < threshold) {
                continue;
            }

            let mut rng = rand::rng();
            let random_chance: f32 = rng.random_range(0.0..1.0);
            self.locs.push(*loc);

            if random_chance > self.chance {
                self.data.insert(uloc, StrokeTest::Negative);
                continue;
            }

            let asset = EditorAsset::Asset(self.assets.choose(&mut rng).unwrap().to_string());
            let scale = rng.random_range(self.scale.0..self.scale.1);
            let random_angle = rng.random_range(self.rotation.0..self.rotation.1);
            let q = Quat::from_euler(EulerRot::XYZ, -FRAC_PI_2, 0.0, random_angle);

            let nudge_x = rng.random_range(self.nudges.0..self.nudges.1)*self.radius_inner;
            let nudge_z = rng.random_range(self.nudges.0..self.nudges.1)*self.radius_inner;

            let mut pos = Vec3::new(loc.x+nudge_x, loc.y, loc.y+nudge_z);

            for navmesh in navs.iter(){
                if let Some((_poly, height)) = navmesh.get_polygon_height(loc){
                    pos.y = height;
                    break;
                }
            }

            let transform = Transform::from_translation(pos).with_rotation(q).with_scale(Vec3::splat(scale));

            let entity = commands.spawn(
                editor_asset_bundle(
                    asset.clone(),
                    &ass,
                    &mut meshes,
                    &mut materials,
                    &transform,
                    &ghost_settings
                )
            ).id();

            commands.entity(entity).remove::<Ghost>();        
            let change_spawn = ChangeSpawn::new(entity, asset, transform);
            self.data.insert(uloc, StrokeTest::Positive(change_spawn));
        }
        system_state.apply(world);
        
    }

    fn done(&mut self, world: &mut World) {
        // info!("Done scatterbrush");

        let mut system_state: SystemState<ResMut<Changes>> = SystemState::new(world);
        let mut changes = system_state.get_mut(world);

        let mut cts = ChangesSet::new();
        for (_k, v) in self.data.iter(){
            match v {
                StrokeTest::Positive(ct) => {
                    cts.add(ct.clone());
                }
                _ => {}
            }
        }
        cts.record(&mut changes);
        system_state.apply(world);
    }
}

fn pack_circles(
    radius_inner: f32, 
    radius_outer: f32, 
    cx: f32, 
    cy: f32
) -> Vec<Vec2> {

    // Vertical spacing between circle centers
    let delta_y = ((2.0 * radius_inner).powi(2) - radius_inner.powi(2)).sqrt();

    // 1. Estimate max number of circles along diameter
    let mut n_row_max = (radius_outer / radius_inner).floor() as i32;

    let (big_x, big_y) = if n_row_max % 2 == 1 {
        // odd
        n_row_max += 2;
        let y = 0.5 * (n_row_max as f32 - 1.0) * delta_y;
        let x = (n_row_max as f32 - 1.0) * radius_inner;
        (x, y)
    } else {
        // even
        n_row_max += 2;
        let y = 0.5 * n_row_max as f32 * delta_y;
        let x = (n_row_max as f32 + 1.0) * radius_inner;
        (x, y)
    };

    let mut inside = Vec::new();

    for row in 0..n_row_max {
        for col in 0..n_row_max {
            let x = (col as f32) * 2.0 * radius_inner + (1.0 + (-1.0f32).powf(row as f32)) * 0.5 * radius_inner;
            let y = (row as f32) * delta_y;
            let dist = ((x - big_x).powi(2) + (y - big_y).powi(2)).sqrt() + radius_inner;
            if dist <= radius_outer {
                inside.push(Vec2::new(
                    cx + (x - big_x),
                    cy + (y - big_y)
                ));
            }
        }
    }

    inside
}

