use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::platform::collections::HashMap;
use bevy::ui::InteractionDisabled;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::prelude::Press;
use bevy_pg_core::prelude::{GameStatePlay, rotate_point_2d};
use bevy_pg_nav::prelude::{GenerateNavMesh, NavMesh};
use bevy_pg_scenes::prelude::{TerrainChunk, CurrentChunk, MapsData, SceneData, SceneObjectData, Markee, Spawner, Marker, Static};
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::tracker::{Changes, Change, Undo, Redo, ChangesSet, ChangeDespawn, ChangeTransform, CurrentTransformChanges};
use crate::ghost::{EditorAsset, Ghost, GhostTransformAxis, GhostTransformMode};
use crate::planes::PlaneToEdit;
use crate::ui::{BrushControls, EditorControlsPanel, PlaneControls, SceneControls, EditorControls};
use crate::settings::{EditorMode, EditorSettings};
use crate::vertex::PlaneVertex;

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


        .add_observer(change_value)
        .add_observer(start_change_value)
        .add_observer(end_change_value)
        .add_observer(unghost_all)
        .add_observer(on_fire_unghost_all)
        .add_observer(on_fire_save_scene)
        .add_observer(save_scene)
        .add_observer(delete_object)
        .add_observer(navmesh_generation)

        .add_observer(toggle_markers_vis)
        .add_observer(toggle_spawners_vis)
        .add_observer(toggle_ghost_axis)
        .add_observer(toggle_ghost_mode)
        .add_observer(toggle_nav_snap)
        .add_observer(toggle_multi_ghost)
        .add_observer(change_brush)
        .add_observer(turn_off_editor)
        .add_observer(toggle_settings)
        .add_observer(change_editor_mode)
        ;
    }
}

fn change_editor_mode(
    trigger: On<ChangeEditorMode>,
    mut editor_settings: ResMut<EditorSettings>,
    mut commands: Commands,
    controls: Query<(Entity, Option<&BrushControls>, Option<&SceneControls>, Option<&PlaneControls>), With<EditorControls>>
){
    editor_settings.mode = trigger.value;
    
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
    _trigger: On<Fire<TurnOffEditor>>,
    mut next_gsp: ResMut<NextState<GameStatePlay>>
){
    next_gsp.set(GameStatePlay::Running);
}


fn change_brush(
    trigger: On<ChangeBrush>,
    mut commands: Commands,
    mut editor_settings: ResMut<EditorSettings>, 
    terrain_chunks: Query<Entity, (With<TerrainChunk>, With<PlaneToEdit>)>,
    vertices: Query<Entity, With<PlaneVertex>>,
){
    for entity in vertices.iter(){
        commands.entity(entity).despawn();
    }
    editor_settings.brush_id = trigger.value;
    editor_settings.brush_typ = (editor_settings.brush_mapping)(&mut commands, &terrain_chunks, trigger.value);
}


fn toggle_nav_snap(
    trigger: On<ToggleSnapNav>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.snap_nav = trigger.value;
}

fn toggle_multi_ghost(
    trigger: On<ToggleMultiGhost>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.multi_ghost = trigger.value;
}


fn toggle_ghost_axis(
    trigger: On<ToggleGhostAxis>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.ghost_transform_axis = trigger.value;
}

fn toggle_ghost_mode(
    trigger: On<ToggleGhostMode>,
    mut editor_settings: ResMut<EditorSettings>
){
    editor_settings.ghost_transform_mode = trigger.value;
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
        let mut option: Option<String> = None;
        if let Some(spawner) = maybe_spawner {
            option = spawner.option.clone();
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
            option,
        };
        sods.entry(name.clone()).or_insert(Vec::new()).push(sod);
    }
    let filename = format!(
        "./assets/maps/{}/{}_{}.scene.json",
        current_chunk.map_name, current_chunk.map_name, current_chunk.chunk_id
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
                    Action::<TurnOffEditor>::new(),
                    HoldAndRelease::new(0.3),
                    bindings![KeyCode::Escape]
                ),
                (
                    Action::<UnghostAll>::new(),
                    Press::default(),
                    bindings![KeyCode::KeyU]
                ),
                (
                    Action::<ToggleEditorSettings>::new(),
                    Press::default(),
                    bindings![KeyCode::Tab]
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
pub struct ToggleGhostAxis {
    pub value: GhostTransformAxis
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleGhostMode {
    pub value: GhostTransformMode
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
    if ghs.ghost_transform_axis == GhostTransformAxis::OriginY && ghs.ghost_transform_mode == GhostTransformMode::Rotation {
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
        match ghs.ghost_transform_mode {
            GhostTransformMode::Translation => {
                let sd = d*ghs.change_value_scale;
                match ghs.ghost_transform_axis {
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
                match ghs.ghost_transform_axis {
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
                match ghs.ghost_transform_axis {
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
pub struct SetYAxisOrigin;

#[derive(InputAction)]
#[action_output(bool)]
pub struct AllAxis;

#[derive(InputAction)]
#[action_output(bool)]
pub struct XZ;

#[derive(InputAction)]
#[action_output(bool)]
pub struct XY;

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleSnapNav {
    pub value: bool
}

#[derive(InputAction)]
#[action_output(bool)]
struct DeleteObject;


#[derive(Event)]
pub struct NavMeshGeneration;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleEditorSettings;


#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct ToggleMultiGhost {
    pub value: bool
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct UnghostAll;


fn toggle_settings(
    _trigger: On<Fire<ToggleEditorSettings>>,
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

fn set_translation_mode(
    _trigger:    On<Fire<SetTranslationMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_mode = GhostTransformMode::Translation;
}

fn set_rotation_mode(
    _trigger:    On<Fire<SetRotationMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_mode = GhostTransformMode::Rotation;
}

fn set_scale_mode(
    _trigger:    On<Fire<SetScaleMode>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_mode = GhostTransformMode::Scale;
}

fn set_x_axis(
    _trigger:    On<Fire<SetXAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::X;
}

fn set_y_axis(
    _trigger:    On<Fire<SetYAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::Y;
}

fn set_all_axis(
    _trigger:    On<Fire<AllAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::All;
}

fn set_xz_axis(
    _trigger:    On<Fire<XZ>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::XZ;
}

fn set_xy_axis(
    _trigger:    On<Fire<XY>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::XY;
}

fn set_y_axis_origin(
    _trigger:    On<Fire<SetYAxisOrigin>>,
    mut ghost_settings: ResMut<EditorSettings> 
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::OriginY;
}

fn set_z_axis(
    _trigger:    On<Fire<SetZAxis>>,
    mut ghost_settings: ResMut<EditorSettings>
){
    ghost_settings.ghost_transform_axis = GhostTransformAxis::Z;
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
    _trigger:       On<NavMeshGeneration>,
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
    }
}
