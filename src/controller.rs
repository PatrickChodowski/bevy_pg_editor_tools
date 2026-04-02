use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::mesh::SerializedMesh;
use bevy::tasks::IoTaskPool;
use bevy::platform::collections::HashMap;
use bevy::ui::InteractionDisabled;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::prelude::Press;
use bevy_pg_core::prelude::{TerrainChunk, GameStatePlay, rotate_point_2d};
use bevy_pg_nav::prelude::{GenerateNavMesh, PGNavmesh, NavConfig};
use bevy_pg_scenes::prelude::{CurrentChunk, MapsData, SceneData, SceneObjectData, Markee, Spawner, Marker, Static, PGSerializedMesh};
use bevy_simple_text_input::TextInputValue;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::assets_panel::EditorAssetPanel;
use crate::text_inputs::{LocInputX, LocInputY, LocInputZ, PlaneDimXInput, PlaneDimZInput, PlaneSubsInput};
use crate::tracker::{Change, ChangeDespawn, ChangePlaneSpawn, ChangeTransform, Changes, ChangesSet, CurrentTransformChanges, Redo, Undo};
use crate::ghost::{EditorAsset, Ghost};
use crate::transform_gizmo::{TransformGizmoConfig, TransformGizmoFocus, TransformGizmoMode};
use crate::ui::{BrushControls, EditorControlsPanel, PlaneControls, SceneControls, EditorControls};
use crate::planes::{PlaneToEdit, plane_mesh};
use crate::settings::{EditorMode, EditorSettings};
use crate::vertex::{DeselectAllVertices, HideVertices};

pub struct PGEditorControllerPlugin;

impl Plugin for PGEditorControllerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_input_context::<EditorController>()
        .add_observer(set_translation_mode)
        .add_observer(set_rotation_mode)
        .add_observer(set_scale_mode)
        .add_observer(unghost_all)
        .add_observer(on_fire_unghost_all)
        .add_observer(on_fire_save_scene)
        .add_observer(save_scene)
        .add_observer(delete_object)
        .add_observer(toggle_navmesh_debug)
        .add_observer(toggle_markers_vis)
        .add_observer(toggle_spawners_vis)
        .add_observer(toggle_nav_snap)
        .add_observer(toggle_plane_wireframe)
        .add_observer(change_brush)
        .add_observer(turn_off_editor)
        .add_observer(toggle_editor_panel)
        .add_observer(toggle_assets_panel)
        .add_observer(change_editor_mode)

        .add_observer(spawn_plane)
        ;
    }
}

fn toggle_navmesh_debug(
    trigger: On<ToggleNavmeshDebug>,
    mut navconfig:  ResMut<NavConfig>
){
    navconfig.debug = trigger.value;
}


fn string_to_f32(s: &str) -> Option<f32> {
    let trimmed = s.trim();
    
    if trimmed.is_empty() {
        return Some(0.0);
    }

    trimmed.parse::<f32>().ok()
}

fn string_to_u32(s: &str) -> Option<u32> {
    let trimmed = s.trim();
    
    if trimmed.is_empty() {
        return Some(0);
    }

    trimmed.parse::<u32>().ok()
}


fn spawn_plane(
    _trigger:          On<SpawnPlane>,
    editor_settings:   Res<EditorSettings>, 
    mut commands:      Commands,
    mut meshes:        ResMut<Assets<Mesh>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    loc_x:             Single<&TextInputValue, With<LocInputX>>,
    loc_y:             Single<&TextInputValue, With<LocInputY>>,
    loc_z:             Single<&TextInputValue, With<LocInputZ>>,
    dim_x:             Single<&TextInputValue, With<PlaneDimXInput>>,
    dim_z:             Single<&TextInputValue, With<PlaneDimZInput>>,
    subs:              Single<&TextInputValue, With<PlaneSubsInput>>,
    mut changes:       ResMut<Changes>,
){

    let Some(x) = string_to_f32(&loc_x.0) else {return;};
    let Some(y) = string_to_f32(&loc_y.0) else {return;};
    let Some(z) = string_to_f32(&loc_z.0) else {return;};
    let Some(dim_x) = string_to_f32(&dim_x.0) else {return;};
    let Some(dim_z) = string_to_f32(&dim_z.0) else {return;};
    let Some(subs) = string_to_u32(&subs.0) else {return;};


    let loc = Vec3::new(x, y, z);

    let plane_entity = commands.spawn((
        plane_mesh(dim_x, dim_z, subs, &mut meshes),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
        Transform::from_translation(loc)
    )).id();

    if editor_settings.plane_wireframe {
        commands.entity(plane_entity).insert(Wireframe);
    }

    let cps = ChangePlaneSpawn::new(
        plane_entity, 
        dim_x, 
        dim_z, 
        subs,
        loc
    );
    cps.record(&mut changes);    
}

fn change_editor_mode(
    trigger: On<ChangeEditorMode>,
    mut editor_settings: ResMut<EditorSettings>,
    mut commands: Commands,
    controls: Query<(Entity, Option<&BrushControls>, Option<&SceneControls>, Option<&PlaneControls>), With<EditorControls>>
){
    editor_settings.mode = trigger.value;
    commands.trigger(UnghostAll);
    commands.trigger(HideVertices);
    commands.trigger(DeselectAllVertices);

    match editor_settings.mode {
        EditorMode::Scene => {
            
            for (entity, _brush, scene, _plane) in controls.iter(){
                if scene.is_some(){
                    commands.entity(entity).remove::<InteractionDisabled>();
                    // node.display = Display::Flex;
                } else {
                    // node.display = Display::None;
                    commands.entity(entity).insert(InteractionDisabled);
                }
            }
        }
        EditorMode::Brushes => {
            for (entity, brush, _scene, _plane) in controls.iter(){
                if brush.is_some(){
                    // node.display = Display::Flex;
                    commands.entity(entity).remove::<InteractionDisabled>();
                } else {
                    // node.display = Display::None;
                    commands.entity(entity).insert(InteractionDisabled);
                }
            }
        }
        EditorMode::Plane => {
            for (entity, _brush, _scene, plane) in controls.iter(){
                if plane.is_some(){
                    // node.display = Display::Flex;
                    commands.entity(entity).remove::<InteractionDisabled>();
                } else {
                    // node.display = Display::None;
                    commands.entity(entity).insert(InteractionDisabled);
                }
            }
        }
    }

}


fn turn_off_editor(
    _trigger: On<Complete<TurnOffEditor>>,
    mut next_gsp: ResMut<NextState<GameStatePlay>>
){
    info!("Action: Turn Off Editor");
    next_gsp.set(GameStatePlay::Running);
}


fn change_brush(
    trigger: On<ChangeBrush>,
    mut commands: Commands,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.brush_id = trigger.value;
    editor_settings.brush_typ = (editor_settings.brush_mapping)(&mut commands, trigger.value, &editor_settings);
}


fn toggle_nav_snap(
    trigger: On<ToggleSnapNav>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.snap_nav = trigger.value;
}

fn toggle_markers_vis(
    trigger: On<ToggleMarkersVis>,
    mut markers:  Query<&mut Visibility, With<Marker>>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.show_markers = trigger.visible;
    for mut vis in markers.iter_mut(){
        if trigger.visible {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn toggle_spawners_vis(
    trigger: On<ToggleSpawnersVis>,
    mut spawners:  Query<&mut Visibility, With<Spawner>>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.show_spawners = trigger.visible;
    for mut vis in spawners.iter_mut(){
        if trigger.visible {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}


fn toggle_plane_wireframe(
    trigger: On<TogglePlaneWireframe>,
    mut commands: Commands,
    query: Query<Entity, With<PlaneToEdit>>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.plane_wireframe = trigger.visible;

    for entity in query.iter(){
        if trigger.visible {
            commands.entity(entity).insert(Wireframe);
        } else {
            commands.entity(entity).try_remove::<Wireframe>();
        }
    }
}


fn on_fire_save_scene(
     _trigger: On<Fire<SaveScene>>,
     mut commands: Commands,
){
    commands.trigger(SaveScene);
}





fn save_scene(
    _trigger: On<SaveScene>,
    current_chunk: Res<CurrentChunk>,
    objects: Query<
        (
            &Transform,
            &Name,
            Option<&Markee>,
            Option<&Spawner>,
            Option<&Marker>,
        ),
        Or<(With<Static>, With<Ghost>, With<EditorAsset>)>,
    >, // All of it Just in case :)
) {
    info!(
        "[EDITOR] save scene for {:?} {:?}",
        current_chunk.chunk_id, current_chunk.map_name
    );

    let mut sods: HashMap<Name, Vec<SceneObjectData>> = HashMap::new();
    for (transform, name, maybe_markee, maybe_spawner, _maybe_marker) in objects.iter() {
        if maybe_markee.is_some() {
            continue;
        }
        let mut data: Option<HashMap<String,String>> = None;
        if let Some(spawner) = maybe_spawner {
            data = Some(spawner.data.clone());
        }
        // if let Some(marker) = maybe_marker {
        //     match marker.typ {
        //         _ => {}
        //     }
        // }

        let sod = SceneObjectData {
            location: transform.translation,
            rotation: transform.rotation.to_euler(EulerRot::XYZ).into(),
            scale: transform.scale,
            data,
        };
        sods.entry(name.clone()).or_insert(Vec::new()).push(sod);
    }
    let filename = format!(
        "./assets/maps/{}_{}.scene.json",
        current_chunk.map_name, current_chunk.chunk_id
    );

    info!("[EDITOR] Saving to file {}", filename);
    let sd = SceneData {
        map_name: current_chunk.map_name.clone(),
        chunk_id: current_chunk.chunk_id.clone(),
        objects: sods,
    };

    IoTaskPool::get()
        .spawn(async move {
            let f = File::create(&filename).ok().unwrap();
            let mut writer = BufWriter::new(f);
            let _res = serde_json::to_writer_pretty(&mut writer, &sd);
            let _res = writer.flush();
        })
        .detach();
}


#[derive(Component, Reflect)]
pub struct EditorController;

pub fn editor_controller() -> impl Bundle {
    return (
        EditorController,
        actions!(
            EditorController[
                (
                    Action::<TurnOffEditor>::new(),
                    Press::default(),
                    bindings![KeyCode::Escape]
                ),
                (
                    Action::<UnghostAll>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyU]
                ),
                (
                    Action::<ToggleEditorPanel>::new(),
                    Press::default(),
                    bindings![KeyCode::Space]
                ),
                (
                    Action::<ToggleAssetsPanel>::new(),
                    Press::default(),
                    bindings![KeyCode::Tab]
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
                    Action::<DeleteObject>::new(),
                    Press::default(),
                    bindings![KeyCode::BracketLeft]
                ),
            ]
        )
    );
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct SpawnPlane;

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct TriggerThumbnails;


#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct SaveScene;


#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleMarkersVis {
    pub visible: bool
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleSpawnersVis {
    pub visible: bool
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct TogglePlaneWireframe {
    pub visible: bool
}


#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ChangeBrush{
    pub value: usize
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ChangeEditorMode{
    pub value: EditorMode
}


#[derive(InputAction)]
#[action_output(bool)]
pub struct TurnOnEditor;

#[derive(InputAction)]
#[action_output(bool)]
pub struct TurnOffEditor;



// fn change_value(
//     trigger:        On<Fire<ChangeValue>>,
//     mut transforms: Query<&mut Transform, With<Ghost>>,
//     ghs:            Res<EditorSettings>,
//     navs:           Query<&PGNavmesh>
// ){

//     let delta_i32 = trigger.value as i32;
//     if delta_i32 == 0 {
//         return;
//     }
//     let d = delta_i32 as f32;

//     let mut origin: Option<Vec2> = None;

//     // Special case for OriginY and Rotation: need origin point of many transforms
//     if ghs.ghost_transform_axis == GhostTransformAxis::OriginY && ghs.ghost_transform_mode == GhostTransformMode::Rotation {
//         let count = transforms.iter().len();
//         match count {
//             0 => {}
//             1 => {
//                 origin = Some(transforms.iter().next().unwrap().translation.xz());
//             }
//             _ => {
//                 // Average:
//                 // origin = Some(transforms.iter().map(|t| t.translation.xz()).sum::<Vec2>()/count as f32);

//                 // Middle:
//                     let (min_x, max_x, min_z, max_z) = transforms.iter()
//                     .map(|t| t.translation)
//                     .fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |(min_x, max_x, min_z, max_z), pos| (
//                         min_x.min(pos.x),
//                         max_x.max(pos.x),
//                         min_z.min(pos.z),
//                         max_z.max(pos.z)));
//                     origin = Some(Vec2::new((min_x + max_x) * 0.5, (min_z + max_z) *0.5));
//             }
//         }
//     }

//     for mut transform in transforms.iter_mut(){
//         match ghs.ghost_transform_mode {
//             GhostTransformMode::Translation => {
//                 let sd = d*ghs.change_value_scale;
//                 match ghs.ghost_transform_axis {
//                     GhostTransformAxis::X => {
//                         transform.translation.x += sd;
//                         if ghs.snap_nav {

//                             for navmesh in navs.iter(){
//                                 if let Some((_poly, world_pos)) = navmesh.has_point(&transform.translation.xz()){
//                                     transform.translation.y = world_pos.y - 1.75;
//                                     break;
//                                 }
//                             }

//                         }     
//                     }
//                     GhostTransformAxis::Y => {transform.translation.y += sd}
//                     GhostTransformAxis::Z => {
//                         transform.translation.z += sd;
//                         if ghs.snap_nav {
//                             for navmesh in navs.iter(){
//                                 if let Some((_poly, world_pos)) = navmesh.has_point(&transform.translation.xz()){
//                                     transform.translation.y = world_pos.y - 1.75;
//                                     break;
//                                 }
//                             }
//                         }  
//                     }
//                     _ => {}
//                 }
//             }
//             GhostTransformMode::Rotation => {
//                 let sd = d*0.01*ghs.change_value_scale;
//                 match ghs.ghost_transform_axis {
//                     GhostTransformAxis::X => {transform.rotate_x(sd)}
//                     GhostTransformAxis::Y => {transform.rotate_y(sd)}
//                     GhostTransformAxis::Z => {transform.rotate_z(sd)}
//                     GhostTransformAxis::OriginY => {
//                         if let Some(origin) = origin {
//                             transform.translation = rotate_point_2d(&transform.translation, &origin, sd);
//                             transform.rotate_y(-sd);
//                             if ghs.snap_nav {
//                                 for navmesh in navs.iter(){
//                                     if let Some((_poly, world_pos)) = navmesh.has_point(&transform.translation.xz()){
//                                         transform.translation.y = world_pos.y - 1.75;
//                                         break;
//                                     }
//                                 }
//                             }  
//                         }
//                     }
//                     GhostTransformAxis::All => {
//                         transform.rotate_x(sd);
//                         transform.rotate_y(sd);
//                         transform.rotate_z(sd);
//                     }
//                     GhostTransformAxis::XZ => {
//                         transform.rotate_x(sd);
//                         transform.rotate_z(sd);
//                     }
//                     GhostTransformAxis::XY => {
//                         transform.rotate_x(sd);
//                         transform.rotate_y(sd);
//                     }
//                 }
//             }
//             GhostTransformMode::Scale => {
//                 let sd = d*ghs.change_value_scale;
//                 match ghs.ghost_transform_axis {
//                     GhostTransformAxis::X => {transform.scale.x += sd}
//                     GhostTransformAxis::Y => {transform.scale.y += sd}
//                     GhostTransformAxis::Z => {transform.scale.z += sd}
//                     GhostTransformAxis::XY => {
//                         transform.scale.x += sd;
//                         transform.scale.z += sd; // Its inversed somehow
//                     }
//                     GhostTransformAxis::XZ => {
//                         transform.scale.x += sd;
//                         transform.scale.y += sd; // Its inversed somehow
//                     }
//                     GhostTransformAxis::All => {
//                         transform.scale += sd;
//                     }
//                     _ => {}
//                 }
//             }

//         }
//     }
// }

#[derive(Event)]
pub struct ToggleNavmeshDebug {
    pub value: bool
}

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
pub struct SetYAxisOrigin;

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleSnapNav {
    pub value: bool
}

#[derive(InputAction)]
#[action_output(bool)]
struct DeleteObject;



#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleEditorPanel;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleAssetsPanel;


#[derive(InputAction, Event)]
#[action_output(bool)]
 pub struct UnghostAll;


fn toggle_editor_panel(
    _trigger: On<Fire<ToggleEditorPanel>>,
    mut node: Single<&mut Node, With<EditorControlsPanel>>
){
    match node.display {
        Display::None => {
            node.display = Display::Flex;
        }
        Display::Flex => {
            node.display = Display::None;
        }
        _ => {}
    }
}

fn toggle_assets_panel(
    _trigger: On<Fire<ToggleAssetsPanel>>,
    mut node: Single<&mut Node, With<EditorAssetPanel>>
){
    match node.display {
        Display::None => {
            node.display = Display::Flex;
        }
        Display::Flex => {
            node.display = Display::None;
        }
        _ => {}
    }
}


fn set_translation_mode(
    _trigger:    On<Fire<SetTranslationMode>>,
    mut transform_gizmo_settings: ResMut<TransformGizmoConfig>
){
    transform_gizmo_settings.mode = TransformGizmoMode::Translate;

}

fn set_rotation_mode(
    _trigger:    On<Fire<SetRotationMode>>,
    mut transform_gizmo_settings: ResMut<TransformGizmoConfig>
){
    transform_gizmo_settings.mode = TransformGizmoMode::Rotate;
}

fn set_scale_mode(
    _trigger:    On<Fire<SetScaleMode>>,
    mut transform_gizmo_settings: ResMut<TransformGizmoConfig>
){
    transform_gizmo_settings.mode = TransformGizmoMode::Scale;
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

fn on_fire_unghost_all(
    _trigger:     On<Fire<UnghostAll>>,
    mut commands: Commands
){
    commands.trigger(UnghostAll);
}

fn unghost_all(
    _trigger:     On<UnghostAll>,
    mut commands: Commands,
    query:        Query<Entity, With<Ghost>>
){
    for entity in query.iter(){
        commands.entity(entity).remove::<Ghost>();
        commands.entity(entity).try_remove::<TransformGizmoFocus>();
    }
}
