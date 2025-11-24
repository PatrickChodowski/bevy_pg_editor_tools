use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::prelude::Press;
use bevy_pg_nav::prelude::{GenerateNavMesh, NavMesh};
use crate::planes::PlaneToEdit;
use crate::terrain_brushes::{HeightBrushType, ColorBrushType, TerrainColorBrush, TerrainHeightBrush};
use crate::vertex::SpawnVertices;
use bevy_pg_scenes::prelude::{TerrainChunk, CurrentChunk, MapsData};

use crate::brushes::{NothingBrush, BrushSettings, ScatterBrush};
use crate::tracker::{Changes, Change, Undo, Redo, ChangesSet, ChangeDespawn, ChangeTransform, CurrentTransformChanges};

// use crate::input::controller::ToggleEditor;
use crate::ghost::{EditorAsset, Ghost, GhostTransformAxis, GhostTransformMode, EditorSettings};
use crate::tools::utils::rotate_point_2d;

pub struct PGEditorControllerPlugin;

impl Plugin for PGEditorControllerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_input_context::<EditorController>()
        .add_observer(set_translation_mode)
        .add_observer(set_rotation_mode)
        .add_observer(set_scale_mode)
        .add_observer(set_all_axis)
        .add_observer(set_xz_axis)
        .add_observer(set_xy_axis)
        .add_observer(set_y_axis)
        .add_observer(set_y_axis_origin)
        .add_observer(set_z_axis)
        .add_observer(set_x_axis)
        .add_observer(toggle_snap_nav)
        .add_observer(delete_object)
        .add_observer(change_value)
        .add_observer(change_value_scale)
        .add_observer(start_change_value)
        .add_observer(end_change_value)
        .add_observer(navmesh_generation)
        .add_observer(toggle_multi_ghost)
        .add_observer(unghost_all)
        .add_observer(toggle_brush)

        ;
    }
}




#[derive(Component, Reflect)]
pub struct EditorController;

pub fn editor_controller() -> impl Bundle {
    return (
        EditorController,
        actions!(
            EditorController[
                (
                    Action::<ChangeAnim>::new(),
                    Press::default(),
                    Bindings::spawn(
                        Bidirectional::<Binding, Binding> {
                            negative: KeyCode::Comma.into(), 
                            positive: KeyCode::Period.into()
                        }
                    )
                ),
                (
                    Action::<ChangeValueScale>::new(),
                    Press::default(),
                    Bindings::spawn(
                        Bidirectional::<Binding, Binding> {
                            negative: KeyCode::ArrowDown.into(), 
                            positive: KeyCode::ArrowUp.into()
                        }
                    )
                ),
                (
                    Action::<ToggleBrush>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyM]
                ),
                (
                    Action::<ToggleEditor>::new(),
                    HoldAndRelease::new(1.0),
                    bindings![KeyCode::Tab]
                ),
                (
                    Action::<NavMeshGeneration>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyG]
                ),
                (
                    Action::<UnghostAll>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyU]
                ),
                (
                    Action::<ToggleMultiGhost>::new(),
                    Press::default(),
                    bindings![KeyCode::ShiftLeft]
                ),
                (
                    Action::<TriggerThumbnails>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyL] // L as I have no other idea for name
                ),
                (
                    Action::<SaveScene>::new(),
                    Press::default(),
                    bindings![KeyCode::Enter]
                ),
                (
                    Action::<SetYAxisOrigin>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit0]
                ),
                (
                    Action::<SetTranslationMode>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit1]
                ),
                (
                    Action::<SetRotationMode>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit2]
                ),
                (
                    Action::<SetScaleMode>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit3]
                ),
                (
                    Action::<XY>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit4]
                ),
                (
                    Action::<XZ>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit5]
                ),
                (
                    Action::<AllAxis>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit6]
                ),
                (
                    Action::<SetXAxis>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit7]
                ),
                (
                    Action::<SetYAxis>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit8]
                ),
                (
                    Action::<SetZAxis>::new(),
                    Press::default(),
                    bindings![KeyCode::Digit9]
                ),
                (
                    Action::<Undo>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyZ]
                ),
                (
                    Action::<Redo>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyX]
                ),
                (
                    Action::<SpawnHelpDisplay>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyH]
                ),

                (
                    Action::<ToggleSnapNav>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyN]
                ),
                (
                    Action::<DeleteObject>::new(),
                    Press::default(),
                    bindings![KeyCode::BracketLeft]
                ),
                (
                    Action::<ChangeValue>::new(),
                    Down::default(),
                    Bindings::spawn(
                        Bidirectional::<Binding, Binding> {
                            negative: KeyCode::ArrowLeft.into(), 
                            positive: KeyCode::ArrowRight.into()
                        }
                    )
                )
            ]
        )
    );
}




#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct TriggerThumbnails;


#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct SaveScene;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleBrush;

#[derive(InputAction)]
#[action_output(f32)]
struct ChangeValue;

#[derive(InputAction)]
#[action_output(f32)]
pub struct ChangeValueScale;


fn start_change_value(
    _trigger:       On<Start<ChangeValue>>,
    transforms:     Query<(Entity, &Transform), With<Ghost>>,
    mut commands:   Commands
){
    let mut ctcs = CurrentTransformChanges::new();
    for (entity, transform) in transforms.iter(){
        ctcs.data.insert(entity, ChangeTransform::new(entity, *transform));
    }
    commands.insert_resource(ctcs);
}

fn end_change_value(
    _trigger:       On<Complete<ChangeValue>>,
    transforms:     Query<(Entity, &Transform), With<Ghost>>,
    mut changes:    ResMut<Changes>,
    mut commands:   Commands,
    mut ctcs:       ResMut<CurrentTransformChanges>
){
    let mut cts = ChangesSet::new();
    for (entity, transform) in transforms.iter(){
        let ct = ctcs.data.get_mut(&entity).unwrap();
        ct.new = *transform;
        if ct.old != ct.new {
            cts.add(*ct);
        }
    }
    if cts.len() > 0 {
        cts.record(&mut changes);
    }
    commands.remove_resource::<CurrentTransformChanges>();
}

fn change_value_scale(
    trigger:        On<Fire<ChangeValueScale>>,
    mut ghs:            ResMut<EditorSettings>
){
    ghs.change_value_scale += trigger.value*0.1;
    ghs.change_value_scale = ghs.change_value_scale.clamp(0.1, 1.0);
}


fn change_value(
    trigger:        On<Fire<ChangeValue>>,
    mut transforms: Query<&mut Transform, With<Ghost>>,
    ghs:            Res<EditorSettings>,
    navmesh:        Res<NavMesh>
){

    let delta_i32 = trigger.value as i32;
    if delta_i32 == 0 {
        return;
    }
    let d = delta_i32 as f32;

    let mut origin: Option<Vec2> = None;

    // Special case for OriginY and Rotation: need origin point of many transforms
    if ghs.axis == GhostTransformAxis::OriginY && ghs.mode == GhostTransformMode::Rotation {
        let count = transforms.iter().len();
        match count {
            0 => {}
            1 => {
                origin = Some(transforms.iter().next().unwrap().translation.xz());
            }
            _ => {
                // Average:
                // origin = Some(transforms.iter().map(|t| t.translation.xz()).sum::<Vec2>()/count as f32);

                // Middle:
                    let (min_x, max_x, min_z, max_z) = transforms.iter()
                    .map(|t| t.translation)
                    .fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |(min_x, max_x, min_z, max_z), pos| (
                        min_x.min(pos.x),
                        max_x.max(pos.x),
                        min_z.min(pos.z),
                        max_z.max(pos.z)));
                    origin = Some(Vec2::new((min_x + max_x) * 0.5, (min_z + max_z) *0.5));
            }
        }
    }

    for mut transform in transforms.iter_mut(){
        match ghs.mode {
            GhostTransformMode::Translation => {
                let sd = d*ghs.change_value_scale;
                match ghs.axis {
                    GhostTransformAxis::X => {
                        transform.translation.x += sd;
                        if ghs.snap_nav {
                            if let Some((_poly, height)) = navmesh.get_polygon_height(transform.translation.xz()){
                                transform.translation.y = height;
                            }
                        }     
                    }
                    GhostTransformAxis::Y => {transform.translation.y += sd}
                    GhostTransformAxis::Z => {
                        transform.translation.z += sd;
                        if ghs.snap_nav {
                            if let Some((_poly, height)) = navmesh.get_polygon_height(transform.translation.xz()){
                                transform.translation.y = height;
                            }
                        }  
                    }
                    _ => {}
                }
            }
            GhostTransformMode::Rotation => {
                let sd = d*0.01*ghs.change_value_scale;
                match ghs.axis {
                    GhostTransformAxis::X => {transform.rotate_x(sd)}
                    GhostTransformAxis::Y => {transform.rotate_y(sd)}
                    GhostTransformAxis::Z => {transform.rotate_z(sd)}
                    GhostTransformAxis::OriginY => {
                        if let Some(origin) = origin {
                            transform.translation = rotate_point_2d(&transform.translation, &origin, sd);
                            transform.rotate_y(-sd);
                            if ghs.snap_nav {
                                if let Some((_poly, height)) = navmesh.get_polygon_height(transform.translation.xz()){
                                    transform.translation.y = height;
                                }
                            }  
                        }
                    }
                    GhostTransformAxis::All => {
                        transform.rotate_x(sd);
                        transform.rotate_y(sd);
                        transform.rotate_z(sd);
                    }
                    GhostTransformAxis::XZ => {
                        transform.rotate_x(sd);
                        transform.rotate_z(sd);
                    }
                    GhostTransformAxis::XY => {
                        transform.rotate_x(sd);
                        transform.rotate_y(sd);
                    }
                }
            }
            GhostTransformMode::Scale => {
                let sd = d*ghs.change_value_scale;
                match ghs.axis {
                    GhostTransformAxis::X => {transform.scale.x += sd}
                    GhostTransformAxis::Y => {transform.scale.y += sd}
                    GhostTransformAxis::Z => {transform.scale.z += sd}
                    GhostTransformAxis::XY => {
                        transform.scale.x += sd;
                        transform.scale.z += sd; // Its inversed somehow
                    }
                    GhostTransformAxis::XZ => {
                        transform.scale.x += sd;
                        transform.scale.y += sd; // Its inversed somehow
                    }
                    GhostTransformAxis::All => {
                        transform.scale += sd;
                    }
                    _ => {}
                }
            }

        }
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct SpawnHelpDisplay;

#[derive(InputAction)]
#[action_output(bool)]
struct SetTranslationMode;

#[derive(InputAction)]
#[action_output(bool)]
struct SetRotationMode;

#[derive(InputAction)]
#[action_output(bool)]
struct SetScaleMode;

#[derive(InputAction)]
#[action_output(bool)]
struct SetXAxis;

#[derive(InputAction)]
#[action_output(bool)]
struct SetZAxis;

#[derive(InputAction)]
#[action_output(bool)]
struct SetYAxis;

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct SetYAxisOrigin;

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct AllAxis;

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct XZ;

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct XY;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleSnapNav;

#[derive(InputAction)]
#[action_output(bool)]
struct DeleteObject;


#[derive(InputAction)]
#[action_output(bool)]
pub struct NavMeshGeneration;


#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleMultiGhost;

#[derive(InputAction)]
#[action_output(bool)]
pub struct UnghostAll;


fn set_translation_mode(
    _trigger:    On<Fire<SetTranslationMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.mode = GhostTransformMode::Translation;
}

fn set_rotation_mode(
    _trigger:    On<Fire<SetRotationMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.mode = GhostTransformMode::Rotation;
}

fn set_scale_mode(
    _trigger:    On<Fire<SetScaleMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.mode = GhostTransformMode::Scale;
}


fn set_x_axis(
    _trigger:    On<Fire<SetXAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::X;
}

fn set_y_axis(
    _trigger:    On<Fire<SetYAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::Y;
}

fn set_all_axis(
    _trigger:    On<Fire<AllAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::All;
}

fn set_xz_axis(
    _trigger:    On<Fire<XZ>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::XZ;
}

fn set_xy_axis(
    _trigger:    On<Fire<XY>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::XY;
}

fn set_y_axis_origin(
    _trigger:    On<Fire<SetYAxisOrigin>>,
    mut ghost_settings: ResMut<EditorSettings> 
){
    ghost_settings.axis = GhostTransformAxis::OriginY;
}

fn set_z_axis(
    _trigger:    On<Fire<SetZAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.axis = GhostTransformAxis::Z;
}

fn toggle_snap_nav(
    _trigger:    On<Fire<ToggleSnapNav>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.snap_nav = !ghost_settings.snap_nav;
}

fn toggle_brush(
    _trigger:    On<Fire<ToggleBrush>>,
    mut commands: Commands,
    mut bs:      ResMut<BrushSettings>,
    mut editor_brushes: ResMut<EditorBrushes>,
    terrain_chunks: Query<Entity, (With<TerrainChunk>, With<PlaneToEdit>)>
){

    match *editor_brushes {
        EditorBrushes::Trees => {*editor_brushes = EditorBrushes::Grass;}
        EditorBrushes::Grass => {*editor_brushes = EditorBrushes::Rocks;}
        EditorBrushes::Rocks => {*editor_brushes = EditorBrushes::Textures;}
        EditorBrushes::Textures => {*editor_brushes = EditorBrushes::TerrainHeight;}
        EditorBrushes::TerrainHeight => {*editor_brushes = EditorBrushes::TerrainColor;}
        EditorBrushes::TerrainColor => {*editor_brushes = EditorBrushes::Trees;}
    }
    bs.radius = 30.0;
    match *editor_brushes {
        EditorBrushes::Trees => {
            bs.typ = Box::new(ScatterBrush::trees())
        }
        EditorBrushes::Grass => {
            bs.typ = Box::new(ScatterBrush::grass())
        }
        EditorBrushes::Rocks => {
            bs.typ = Box::new(ScatterBrush::rocks())
        }
        EditorBrushes::Textures => {
            bs.typ = Box::new(NothingBrush)
        }
        EditorBrushes::TerrainHeight => {
            bs.typ = Box::new(TerrainHeightBrush{typ: HeightBrushType::Value(1.0), reselection: true});
            for entity in terrain_chunks.iter(){
                commands.trigger(SpawnVertices{plane_entity: entity});
            }
        }
        EditorBrushes::TerrainColor => {
            bs.typ = Box::new(TerrainColorBrush{typ: ColorBrushType::Value{clr: [0.5, 0.5, 0.5, 1.0]}});
            for entity in terrain_chunks.iter(){
                commands.trigger(SpawnVertices{plane_entity: entity});
            }
        }
    }
}

fn delete_object(
    _trigger:    On<Fire<DeleteObject>>,
    mut commands: Commands,
    query:       Query<(Entity, &EditorAsset, &Transform), With<Ghost>>,
    mut changes: ResMut<Changes>
){
    let mut despawns = ChangesSet::new();
    for (entity, asset, transform) in query.iter(){
        commands.entity(entity).try_despawn();
        despawns.add(ChangeDespawn::new(entity, asset.clone(), *transform));
    }
    if despawns.len() > 0 {
        despawns.record(&mut changes);
    }
}

fn navmesh_generation(
    _trigger:       On<Fire<NavMeshGeneration>>,
    mut commands:   Commands,
    current_chunk:  Res<CurrentChunk>,
    terrain_chunks: Query<(&TerrainChunk, &Name)>,
    mapsdata:       Res<MapsData>
){
    for (terrain_chunk, name) in terrain_chunks.iter(){
        if (terrain_chunk.map_name == current_chunk.map_name) &
           (terrain_chunk.chunk_id == current_chunk.chunk_id) {

            commands.write_message(GenerateNavMesh::new(
                name.to_string(), 
                &current_chunk.map_name, 
                &current_chunk.chunk_id,
                mapsdata.chunk_size
            
            ));
            break;
        }
    }
}

fn toggle_multi_ghost(
    _trigger:       On<Fire<ToggleMultiGhost>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.multi_ghost = !ghost_settings.multi_ghost;
}

fn unghost_all(
    _trigger: On<Fire<UnghostAll>>,
    mut commands: Commands,
    query:        Query<Entity, With<Ghost>>
){
    for entity in query.iter(){
        commands.entity(entity).remove::<Ghost>();
    }
}