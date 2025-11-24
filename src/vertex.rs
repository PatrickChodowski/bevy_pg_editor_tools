use bevy::math::f32;
use bevy::prelude::*;
use bevy::mesh::SerializedMesh;
use bevy::mesh::VertexAttributeValues;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::color::palettes::css::ORANGE_RED;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::prelude::Press;

use crate::planes::PlaneToEdit;

pub struct TerrainEditorVertexPlugin {
    pub vertex_radius: f32
}

impl TerrainEditorVertexPlugin {
    pub fn new(vertex_radius: f32) -> Self {
        TerrainEditorVertexPlugin {
            vertex_radius
        }
    }
}

impl Plugin for TerrainEditorVertexPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, init)
        .insert_resource(VertexPluginSettings::new(
            self.vertex_radius
        ))
        .add_observer(init_plane_to_edit)
        .add_observer(on_remove_plane)
        .add_observer(select_vertex)
        .add_observer(deselect_vertex)
        .add_observer(deselect_all_vertices)
        .add_systems(Update, vertex_changed)
        .add_observer(serialize_planes)
        ;
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct SerializePlanes;

fn serialize_planes(
    _trigger: On<Fire<SerializePlanes>>,
    meshes:   Res<Assets<Mesh>>,
    query:   Query<&Mesh3d, With<PlaneToEdit>>
){
    for mesh3d in query.iter(){
        let Some(mesh) = meshes.get(&mesh3d.0) else {continue;};
        let serialized_mesh = SerializedMesh::from_mesh(mesh.clone());
        let json = serde_json::to_string_pretty(&serialized_mesh).unwrap();
        let _a = std::fs::write("assets/meshes/mesh_serialized.json", json);
    }
}

pub fn load_mesh_from_file(path: &str) -> std::io::Result<Mesh> {
    let json = std::fs::read_to_string(path)?;
    let serialized: SerializedMesh = serde_json::from_str(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    return Ok(serialized.into_mesh());
}

#[derive(Component, Reflect)]
pub struct TerrainVertexController;

pub fn terrain_vertex_controller() -> impl Bundle {
    return (
        TerrainVertexController,
        actions!(
            TerrainVertexController[
                (
                    Action::<DeselectAllVertices>::new(),
                    Press::default(),
                    bindings![MouseButton::Right]
                ),
                (
                    Action::<SerializePlanes>::new(),
                    Press::default(),
                    bindings![KeyCode::Space]
                )
            ]
        )
    );
}

#[derive(Resource)]
struct VertexPluginSettings {
    radius: f32
}
impl VertexPluginSettings {
    fn new(radius: f32) -> Self {
        VertexPluginSettings{radius}
    }
}

#[derive(Resource)]
pub struct VertexRefs {
    mesh_handle: Mesh3d,
    selected_mat_handle: MeshMaterial3d<StandardMaterial>,
    mat_handle: MeshMaterial3d<StandardMaterial>,
    pub radius: f32
}

fn init(
    mut commands:   Commands,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    vertex_plugin_settings:    Res<VertexPluginSettings>
){
    commands.insert_resource(
        VertexRefs{
            radius: vertex_plugin_settings.radius,
            mesh_handle: Mesh3d(meshes.add(Sphere{radius: vertex_plugin_settings.radius, ..default()})),
            mat_handle: MeshMaterial3d(materials.add(Color::BLACK.with_alpha(0.85))),
            selected_mat_handle: MeshMaterial3d(materials.add(Color::from(ORANGE_RED).with_alpha(0.85)))
        }
    );
}

#[derive(Component, Copy, Clone)]
pub struct PlaneVertex {
    pub index: usize,
    pub loc: [f32;3],
    pub clr: [f32;4],
    pub radius: f32,
    pub plane_entity: Entity
}
impl PlaneVertex {
    pub fn new(
        index: usize, 
        loc: &[f32;3], 
        clr: &[f32; 4], 
        radius: f32,
        plane_entity: Entity
    ) -> Self{
        PlaneVertex {
            loc: *loc, 
            clr: *clr, 
            index, 
            radius, 
            plane_entity
        }
    }
}

#[derive(Component)]
pub struct SelectedVertex;

pub fn extract_mesh_data(mesh: &Mesh) -> (Vec<[f32; 3]>, Vec<[f32; 4]>){
    let v_pos: Vec<[f32; 3]> = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap().to_vec();
    let mut v_clr: Vec<[f32; 4]> = Vec::new();
    if let Some(attr_vcolor) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        if let VertexAttributeValues::Float32x4(vcolors) = attr_vcolor {
            v_clr = vcolors.to_vec();
        }
    } else {
        v_clr = vec![[1.0, 1.0, 1.0, 1.0]; v_pos.len()];
    }
    return (v_pos, v_clr);
}

#[derive(Event)]
pub struct SpawnVertices{
    pub plane_entity: Entity
}

fn init_plane_to_edit(
    trigger:      On<SpawnVertices>,
    mut commands: Commands,
    query:        Query<&Mesh3d, With<PlaneToEdit>>,
    meshes:       Res<Assets<Mesh>>,
    vertex_refs:  Res<VertexRefs>
){
    let Ok(mesh3d) = query.get(trigger.plane_entity) else {return;};
    let Some(mesh) = meshes.get(&mesh3d.0) else {return;};
    let (v_pos, v_clr) = extract_mesh_data(mesh);
    let mut vertices: Vec<Entity> = Vec::new();
    for (index, pos) in v_pos.iter().enumerate(){
        let entity = commands.spawn((
            vertex_refs.mat_handle.clone(),
            vertex_refs.mesh_handle.clone(),
            NotShadowCaster,
            NotShadowReceiver,
            Transform::from_translation(pos.clone().into()).with_scale(Vec3::splat(1.0)),
            PlaneVertex::new(index, pos, &v_clr[index], vertex_refs.radius, trigger.plane_entity),
        )).id();
        vertices.push(entity);
    }
    commands.entity(trigger.plane_entity).add_children(&vertices);
}

fn on_remove_plane(
    trigger:       On<Remove, PlaneToEdit>,
    mut commands: Commands,
    vertex:       Query<(Entity, &PlaneVertex)>
){
    for (entity, plane_vertex) in vertex.iter(){
        if plane_vertex.plane_entity == trigger.entity{
            commands.entity(entity).despawn();
        }
    }
}


fn select_vertex(
    trigger:       On<Add, SelectedVertex>,
    mut commands:  Commands,
    vertex_refs:   Res<VertexRefs>,
){
    commands.entity(trigger.entity).insert(vertex_refs.selected_mat_handle.clone());
}

fn deselect_vertex(
    trigger:       On<Remove, SelectedVertex>,
    mut commands:  Commands,
    vertex_refs:   Res<VertexRefs>,
){
    commands.entity(trigger.entity).insert(vertex_refs.mat_handle.clone());
}

fn deselect_all_vertices(
    _trigger: On<Fire<DeselectAllVertices>>,
    mut commands: Commands,
    query:  Query<Entity, With<SelectedVertex>>
){
    for entity in query.iter(){
        commands.entity(entity).remove::<SelectedVertex>();
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct DeselectAllVertices;

fn vertex_changed(
    mut vertices:   Query<&PlaneVertex, (With<SelectedVertex>, Changed<PlaneVertex>)>,
    plane_mesh3d:   Single<&Mesh3d, With<PlaneToEdit>>,
    mut meshes:     ResMut<Assets<Mesh>>
){
    let Some(plane_mesh) = meshes.get_mut(&plane_mesh3d.0) else {return;};
    let (mut v_pos, mut v_clr) = extract_mesh_data(plane_mesh);
    for plane_vertex in vertices.iter_mut(){
        v_pos[plane_vertex.index] = plane_vertex.loc;
        v_clr[plane_vertex.index] = plane_vertex.clr;
    }
    plane_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    plane_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_clr);
}
