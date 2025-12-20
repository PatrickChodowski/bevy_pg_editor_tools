use bevy::color::palettes::css::WHITE;
use bevy::input::common_conditions::input_just_pressed;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use bevy::prelude::Press;
use std::f32::consts::FRAC_PI_2;
use bevy_pg_nav::prelude::PGNavmesh;
use bevy_pg_core::prelude::{MainCamera, GameState, GameStatePlay, AABB, PointerData};
use bevy_pg_scenes::prelude::{Spawner, Marker, AssetSource, AssignComponents, Static};


use crate::assets_panel::EditorAssetPanel;
use crate::box_select::{BoxSelect, BoxSelectFinal, box_select_changed};
use crate::settings::EditorSettings;
use crate::tracker::{Changes, Change, ChangesSet, ChangeSpawn, CurrentTransformChanges};

pub struct PGEditorGhostPlugin{
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, option: Option<String>) -> Spawner
}

#[derive(Resource)]
pub struct EditorGhostSettings {
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, option: Option<String>) -> Spawner
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
        .add_systems(OnExit(GameStatePlay::Editor), unghost_all)
        .add_systems(OnEnter(GameStatePlay::Editor), init)
        .add_systems(Update, (
            spawn_asset.run_if(on_message::<EditorSpawnAsset>),
            bs_select.run_if(box_select_changed),
            copy_ghost.run_if(input_just_pressed(MouseButton::Left))
        ).run_if(in_state(GameStatePlay::Editor)))

        .add_observer(toggle_ghost)
        .add_observer(add_ghost)
        .add_observer(remove_ghost)
        .add_observer(ghost_transform_drag_start)
        .add_observer(ghost_transform_drag)
        .add_observer(ghost_transform_drag_end)
        .add_observer(ghost_bs_selected)
        .add_observer(add_editor_asset)

        ;
    }
}

#[derive(Resource)]
pub struct GhostMaterialRef {
    handle: Handle<StandardMaterial>
}

fn init(
    mut commands: Commands,
    mut materials:  ResMut<Assets<StandardMaterial>>

){
    commands.insert_resource(GhostMaterialRef{
        handle: materials.add(
            StandardMaterial::from_color(GHOST_COLOR)
        )
    });
}

const GHOST_COLOR: Srgba = Srgba { red: 0.565, green: 0.933, blue: 0.565, alpha: 0.6 };

fn bs_select(
    mut commands: Commands,
    ghs:          Res<EditorSettings>,
    query:        Query<&BoxSelect>,
    assets:       Query<(Entity, &Transform), With<EditorAsset>>,
    mut gizmos:   Gizmos,
    ghost_marks:  Query<Entity, With<GhostMark>>,
){
    for entity in ghost_marks.iter(){
        commands.entity(entity).remove::<GhostMark>();
    }
    if ghs.multi_ghost == false {
        return;
    }

    let Ok(box_select) = query.single() else {return};
    let aabb = AABB::from_loc_dims(box_select.loc.xz(), box_select.dims);
        for (entity, transform) in assets.iter(){
            if aabb.has_point(transform.translation.xz()){
                let iso = Isometry3d{
                    translation: transform.translation.into(), 
                    rotation: Quat::from_rotation_x(FRAC_PI_2)
                };
                gizmos.circle(iso, 5.0, Color::from(WHITE));
                commands.entity(entity).insert(GhostMark);
            }
    }  
}


fn ghost_bs_selected(
    trigger:      On<BoxSelectFinal>,
    mut commands: Commands,
    ghs:          Option<Res<EditorSettings>>,
    assets:       Query<(Entity, &MeshMaterial3d<StandardMaterial>, &Transform), With<EditorAsset>>,
    ghost_marks:  Query<Entity, With<GhostMark>>,
    game_state:   Option<Res<State<GameStatePlay>>>
){
    if let Some(game_state) = game_state {
        if *game_state != GameStatePlay::Editor {
            return;
        }
    } else {
        return;
    }
    
    for entity in ghost_marks.iter(){
        commands.entity(entity).remove::<GhostMark>();
    }
    if ghs.as_ref().unwrap().multi_ghost == false {
        return;
    }
    for (entity, mat, transform) in assets.iter(){
        if trigger.has_point(transform.translation.xz()){
            commands.entity(entity).insert(Ghost{material_after: mat.0.clone()});
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum GhostTransformMode {
    Translation,
    #[default]
    Rotation,
    Scale
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum GhostTransformAxis {
    X, 
    #[default]
    Y, 
    Z,
    OriginY,
    All,
    XZ,
    XY,
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

                info!("ControlLeft Multighost copy spawn");
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
    trigger:      On<Pointer<Press>>,
    mut commands: Commands,
    query:        Query<(Entity, &MeshMaterial3d<StandardMaterial>, Option<&Ghost>), With<EditorAsset>>,
){
    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Secondary {

            if let Ok((entity, material, ghost)) = query.get(trigger.entity){
                if let Some(_ghost) = ghost {
                    commands.entity(entity).remove::<Ghost>();
                } else {
                    commands.entity(entity).insert(Ghost{material_after: material.0.clone()});
                }
            }
        }
    }
}

pub(super) fn unghost_all(
    mut commands: Commands,
    query:        Query<Entity, With<Ghost>>
){
    for entity in query.iter(){
        commands.entity(entity).remove::<Ghost>();
    }

    commands.remove_resource::<GhostMaterialRef>();
}

fn add_ghost(
    trigger:        On<Add, Ghost>,
    mut commands:   Commands,
    ghost_mat:      Res<GhostMaterialRef>,
    ghosts:         Query<Entity, With<Ghost>>,
    ghs:            Res<EditorSettings>
){
    commands.entity(trigger.entity).insert(
        MeshMaterial3d(ghost_mat.handle.clone())
    );
    
    if ghs.multi_ghost == false {
        for entity in ghosts.iter(){
            if entity != trigger.entity{
                commands.entity(entity).remove::<Ghost>(); 
            }
        }
    }

    // commands.entity(trigger.entity)
    // .observe(ghost_transform_out)
    // .observe(ghost_transform_hover)
    ;
    
}

fn remove_ghost(
    trigger:      On<Remove, Ghost>,
    query:        Query<(Entity, &Ghost)>,
    mut commands: Commands
){
    if let Ok((entity, ghost)) = query.get(trigger.entity){
        // Try as it might be being despawned in the same time
        commands.entity(entity).try_insert(MeshMaterial3d(ghost.material_after.clone())); 
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
pub struct GhostMark;


fn spawn_asset(
    mut event:      MessageReader<EditorSpawnAsset>,
    mut commands:   Commands,
    ass:            Res<AssetServer>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    pointer:        Res<PointerData>,
    mut changes:    ResMut<Changes>,
    ghost_settings: Res<EditorGhostSettings>
){
    let mut spawns = ChangesSet::new();
    for ev in event.read(){

        // info!("[EDITOR] [GHOST] Spawning {:?}", ev.asset);
        let mut translation: Option<Vec3> = None;
        let rotation: Quat;
        let scale: Vec3;

        if let Some(ev_translation) = ev.translation {
            translation = Some(ev_translation);
        } else {
            if let Some(world_pos) = pointer.center_screen_world_pos{
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
            match ev.asset {
                EditorAsset::Marker(_) => {
                    scale = Vec3::splat(1.0);
                }
                _ => {
                    scale = Vec3::splat(10.0);
                }
            }
        }



        // Only one that might be missing (based on pointer/world position)
        if let Some(translation) = translation {
            let mut transform = Transform::from_translation(translation).with_rotation(rotation).with_scale(scale);

            let entity = commands.spawn(
                editor_asset_bundle(
                    ev.asset.clone(),
                    &ass,
                    &mut meshes,
                    &mut materials,
                    &mut transform,
                    &ghost_settings
                )
            ).id();

            spawns.add(ChangeSpawn::new(entity, ev.asset.clone(), transform.clone()));



            // Special casing for marker hierarchy?
            match ev.asset {
                EditorAsset::Marker(_) => {

                }
                _ => {}
            }
        }
    }

    // Only here to not be triggered from undo/redo
    if spawns.len() > 0 {
        spawns.record(&mut changes);
    }


}

#[derive(Component, Clone, Debug)]
pub enum EditorAsset {
    Spawner(String),
    Asset(String),
    Marker(String)
}

pub(super) fn editor_asset_bundle(
    asset:      EditorAsset,
    ass:        &AssetServer,
    meshes:     &mut ResMut<Assets<Mesh>>,
    materials:  &mut ResMut<Assets<StandardMaterial>>,
    transform:  &Transform,
    settings:   &Res<EditorGhostSettings>
) -> impl Bundle {
    
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
            let spawner = (settings.spawners_mapping)(spawner_name.clone(), None);
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
        Pickable::default() // Editor only
    );

    return bundle;
}

fn ghost_transform_drag_start(
    mut trigger:    On<Pointer<DragStart>>,
    window_entity:  Single<Entity, With<PrimaryWindow>>,
    transforms:     Query<(Entity, &Transform), With<Ghost>>,
    mut commands:   Commands,
    game_state:     Option<Res<State<GameStatePlay>>>,
    hovermap:       Res<HoverMap>,
    assbuttons:     Query<&EditorAssetPanel>
){
    if let Some(game_state) = game_state {
        if *game_state != GameStatePlay::Editor {
            return;
        }
    } else {
        return;
    }

    if trigger.entity != *window_entity {
        return;
    }
    trigger.propagate(false);

    // Prevent start dragging if clicking on asset button in the same time
    let hit_data = hovermap.0.get(&PointerId::Mouse).unwrap();
    if hit_data.len() > 0 {
        let hit_entities: Vec<Entity> = hit_data.keys().cloned().collect::<Vec<Entity>>();
        for entity in hit_entities.iter(){
            if assbuttons.contains(*entity){
                return;
            }
        }
    }

    if transforms.iter().len() == 0 {
        return;
    }

    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Primary {
            let mut ctcs = CurrentTransformChanges::new();
            for (entity, transform) in transforms.iter(){
                ctcs.add(entity, transform);
            }
            commands.insert_resource(ctcs);
        }
    }
}

fn ghost_transform_drag_end(
    mut trigger:    On<Pointer<DragEnd>>,
    window_entity:  Single<Entity, With<PrimaryWindow>>,
    transforms:     Query<(Entity, &Transform), With<Ghost>>,
    mut commands:   Commands,
    changes:        Option<ResMut<Changes>>,
    ctcs:           Option<ResMut<CurrentTransformChanges>>,
    game_state:     Option<Res<State<GameStatePlay>>>
){

    if let Some(game_state) = game_state {
        if *game_state != GameStatePlay::Editor {
            return;
        }
    } else {
        return;
    }

    if trigger.entity != *window_entity {
        return;
    }
    trigger.propagate(false);

    let Some(mut ctcs) = ctcs else {return};
    let Some(mut changes) = changes else {return};


    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Primary {
            let mut cts = ChangesSet::new();
            for (entity, transform) in transforms.iter(){
                let change_transform = ctcs.get(entity);
                change_transform.new = *transform;
                if change_transform.old != change_transform.new {
                    cts.add(change_transform.clone());
                }
            }
            if cts.len() > 0 {
                cts.record(&mut changes);
            }
            commands.remove_resource::<CurrentTransformChanges>();
        }
    }
}

fn ghost_transform_drag(
    mut trigger:    On<Pointer<Drag>>,
    window_entity:  Single<Entity, With<PrimaryWindow>>,
    mut transforms: Query<&mut Transform, With<Ghost>>,
    pointer:        Res<PointerData>,
    camera:         Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    navmesh:        Option<Res<PGNavmesh>>,
    ghs:            Option<Res<EditorSettings>>,
    game_state:     Option<Res<State<GameStatePlay>>>,
    ctcs:           Option<Res<CurrentTransformChanges>>,
){

    if let Some(game_state) = game_state {
        if *game_state != GameStatePlay::Editor {
            return;
        }
    } else {
        return;
    }
    
    let Some(navmesh) = navmesh else {return};

    if trigger.entity != *window_entity {
        return;
    }

    if ctcs.is_none(){
        return;
    }

    trigger.propagate(false);

    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Primary {

            let (camera, camera_transform) = camera.into_inner();
            let Some(world_pos) = pointer.world_pos else {return};
            let Some(cursor_pos) = pointer.cursor_pos else {return};

            // let factor: f32 = 0.32;
            let factor: f32 = 1.0;
            let delta_x = trigger.delta.x*factor;
            let delta_y = trigger.delta.y*factor;

            let Ok(previous_cursor_ray) = camera.viewport_to_world(camera_transform, cursor_pos+Vec2::new(delta_x, delta_y)) else {return};

            let previous_origin = Vec3A::from(previous_cursor_ray.origin);
            let previous_direction = Vec3A::from(*previous_cursor_ray.direction);

            let Some((previous_world_pos, _dist, _index)) = navmesh.ray_intersection(
                &previous_origin,
                &previous_direction
            ) else {return};

            let world_delta = world_pos.xz() - previous_world_pos.xz();

            for mut transform in transforms.iter_mut(){
                transform.translation.x -= world_delta.x;
                transform.translation.z -= world_delta.y;
                if ghs.as_ref().unwrap().snap_nav {
                    if let Some((_poly, height)) = navmesh.get_polygon_height(transform.translation.xz()){
                        transform.translation.y = height;
                    }
                }   
            }
        }
    }
}


fn add_editor_asset(
    trigger: On<Add, EditorAsset>,
    query: Query<(&EditorAsset, Option<&Spawner>, Option<&Marker>)>,
    ghost_settings: Res<EditorGhostSettings>,
    mut commands: Commands
){
    let entity = trigger.entity;
    let Ok((asset, maybe_spawner, maybe_marker)) = query.get(entity) else {return};
    match asset {
        EditorAsset::Asset(asset_name) => {
            let asset_path = format!("objects/{}.glb", asset_name.clone());
            commands.entity(entity).insert((AssetSource::new_mm(asset_path.clone()), Static));
        }
        EditorAsset::Spawner(spawner_name) => {
            if let Some(_spawner) = maybe_spawner {} else{
                // Insert generic spawner only if there is no spawner component yet;
                commands.entity(entity).insert((ghost_settings.spawners_mapping)(spawner_name.clone(), None));
            }
        }
        EditorAsset::Marker(marker_name) => {
            if let Some(_marker) = maybe_marker {} else{
                // Insert generic marker only if there is no spawner component yet;
                commands.entity(entity).insert((ghost_settings.markers_mapping)(marker_name.clone()));
            }
        }
    }
}


// fn ghost_transform_hover(
//     trigger:         On<Pointer<Move>>,
//     mut query:       Query<(&mut Text, &TextSection)>,
//     objs:            Query<(&Transform, &Name)> 
// ){
//     for (mut txt, section) in query.iter_mut(){
//         match section {
//             TextSection::Hover => {
//             let Ok((transform, name)) = objs.get(trigger.entity) else {return};
//             txt.0 = format!("{}: ({:.0}, {:.0}, {:.0})", 
//                                 name, 
//                                 transform.translation.x, 
//                                 transform.translation.y, 
//                                 transform.translation.z);
//             }
//             _ => {}
//         }
//     }
// }

// fn ghost_transform_out(
//     _trigger:         On<Pointer<Out>>,
//     mut query:       Query<(&mut Text, &TextSection)>,
// ){
//     for (mut txt, section) in query.iter_mut(){
//         match section {
//             TextSection::Hover => {
//                 txt.0 = format!("");
//             }
//             _ => {}
//         }
//     }
// }