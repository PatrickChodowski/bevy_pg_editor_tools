use bevy::prelude::*;
use bevy_pg_scenes::prelude::{Spawner, Marker, Markee, Spawnee, Static,TerrainChunk};
use bevy_pg_core::prelude::{GameStatePlay, MainCamera, Player};
use bevy_enhanced_input::prelude::ContextActivity;

pub mod assets_panel;
pub mod box_select;
pub mod brushes;
pub mod controller;
pub mod ghost;
pub mod tracker;
pub mod thumbnails;
pub mod noises;
pub mod planes;
pub mod vertex;
pub mod ui;
pub mod settings;
pub mod terrain_brushes;

use assets_panel::PGEditorAssetsPanelPlugin;
use brushes::{PGEditorBrushSelectPlugin, BrushSelectController};
use box_select::PGEditorBoxSelectPlugin;
use controller::{PGEditorControllerPlugin, EditorController};
use ghost::{PGEditorGhostPlugin, EditorGhostSettings, EditorAsset, EditorGhostTransformMemory};
use planes::PlaneToEdit;
use settings::EditorSettings;
use thumbnails::PGEditorThumbnailsPlugin;
use tracker::{PGEditorTrackerPlugin, CurrentTransformChanges, Changes};
use ui::PGEditorUIPlugin;
use vertex::{PGEditorVertexPlugin, SpawnVertices};

use crate::brushes::BrushType;


pub struct PGEditorPlugin{
    pub spawner_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub marker_mesh: fn(id: usize, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> (Handle<Mesh>, Handle<StandardMaterial>),
    pub markers_mapping: fn(name: String) -> Marker,
    pub spawners_mapping: fn(name: String, option: Option<String>) -> Spawner,
    pub brush_mapping: fn(commands: &mut Commands, brush_id: usize, editor_settings: &ResMut<EditorSettings>) -> Box<dyn BrushType>,
    pub brush_id_labels: Vec<(usize, &'static str)>,
    pub vertex_radius: f32
}

impl Plugin for PGEditorPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(
            (
                PGEditorTrackerPlugin,
                PGEditorBrushSelectPlugin,
                PGEditorBoxSelectPlugin,
                PGEditorAssetsPanelPlugin,
                PGEditorThumbnailsPlugin,
                PGEditorControllerPlugin,
                PGEditorVertexPlugin{
                    vertex_radius: self.vertex_radius
                },
                PGEditorGhostPlugin{
                    spawner_mesh: self.spawner_mesh,
                    marker_mesh: self.marker_mesh,
                    markers_mapping: self.markers_mapping,
                    spawners_mapping: self.spawners_mapping
                },
                PGEditorUIPlugin
            )
        )
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
    brush: Query<Entity, With<BrushSelectController>>,
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

    let Ok(entity) = brush.single() else { return };
    commands
        .entity(entity)
        .insert(ContextActivity::<BrushSelectController>::ACTIVE);

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
    brush: Query<Entity, With<BrushSelectController>>,
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

    let Ok(brush_entity) = brush.single() else {
        return;
    };
    commands
        .entity(brush_entity)
        .insert(ContextActivity::<BrushSelectController>::INACTIVE);

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
    pub use crate::box_select::{
        BoxSelectController, box_select_controller, box_select_changed, 
        BoxSelectFinal, BoxSelect, PGEditorBoxSelectPlugin
    };
    pub use crate::brushes::{
        BrushSelectController, brush_select_controller, brush_changed, BrushDone, BrushStart,
         Brush, PGEditorBrushSelectPlugin, BrushType, ScatterBrush, NothingBrush
    };
    pub use crate::controller::{
        PGEditorControllerPlugin, editor_controller, EditorController, 
        TurnOnEditor, TurnOffEditor, SaveScene, ChangeBrush, ToggleMarkersVis, ToggleSpawnersVis, 
        ToggleGhostAxis, ToggleGhostMode, ToggleSnapNav, ToggleMultiGhost, 
        ChangeEditorMode, NavMeshGeneration, UnghostAll, TriggerThumbnails, ToggleEditorPanel, ToggleAssetsPanel
    };
    pub use crate::ghost::{
        PGEditorGhostPlugin, EditorGhostTransformMemory, Ghost, 
        EditorAsset, GhostTransformAxis, GhostTransformMode
    };
    pub use crate::thumbnails::PGEditorThumbnailsPlugin;
    pub use crate::tracker::{
        PGEditorTrackerPlugin, Undo, Redo, UndoMessage,
         RedoMessage, Changes, Change, ChangesSet, CurrentTransformChanges
    };
    pub use crate::planes::{PlaneToEdit, plane_mesh};
    pub use crate::vertex::{
        SpawnVertices, SelectedVertex, PlaneVertex, PGEditorVertexPlugin, 
        TerrainVertexController, VertexRefs, terrain_vertex_controller, ShowVertices, HideVertices
    };
    pub use crate::terrain_brushes::{
        TerrainHeightBrush, TerrainColorBrush, HeightBrushType, ColorBrushType
    };
    pub use crate::noises::{NoiseType, Noise};
    pub use crate::settings::{EditorSettings, EditorMode};

    pub use crate::PGEditorPlugin;
}