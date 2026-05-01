use bevy::platform::collections::HashMap;
use bevy::input::common_conditions::input_just_pressed;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::prelude::Press;
use bevy::tasks::IoTaskPool;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufWriter, Write};

use std::f32::consts::FRAC_PI_2;
use bevy_pg_core::prelude::{GameState, GameStatePlay, AABB, WaterChunk};
use bevy_pg_scenes::prelude::{Spawner, Marker, AssetSource, AssignComponents, SceneObjectData, SceneData, Markee, PlaneToEdit, Static, WaterData, spawn_water, water_mm};

use crate::prelude::{EditorMode, EditorSettings};
use crate::editor_pointer::EditorPointer;
use crate::tracker::{Changes, Change, ChangesSet, ChangeSpawn};
use crate::transform_gizmo::{TransformGizmoFocus, TransformGizmoState};

pub struct PGEditorGhostPlugin{
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, maybe_data: &Option<HashMap<String, String>>) -> Spawner
}

#[derive(Resource)]
pub struct EditorGhostSettings {
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, maybe_data: &Option<HashMap<String, String>>) -> Spawner
}


impl Plugin for PGEditorGhostPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(EditorGhostSettings{
            spawner_mesh: self.spawner_mesh,
            marker_mesh: self.marker_mesh,
            markers_mapping: self.markers_mapping,
            spawners_mapping: self.spawners_mapping
        })
        .add_message::<EditorSpawnAsset>()
        .add_systems(OnExit(GameStatePlay::Editor), clear)
        .add_systems(OnEnter(GameStatePlay::Editor), init)
        .add_systems(Update, (
            spawn_asset.run_if(on_message::<EditorSpawnAsset>),
            copy_ghost.run_if(input_just_pressed(MouseButton::Left))
        ).run_if(in_state(GameStatePlay::Editor)))

        .add_observer(toggle_ghost)
        .add_observer(add_ghost)
        .add_observer(add_ghost_mark)
        .add_observer(remove_ghost)
        .add_observer(remove_ghostmark)
        .add_observer(add_editor_asset)
        .add_observer(save_scene)

        ;
    }
}

#[derive(Resource)]
pub struct GhostMaterials {
    handle: Handle<StandardMaterial>,
    mark_handle: Handle<StandardMaterial>,
}

const GHOST_COLOR: Srgba = Srgba { red: 0.565, green: 0.933, blue: 0.565, alpha: 0.65 };
const GHOSTMARK_COLOR: Srgba = Srgba { red: 0.565, green: 0.733, blue: 0.565, alpha: 0.55 };

fn init(
    mut commands:   Commands,
    mut materials:  ResMut<Assets<StandardMaterial>>
){
    commands.insert_resource(GhostMaterials{
        handle: materials.add(
            StandardMaterial::from_color(GHOST_COLOR)
        ),
        mark_handle: materials.add(
            StandardMaterial::from_color(GHOSTMARK_COLOR)
        )
    });
}




fn copy_ghost(
    mut commands: Commands,
    query:        Query<(&EditorAsset, &Transform)>,
    ghosts:       Query<(Entity, &EditorAsset, &Transform), With<Ghost>>,
    keys:         Res<ButtonInput<KeyCode>>,
    hovermap:     Res<HoverMap>
){

    for key in keys.get_pressed() {
        match key {
            KeyCode::ControlLeft => {

                info!("ControlLeft copy spawn");
                let offset_x: f32 = 10.0;
                let offset_z: f32 = 10.0;
                // TODO: maybe.. improve spawn position logic instead of just offset
                for (entity, asset, transform) in ghosts.iter(){

                    let mut loc = transform.translation;
                    loc.x += offset_x;
                    loc.z += offset_z;

                    // Remove ghost from original
                    commands.entity(entity).remove::<Ghost>();

                    commands.write_message(
                    EditorSpawnAsset::new(
                        asset.clone(),
                         Some(loc), 
                         Some(transform.rotation), 
                         Some(transform.scale)
                    ));
                }
                
            }
            KeyCode::KeyT => {
                let hit_data = hovermap.0.get(&PointerId::Mouse).unwrap();
                if hit_data.len() > 0 {
                    let hit_entities: Vec<Entity> = hit_data.keys().cloned().collect::<Vec<Entity>>();
                    for entity in hit_entities.iter(){
                        if let Ok((_asset, transform)) = query.get(*entity){
                            info!("Copying transform of {}", entity);
                            commands.insert_resource(
                                EditorGhostTransformMemory::new(transform.rotation, transform.scale)
                            );
                            break
                        } 
                    }
                }
            }
            _ => {}
        }
    }
}

fn toggle_ghost(
    trigger:            On<Pointer<Press>>,
    mut commands:       Commands,
    query:              Query<(Entity, &MeshMaterial3d<StandardMaterial>, Option<&Ghost>, Option<&TransformGizmoFocus>)>,
    assets:             Query<&Static>,
    planes:             Query<&PlaneToEdit>,
    focus:              Query<Entity, With<TransformGizmoFocus>>,
    ghosts:             Query<Entity, With<Ghost>>,
    gizmo_state:        Res<TransformGizmoState>,
    state:              Res<State<GameStatePlay>>,
    keys:               Res<ButtonInput<KeyCode>>,
    editor_settings:    Res<EditorSettings>
){

    if *state != GameStatePlay::Editor {
        return;
    }

    let mut multi_ghost: bool = false;
    let mut remove_ghost: bool = false;
    let mut unghost_all: bool = false;
    for key in keys.get_pressed() {
        match key {
            KeyCode::ShiftLeft => {
                multi_ghost = true;
            }
            KeyCode::KeyR => {
                remove_ghost = true;
            }
            KeyCode::KeyU => {
                unghost_all = true;
            }
            _ => {}
        }
    }


    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Primary {

            if unghost_all {
                for focus_entity in focus.iter(){
                    commands.entity(focus_entity).remove::<TransformGizmoFocus>();
                }
                for ghost_entity in ghosts.iter(){
                    commands.entity(ghost_entity).remove::<Ghost>();
                } 
            }

            if let Some(_hovered_axis) = gizmo_state.hovered_axis {
                if remove_ghost {
                    commands.entity(trigger.entity).try_remove::<Ghost>();
                    commands.entity(trigger.entity).try_remove::<TransformGizmoFocus>();
                }
            } else {

                let maybe_data: Option<(Entity, &MeshMaterial3d<StandardMaterial>, Option<&Ghost>, Option<&TransformGizmoFocus>)> = match editor_settings.mode {
                    EditorMode::Plane => {
                        if let Ok(_) = planes.get(trigger.entity){
                            if let Ok(data) = query.get(trigger.entity){
                                Some(data)
                            } else {
                                None
                            }
                        } else {None}
                    }
                    EditorMode::Scene => {
                        if let Ok(_) = assets.get(trigger.entity){
                            if let Ok(data) = query.get(trigger.entity){
                                Some(data)
                            } else {
                                None
                            }
                        } else {None}

                    }
                    _ => {None}
                };

                if let Some((entity, material, maybe_ghost, maybe_gizmo)) = maybe_data {
                    match (maybe_ghost, remove_ghost, multi_ghost) {

                        (None, false, false) => {
                            for focus_entity in focus.iter(){
                                commands.entity(focus_entity).remove::<TransformGizmoFocus>();
                            }
                            for ghost_entity in ghosts.iter(){
                                commands.entity(ghost_entity).remove::<Ghost>();
                            }
                            commands.entity(entity).insert(Ghost{material_after: material.0.clone()});
                            commands.entity(entity).insert(TransformGizmoFocus);
                        }

                        (None, false, true) => {
                            for focus_entity in focus.iter(){
                                commands.entity(focus_entity).remove::<TransformGizmoFocus>();
                            }
                            commands.entity(entity).insert(Ghost{material_after: material.0.clone()});
                            commands.entity(entity).insert(TransformGizmoFocus);
                        }

                        (None, true, _) => {
                            // do nothing on remove if they dont have anything
                        }
                        (Some(_), false, _) => {
                            if maybe_gizmo.is_none(){
                                for focus_entity in focus.iter(){
                                    commands.entity(focus_entity).remove::<TransformGizmoFocus>();
                                }
                                commands.entity(entity).insert(TransformGizmoFocus);
                            }
                        }
                        (Some(_), true, _) => {
                            commands.entity(entity).remove::<TransformGizmoFocus>();
                            commands.entity(entity).remove::<Ghost>();
                        }

                    }
                }

            }

        }
    }
}

pub(super) fn clear(
    mut commands: Commands,
    query:        Query<Entity, With<Ghost>>
){
    for entity in query.iter(){
        commands.entity(entity).remove::<Ghost>();
        commands.entity(entity).try_remove::<TransformGizmoFocus>();
    }

    commands.remove_resource::<GhostMaterials>();
}

fn add_ghost(
    trigger:        On<Add, Ghost>,
    mut query:      Query<&mut MeshMaterial3d<StandardMaterial>, With<Ghost>>,
    ghost_mat:      Res<GhostMaterials>
){
    if let Ok(mut material) = query.get_mut(trigger.entity){
        material.0 = ghost_mat.handle.clone();
    }  
}

fn add_ghost_mark(
    trigger:        On<Add, GhostMark>,
    mut query:      Query<&mut MeshMaterial3d<StandardMaterial>, With<GhostMark>>,
    ghost_mat:      Res<GhostMaterials>
){
    if let Ok(mut material) = query.get_mut(trigger.entity){
        material.0 = ghost_mat.mark_handle.clone();
    }  
}

fn remove_ghostmark(
    trigger:      On<Remove, GhostMark>,
    mut query:    Query<(&mut MeshMaterial3d<StandardMaterial>, &GhostMark)>,
){
    if let Ok((mut material, ghostmark)) = query.get_mut(trigger.entity){
        material.0 = ghostmark.material_after.clone();
    }
}

fn remove_ghost(
    trigger:      On<Remove, Ghost>,
    mut query:    Query<(&mut MeshMaterial3d<StandardMaterial>, &Ghost)>,
){
    if let Ok((mut material, ghost)) = query.get_mut(trigger.entity){
        material.0 = ghost.material_after.clone();
    }
}

#[derive(Resource, Debug)]
pub struct EditorGhostTransformMemory{
    pub rotation: Quat,
    pub scale:    Vec3
}
impl EditorGhostTransformMemory {
    pub fn new(
        rotation: Quat, 
        scale:    Vec3
    ) -> Self {
        Self {
            rotation,
            scale
        }
    }
}



#[derive(Message)]
pub(super) struct EditorSpawnAsset{
    pub(super) asset:      EditorAsset,
    pub(super) translation: Option<Vec3>,
    pub(super) rotation:    Option<Quat>,
    pub(super) scale:       Option<Vec3>
}

impl EditorSpawnAsset {
    pub(super) fn new(
        asset:       EditorAsset, 
        translation: Option<Vec3>,
        rotation:    Option<Quat>,
        scale:       Option<Vec3>

    ) -> Self {
        info!("Spawning new Asset {:?}", asset);
        Self {
            asset,
            translation,
            rotation,
            scale
        }
    }
}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Ghost {
    pub(super) material_after: Handle<StandardMaterial>
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct GhostMark {
    pub(super) material_after: Handle<StandardMaterial>
}


fn spawn_asset(
    mut event:      MessageReader<EditorSpawnAsset>,
    mut commands:   Commands,
    ass:            Res<AssetServer>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    pointer:        Res<EditorPointer>,
    mut changes:    ResMut<Changes>,
    ghost_settings: Res<EditorGhostSettings>,
    water_data:     Res<WaterData>
){
    let mut spawns = ChangesSet::new();
    for ev in event.read(){
        let mut translation: Option<Vec3> = None;
        let rotation: Quat;
        let scale: Vec3;

        if let Some(ev_translation) = ev.translation {
            translation = Some(ev_translation);
        } else {
            if let Some(world_pos) = pointer.center_screen_plane_pos{
                translation = Some(world_pos);
            }
        }

        if let Some(ev_rotation) = ev.rotation {
            rotation = ev_rotation;
        } else {
            match ev.asset {
                EditorAsset::Marker(_) => {
                    rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
                }
                _ => {
                    rotation = Quat::from_euler(EulerRot::XYZ, -FRAC_PI_2, 0.0, 0.0);
                }
            }
        }

        if let Some(ev_scale) = ev.scale {
            scale = ev_scale;
        } else {
            scale = Vec3::splat(1.0);
        }

        // Only one that might be missing (based on pointer/world position)
        if let Some(translation) = translation {
            let mut transform = Transform::from_translation(translation).with_rotation(rotation).with_scale(scale);
            
            let entity: Entity = match ev.asset {
                EditorAsset::Water => {
                    let water_entity = spawn_water(&mut commands, &mut meshes, &mut materials, water_data.material.clone(), &translation, &Vec2::splat(1.0), true);
                    commands.entity(water_entity).insert(ev.asset.clone());
                    water_entity
                }
                _ => {
                    let entity = commands.spawn(
                        editor_asset_bundle(
                            ev.asset.clone(),
                            &ass,
                            &mut meshes,
                            &mut materials,
                            &mut transform,
                            &ghost_settings
                        ).unwrap()
                    ).id();
                    entity

                }
            };

            spawns.add(ChangeSpawn::new(entity, ev.asset.clone(), transform.clone()));
        }
    }

    // Only here to not be triggered from undo/redo
    if spawns.len() > 0 {
        spawns.record(&mut changes);
    }
}



#[derive(Event)]
pub struct SaveScene {
    pub plane_entity: Entity
}


fn save_scene(
    trigger: On<SaveScene>,
    planes:  Query<(&PlaneToEdit, &Transform, Option<&Name>)>,
    objects: Query<
        (
            &Transform,
            &Name,
            Option<&Markee>,
            Option<&Spawner>,
            Option<&Marker>,
            Option<&WaterChunk>
        ),
        Or<(With<Static>, With<Ghost>, With<EditorAsset>)>,
    >, // All of it Just in case :)
) {
    let Ok((plane, plane_transform, maybe_name)) = planes.get(trigger.plane_entity) else {return};
    let Some(name) = maybe_name else {warn!("Plane should have a name before serializing scene. Abort"); return};

    info!(
        "[EDITOR] save scene for plane {}", trigger.plane_entity
    );


    let plane_aabb = AABB::from_loc_dims(plane_transform.translation.xz(), Vec2::new(plane.width, plane.height));

    let mut sods: HashMap<Name, Vec<SceneObjectData>> = HashMap::new();

    for (transform, name, maybe_markee, maybe_spawner, _maybe_marker, maybe_water) in objects.iter() {
        if maybe_markee.is_some() {
            continue;
        }

        // Add only objects belonging to the plane
        if !plane_aabb.has_point(transform.translation.xz()){
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

        let mut scale = transform.scale;
        if let Some(water) = maybe_water {
            scale = Vec3::new(water.dims.x, 1.0, water.dims.y); 
        }

        let sod = SceneObjectData {
            location: transform.translation - plane_transform.translation, // Adjust to origin (plane translation is an origin for a chunk)
            rotation: transform.rotation.to_euler(EulerRot::XYZ).into(),
            scale: scale,
            data,
        };
        sods.entry(name.clone()).or_insert(Vec::new()).push(sod);
    }
    let filename = format!("./assets/scenes/scenes/{}.scene.json",name);

    info!("[EDITOR] Saving to file {}", filename);
    let sd = SceneData {
        saved_loc: plane_transform.translation,
        objects: sods
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


#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub enum EditorAsset {
    Asset(String),
    Spawner (String),
    Marker(String),
    Water
}

pub(super) fn editor_asset_bundle(
    asset:      EditorAsset,
    ass:        &AssetServer,
    meshes:     &mut ResMut<Assets<Mesh>>,
    materials:  &mut ResMut<Assets<StandardMaterial>>,
    transform:  &Transform,
    settings:   &Res<EditorGhostSettings>
) -> Result<impl Bundle> {
    
    let ghost_material: Handle<StandardMaterial>;
    let mesh: Handle<Mesh>;
    let material: Handle<StandardMaterial>;
    let name: String;

    match asset.clone() {
        EditorAsset::Asset(asset_name) => {
            let asset_path = format!("objects/{}.glb", asset_name);
            ghost_material = materials.add(
                StandardMaterial::from_color(GHOST_COLOR)
            );
            mesh = ass.load(
                GltfAssetLabel::Primitive{primitive:0, mesh:0}.from_asset(asset_path.clone()),
            );
            material = ass.load(
                GltfAssetLabel::Material { index: 0, is_scale_inverted: false}.from_asset(asset_path.clone())
            );
            name = asset_name;
        }

        EditorAsset::Spawner(spawner_name) => {
            let spawner = (settings.spawners_mapping)(spawner_name.clone(), &None);
            let (spawner_mesh, spawner_mat) = (settings.spawner_mesh)(spawner.id, meshes, materials);
            mesh = spawner_mesh;
            material = spawner_mat;
            ghost_material = material.clone();
            name = spawner_name
        }

        EditorAsset::Marker(marker_name) => {
            let marker = (settings.markers_mapping)(marker_name.clone());
            let (marker_mesh, marker_mat) = (settings.marker_mesh)(marker.id, meshes, materials);
            mesh = marker_mesh;
            material = marker_mat;
            ghost_material = material.clone();
            name = marker_name
        }
        _ => {
            return Err("Wrong wrong wrong".into());
            /* Water handled in other function */
        }
    }

    let bundle = (
        transform.clone(),
        Mesh3d(mesh), 
        Ghost{material_after: material},
        MeshMaterial3d(ghost_material),
        Name::from(name),
        AssignComponents,
        asset.clone(),
        DespawnOnExit(GameState::Play),
        Static,
        Pickable::default() // Editor only
    );

    return Ok(bundle);
}

fn add_editor_asset(
    trigger: On<Add, EditorAsset>,
    query: Query<(&EditorAsset, Option<&Spawner>, Option<&Marker>, Option<&WaterChunk>)>,
    ghost_settings: Res<EditorGhostSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    water_data: Res<WaterData>
){
    let entity = trigger.entity;
    let Ok((asset, maybe_spawner, maybe_marker, maybe_water)) = query.get(entity) else {return};
    match asset {
        EditorAsset::Asset(asset_name) => {
            let asset_path = format!("objects/{}.glb", asset_name.clone());
            commands.entity(entity).insert((AssetSource::new_mm(asset_path.clone()), Static));
        }
        EditorAsset::Spawner(spawner_name) => {
            if let Some(_spawner) = maybe_spawner {} else{
                commands.entity(entity).insert((ghost_settings.spawners_mapping)(spawner_name.clone(), &None));
            }
        }
        EditorAsset::Marker(marker_name) => {
            if let Some(_marker) = maybe_marker {} else{
                commands.entity(entity).insert((ghost_settings.markers_mapping)(marker_name.clone()));
            }
        }
        EditorAsset::Water => {
            if let Some(water) = maybe_water {
                water_mm(entity, &mut commands, &mut meshes, &mut materials, water_data.material.clone(), &water.dims, true);
            }
        }
    }
}
