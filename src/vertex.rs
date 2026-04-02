use bevy::math::f32;
use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::mesh::SerializedMesh;
use bevy::mesh::VertexAttributeValues;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::color::palettes::css::ORANGE_RED;
use bevy_enhanced_input::prelude::*;
use bevy_pg_core::prelude::GameStatePlay;

use crate::planes::PlaneToEdit;

pub struct PGEditorVertexPlugin;


impl Plugin for PGEditorVertexPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, init)
        .add_observer(on_spawn_vertices)
        .add_observer(on_plane_edit_add)
        .add_observer(on_remove_plane)
        .add_observer(select_vertex)
        .add_observer(deselect_vertex)
        .add_systems(Update, input_delesect_all_vertices.run_if(input_just_pressed(MouseButton::Right)))
        .add_observer(deselect_all_vertices)
        .add_systems(Update, vertex_changed)
        .add_observer(show_vertices)
        .add_observer(hide_vertices)
        ;
    }
}

fn show_vertices(
    _trigger: On<ShowVertices>,
    mut query:  Query<&mut Visibility, With<PlaneVertex>>, 
){
    for mut vis in query.iter_mut(){
        *vis = Visibility::Visible;
    }
}


fn hide_vertices(
    _trigger: On<HideVertices>,
    mut query:  Query<&mut Visibility, With<PlaneVertex>>
){
    for mut vis in query.iter_mut(){
        *vis = Visibility::Hidden;
    }
}

pub fn load_mesh_from_file(path: &str) -> std::io::Result<Mesh> {
    let json = std::fs::read_to_string(path)?;
    let serialized: SerializedMesh = serde_json::from_str(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    return Ok(serialized.into_mesh());
}


#[derive(Resource)]
pub struct VertexRefs {
    mesh_handle: Mesh3d,
    selected_mat_handle: MeshMaterial3d<StandardMaterial>,
    mat_handle: MeshMaterial3d<StandardMaterial>
}

fn init(
    mut commands:   Commands,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>
){
    commands.insert_resource(
        VertexRefs{
            mesh_handle: Mesh3d(meshes.add(Sphere{radius: 1.0, ..default()})),
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

#[derive(Event)]
pub struct ShowVertices;

#[derive(Event)]
pub struct HideVertices;


fn on_plane_edit_add(
    trigger:      On<Add, PlaneToEdit>,
    mut commands: Commands,
    planes:       Query<&PlaneToEdit>,
    query:        Query<&Mesh3d>,
    meshes:       Res<Assets<Mesh>>,
    vertex_refs:  Res<VertexRefs>
){
    let Ok(mesh3d) = query.get(trigger.entity) else {return};
    let Some(mesh) = meshes.get(&mesh3d.0) else {return};
    let (v_pos, v_clr) = extract_mesh_data(mesh);
    let mut vertices: Vec<Entity> = Vec::new();
    let Ok(plane) = planes.get(trigger.entity) else {return};

    let scale = plane.calculate_optimal_vertex_radius(0.3);

    for (index, pos) in v_pos.iter().enumerate(){
        let entity = commands.spawn((
            vertex_refs.mat_handle.clone(),
            vertex_refs.mesh_handle.clone(),
            NotShadowCaster,
            NotShadowReceiver,
            Transform::from_translation(pos.clone().into()).with_scale(Vec3::splat(scale)),
            PlaneVertex::new(index, pos, &v_clr[index], scale, trigger.entity),
            DespawnOnExit(GameStatePlay::Editor),
            Visibility::Hidden
        )).id();
        vertices.push(entity);
    }
    commands.entity(trigger.entity).add_children(&vertices);
}


fn on_spawn_vertices(
    trigger:      On<SpawnVertices>,
    mut commands: Commands,
    query:        Query<(&Mesh3d, &PlaneToEdit)>,
    meshes:       Res<Assets<Mesh>>,
    vertex_refs:  Res<VertexRefs>
){
    let Ok((mesh3d, plane)) = query.get(trigger.plane_entity) else {return;};
    let Some(mesh) = meshes.get(&mesh3d.0) else {return;};
    let (v_pos, v_clr) = extract_mesh_data(mesh);
    let mut vertices: Vec<Entity> = Vec::new();

    let scale = plane.calculate_optimal_vertex_radius(0.3);

    for (index, pos) in v_pos.iter().enumerate(){
        let entity = commands.spawn((
            vertex_refs.mat_handle.clone(),
            vertex_refs.mesh_handle.clone(),
            NotShadowCaster,
            NotShadowReceiver,
            Transform::from_translation(pos.clone().into()).with_scale(Vec3::splat(scale)),
            PlaneVertex::new(index, pos, &v_clr[index], scale, trigger.plane_entity),
            DespawnOnExit(GameStatePlay::Editor),
            Visibility::Hidden
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
            commands.entity(entity).try_despawn();
        }
    }
}


fn select_vertex(
    trigger:       On<Add, SelectedVertex>,
    mut commands:  Commands,
    vertex_refs:   Res<VertexRefs>,
){
    commands.entity(trigger.entity).try_insert(vertex_refs.selected_mat_handle.clone());
}

fn deselect_vertex(
    trigger:       On<Remove, SelectedVertex>,
    mut commands:  Commands,
    vertex_refs:   Res<VertexRefs>,
){
    commands.entity(trigger.entity).try_insert(vertex_refs.mat_handle.clone());
}

fn input_delesect_all_vertices(
    mut commands: Commands
){
    commands.trigger(DeselectAllVertices);
}

fn deselect_all_vertices(
    _trigger: On<DeselectAllVertices>,
    mut commands: Commands,
    query:  Query<Entity, With<SelectedVertex>>
){
    for entity in query.iter(){
        commands.entity(entity).try_remove::<SelectedVertex>();
    }
}

#[derive(InputAction, Event)]
#[action_output(bool)]
pub struct DeselectAllVertices;

fn vertex_changed(
    mut changed_vertices:   Query<(Entity, &PlaneVertex), Changed<PlaneVertex>>,
    added_vertices:         Query<Entity, Added<PlaneVertex>>,
    plane_meshes:           Query<&Mesh3d, With<PlaneToEdit>>,
    mut meshes:             ResMut<Assets<Mesh>>
){
    for (vertex_entity, plane_vertex) in changed_vertices.iter_mut(){

        if added_vertices.contains(vertex_entity){
            continue;
        }

        let Ok(plane_mesh_3d) = plane_meshes.get(plane_vertex.plane_entity) else {continue};
        let Some(plane_mesh) = meshes.get_mut(&plane_mesh_3d.0) else {continue};
        let (mut v_pos, mut v_clr) = extract_mesh_data(plane_mesh);
        v_pos[plane_vertex.index] = plane_vertex.loc;
        v_clr[plane_vertex.index] = plane_vertex.clr;
        plane_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
        plane_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_clr);
        plane_mesh.compute_normals();
    }
}
