use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use bevy_pg_scenes::prelude::{Spawner, Marker, Markee, Spawnee, Static, PlaneToEdit};
use bevy_pg_core::prelude::{GameStatePlay, MainCamera, Player, TerrainChunk};
use bevy_enhanced_input::prelude::ContextActivity;
use bevy::pbr::wireframe::WireframePlugin;

pub mod assets_panel;
pub mod box_select;
pub mod brushes;
pub mod controller;
pub mod ghost;
pub mod tracker;
pub mod plane_loader;
pub mod export_scene_obj;
pub mod editor_pointer;
pub mod thumbnails;
pub mod noises;
pub mod planes;
pub mod vertex;
pub mod ui;
pub mod settings;
pub mod terrain_brushes;
pub mod text_inputs;
pub mod transform_gizmo_render;
pub mod transform_gizmo;

use assets_panel::PGEditorAssetsPanelPlugin;
use brushes::{PGEditorBrushSelectPlugin};
use box_select::PGEditorBoxSelectPlugin;
use controller::{PGEditorControllerPlugin, EditorController};
use editor_pointer::PGEditorPointer;
use ghost::{PGEditorGhostPlugin, EditorGhostSettings, EditorAsset, EditorGhostTransformMemory};
use planes::PGEditorPlanesPlugin;
use plane_loader::PGEditorLoadPlanePlugin;
use settings::EditorSettings;
use thumbnails::PGEditorThumbnailsPlugin;
use text_inputs::PGEditorTextInputs;
use tracker::{PGEditorTrackerPlugin, CurrentTransformChanges, Changes};
use transform_gizmo::TransformGizmoPlugin;
use transform_gizmo_render::TransformGizmoRenderPlugin;
use ui::PGEditorUIPlugin;
use vertex::PGEditorVertexPlugin;

use crate::{brushes::BrushType, export_scene_obj::export_obj_system};


pub struct PGEditorPlugin{
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, maybe_data: &Option<HashMap<String, String>>) -> Spawner,
    pub brush_mapping: fn(commands: &mut Commands, brush_id: usize, editor_settings: &ResMut<EditorSettings>) -> Box<dyn BrushType>,
    pub brush_id_labels: Vec<(usize, &'static str)>
}

impl Plugin for PGEditorPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(
            (
                WireframePlugin::default(),
                PGEditorPointer,
                PGEditorTrackerPlugin,
                PGEditorBrushSelectPlugin,
                PGEditorBoxSelectPlugin,
                PGEditorTextInputs,
                PGEditorAssetsPanelPlugin,
                PGEditorThumbnailsPlugin,
                PGEditorControllerPlugin,
                TransformGizmoPlugin,
                TransformGizmoRenderPlugin,
                PGEditorVertexPlugin,
                PGEditorGhostPlugin{
                    spawner_mesh: self.spawner_mesh,
                    marker_mesh: self.marker_mesh,
                    markers_mapping: self.markers_mapping,
                    spawners_mapping: self.spawners_mapping
                },
                PGEditorPlanesPlugin,
                PGEditorUIPlugin
            )
        )
        .add_plugins(PGEditorLoadPlanePlugin)
        .insert_resource(EditorSettings::new(
            self.brush_mapping,
            self.brush_id_labels.clone()
        ))
        .add_plugins(MeshPickingPlugin::default())
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            // When set to true ray casting will only consider cameras marked with MeshPickingCamera and entities marked with Pickable. false by default.
            ..default()
        })
        .add_systems(OnEnter(GameStatePlay::Editor), init_editor)
        .add_systems(OnExit(GameStatePlay::Editor), exit_editor)
        // .add_systems(Update, export_obj_system.run_if(input_just_pressed(KeyCode::Digit6)))
        ;
    }
}


fn init_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    statics: Query<
        (Entity, &Name),
        (
            With<Static>,
            Without<Spawner>,
            Without<Marker>,
            Without<Markee>,
        ),
    >,
    editor: Query<Entity, With<EditorController>>,
    mut camera: Query<(Entity, &mut MainCamera)>,
    mut spawnees: Query<&mut Visibility, With<Spawnee>>,
    spawners: Query<(Entity, &Spawner, &Name)>,
    markers: Query<(Entity, &Marker, &Name)>,
    terrains: Query<Entity, With<TerrainChunk>>,
    ghost_settings: Res<EditorGhostSettings>,
    editor_settings: Res<EditorSettings>
) {
    info!("[EDITOR] Entering Editor");
    for terrain_entity in terrains.iter() {
        commands.entity(terrain_entity).insert(PlaneToEdit::dummy());
    }

    let Ok((camera_entity, mut camera_data)) = camera.single_mut() else {
        return;
    };
    commands.entity(camera_entity).insert(MeshPickingCamera);
    camera_data.set_dev(&mut commands, camera_entity);

    let Ok(editor_entity) = editor.single() else {
        return;
    };
    commands
        .entity(editor_entity)
        .insert(ContextActivity::<EditorController>::ACTIVE);

    for (entity, name) in statics.iter() {
        commands
            .entity(entity)
            .insert((Pickable::default(), EditorAsset::Asset(name.to_string())));
    }

    commands.insert_resource(Changes::new());

    for mut vis in spawnees.iter_mut() {
        *vis = Visibility::Hidden;
    }

    let mut spawner_visibility = Visibility::Visible;
    if !editor_settings.show_spawners {
        spawner_visibility = Visibility::Hidden;
    }

    let mut marker_visibility = Visibility::Visible;
    if !editor_settings.show_markers {
        marker_visibility = Visibility::Hidden;
    }

    for (spawner_entity, spawner, name) in spawners.iter() {
        let (mesh, mat) = (ghost_settings.spawner_mesh)(spawner.id, &mut meshes, &mut materials);
        commands.entity(spawner_entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Pickable::default(),
            EditorAsset::Spawner(name.to_string()),
            spawner_visibility
        ));
    }

    for (marker_entity, marker, name) in markers.iter() {
        let (mesh, mat) = (ghost_settings.marker_mesh)(marker.id, &mut meshes, &mut materials);
        commands.entity(marker_entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Pickable::default(),
            EditorAsset::Marker(name.to_string()),
            marker_visibility
        ));
    }

    for entity in terrains.iter() {
        commands.entity(entity).insert(PlaneToEdit::dummy());
    };
    
}

fn exit_editor(
    mut commands: Commands,
    statics: Query<Entity, With<Static>>,
    mut camera: Query<(Entity, &mut Transform, &mut MainCamera), Without<Player>>,
    editor: Query<Entity, With<EditorController>>,
    mut spawnees: Query<&mut Visibility, With<Spawnee>>,
    spawners_markers: Query<Entity, Or<(With<Spawner>, With<Marker>)>>,
    player: Query<&Transform, With<Player>>,
    terrains: Query<Entity, With<TerrainChunk>>,
) {
    info!("[EDITOR] Exit Editor");

    for terrain_entity in terrains.iter() {
        commands.entity(terrain_entity).remove::<PlaneToEdit>();
    }
    let Ok((camera_entity, mut camera_transform, mut camera_data)) = camera.single_mut() else {
        return;
    };
    commands.entity(camera_entity).remove::<MeshPickingCamera>();
    let Ok(player_transform) = player.single() else {
        return;
    };
    camera_data.set_player(
        &mut commands,
        camera_entity,
        &mut camera_transform,
        player_transform.translation,
    );

    let Ok(editor_entity) = editor.single() else {
        return;
    };
    commands
        .entity(editor_entity)
        .insert(ContextActivity::<EditorController>::INACTIVE);

    for entity in statics.iter() {
        commands.entity(entity).remove::<Pickable>();
        commands.entity(entity).remove::<EditorAsset>();
    }

    commands.remove_resource::<Changes>();
    commands.remove_resource::<CurrentTransformChanges>();
    commands.remove_resource::<EditorGhostTransformMemory>();

    for mut vis in spawnees.iter_mut() {
        *vis = Visibility::Visible;
    }

    for spawner_marker_entity in spawners_markers.iter() {
        commands
            .entity(spawner_marker_entity)
            .remove::<Visibility>();
        commands.entity(spawner_marker_entity).remove::<Mesh3d>();
        commands
            .entity(spawner_marker_entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
        commands.entity(spawner_marker_entity).remove::<Pickable>();
        commands
            .entity(spawner_marker_entity)
            .remove::<EditorAsset>();
    }
}


pub mod prelude {
    pub use crate::assets_panel::PGEditorAssetsPanelPlugin;
    pub use crate::box_select::{box_select_changed, BoxSelectFinal, BoxSelect, PGEditorBoxSelectPlugin};
    pub use crate::brushes::{
        brush_changed, BrushDone, BrushStart,
        Brush, PGEditorBrushSelectPlugin, BrushType, ScatterBrush, NothingBrush
    };
    pub use crate::controller::{
        PGEditorControllerPlugin, editor_controller, EditorController, 
        TurnOnEditor, TurnOffEditor, ChangeBrush, ToggleMarkersVis, ToggleSpawnersVis, 
        ToggleSnapNav, 
        ChangeEditorMode, UnghostAll, TriggerThumbnails, ToggleEditorPanel, ToggleAssetsPanel, ToggleNavmeshDebug
    };
    pub use crate::ghost::{
        PGEditorGhostPlugin, EditorGhostTransformMemory, Ghost, EditorAsset, 
    };
    pub use crate::thumbnails::PGEditorThumbnailsPlugin;
    pub use crate::tracker::{
        PGEditorTrackerPlugin, Undo, Redo, UndoMessage,
         RedoMessage, Changes, Change, ChangesSet, CurrentTransformChanges
    };
    pub use crate::planes::plane_mesh;
    pub use crate::vertex::{
        SpawnVertices, SelectedVertex, PlaneVertex, PGEditorVertexPlugin, VertexRefs, ShowVertices, HideVertices
    };
    pub use crate::terrain_brushes::{
        TerrainHeightBrush, TerrainColorBrush, HeightBrushType, ColorBrushType
    };
    pub use crate::noises::{NoiseType, Noise};
    pub use crate::settings::{EditorSettings, EditorMode};
    pub use crate::text_inputs::text_input_field;

    pub use crate::PGEditorPlugin;
}