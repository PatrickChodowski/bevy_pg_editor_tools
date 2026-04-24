
use bevy::prelude::*;
use bevy::pbr::wireframe::Wireframe;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::color::palettes::css::{WHITE, BLACK};
use bevy::color::palettes::tailwind::GRAY_500;
use bevy::mesh::SerializedMesh;
use bevy::picking::pointer::PointerId;
use bevy_pg_core::prelude::{GameStatePlay, PointerData, TerrainChunk};
use bevy_pg_nav::prelude::{GenerateNavMesh, PGNavmesh, NavConfig, TerrainRayMeshData};
use bevy_pg_scenes::prelude::{PGSerializedMesh, PlaneToEdit};
use bevy_simple_text_input::{
    TextInput, TextInputPlaceholder, TextInputSettings, TextInputTextFont, 
    TextInputTextColor, TextInputInactive, TextInputValue, TextInputSystem, TextInputSubmitMessage
};
use bevy::feathers::controls::{
    ButtonProps, SliderProps, ButtonVariant, ColorSliderProps, ColorChannel, ColorSwatch,  
    button, checkbox, radio, slider, color_slider, color_swatch, ColorSlider, SliderBaseColor
};
use bevy::feathers::theme::{ThemeBackgroundColor, ThemedText, UiTheme};
use bevy::ui::Checked;
use bevy::ui_widgets::{Activate, RadioButton, RadioGroup, 
    SliderStep, SliderValue, SliderPrecision, ValueChange, observe
};

use crate::ghost::{Ghost, SaveScene};
use crate::tracker::{Changes, ChangePlaneDespawn, Change, ChangePlaneSpawn};
use crate::prelude::{EditorMode, EditorSettings};
use crate::text_inputs::{LocInputX, LocInputY, LocInputZ, PlaneDimXInput, PlaneDimZInput, PlaneSubsInput, string_to_f32, string_to_u32};


pub struct PGEditorPlanesPlugin;

impl Plugin for PGEditorPlanesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_observer(open_planes_popup)
        .add_observer(on_click_plane)
        .add_observer(navmesh_generation)
        .add_observer(serialize_plane)
        .add_observer(chunk_plane)
        .add_observer(delete_plane)
        .add_observer(spawn_plane)
        .add_systems(Update, read_plane_name_on_submit.after(TextInputSystem).run_if(in_state(GameStatePlay::Editor).and(on_message::<TextInputSubmitMessage>)))
        .add_observer(on_add_plane)
        .add_systems(Update, updated_plane)
        ;
    }
}

fn updated_plane(
    mut query: Query<(&Mesh3d, &GlobalTransform, &mut TerrainRayMeshData), Or<(Changed<Transform>, Changed<Mesh3d>, Changed<PlaneToEdit>)>>,
    meshes:    Res<Assets<Mesh>>
){
    for (plane_mesh, plane_transform, mut plane_trmd) in query.iter_mut(){
        let Some(mesh) = meshes.get(&plane_mesh.0) else {continue};
        let trmd = TerrainRayMeshData::from_mesh(mesh, &plane_transform.to_matrix());
        *plane_trmd = trmd;
    }
}


fn on_add_plane(
    trigger: On<Add, PlaneToEdit>,
    mut commands: Commands,
    query: Query<(&Mesh3d, &GlobalTransform),  (With<PlaneToEdit>, Without<TerrainRayMeshData>)>,
    meshes: Res<Assets<Mesh>>
){
    let Ok((plane_mesh, plane_transform)) = query.get(trigger.entity) else {return};
    let Some(mesh) = meshes.get(&plane_mesh.0) else {return};
    let trmd = TerrainRayMeshData::from_mesh(mesh, &plane_transform.to_matrix());
    commands.entity(trigger.entity).insert(trmd);
}


fn read_plane_name_on_submit(
    mut msgs:     MessageReader<TextInputSubmitMessage>,
    mut commands: Commands,
    forms:        Query<&PlaneNameInput>,
    planes:       Query<Entity, With<PlaneToEdit>>
){
    for msg in msgs.read(){
        if let Ok(plane_name_input) = forms.get(msg.entity){
            if let Ok(plane_entity) = planes.get(plane_name_input.plane_entity){
                commands.entity(plane_entity).insert(Name::from(msg.value.clone()));
            }
        } 
    }
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
        Transform::from_translation(loc),
        TerrainChunk{dims: Vec2::new(dim_x, dim_z)}
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



fn delete_plane(
    trigger:      On<DeletePlane>,
    mut commands: Commands,
    planes:       Query<(Entity, &PlaneToEdit, &Transform)>,
    popups:       Query<(Entity, &PlanesPopup)>,
    mut changes:  ResMut<Changes>
){
    if let Ok((plane_entity, plane, plane_transform)) = planes.get(trigger.plane_entity){
        commands.entity(plane_entity).despawn();

        let c = ChangePlaneDespawn::new(plane_entity, plane.width, plane.height, plane.subdivisions, plane_transform.translation);
        c.record(&mut changes);

        for (popup_entity, planes_popup) in popups.iter(){
            if planes_popup.plane_entity == plane_entity {
                commands.entity(popup_entity).despawn();
                break;
            }
        }
    }
}

fn serialize_plane(
    trigger:         On<SerializePlane>,
    planes:          Query<(&Mesh3d, &Transform, Option<&Name>, &PlaneToEdit)>,
    meshes:          Res<Assets<Mesh>>
){

    if let Ok((mesh3d, transform, maybe_name, plane)) = planes.get(trigger.plane_entity){

        if maybe_name.is_none(){
            warn!("Plane needs a name before serializing");
            return;
        }

        let Some(mesh) = meshes.get(&mesh3d.0) else {return};
        let serialized_mesh = SerializedMesh::from_mesh(mesh.clone());
        let name_str = maybe_name.unwrap().to_string();
        let pg_serialized_mesh = PGSerializedMesh{
            saved_loc: transform.translation,
            name: name_str.clone(),
            data: serialized_mesh,
            width: plane.width,
            height: plane.height,
            subdivisions: plane.subdivisions
        };

        let json = serde_json::to_string_pretty(&pg_serialized_mesh).unwrap();
        let mesh_path: String = format!("assets/scenes/terrains/{}.mesh.json", name_str);
        info!("serializing to path: {}", mesh_path);
        let res = std::fs::write(mesh_path, json);
        info!("{:?}", res);    
    }
}


fn chunk_plane(
    trigger:        On<ChunkPlane>,
    mut commands:   Commands,
    planes:         Query<(&Mesh3d, &PlaneToEdit, Option<&Name>)>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
){
    let Ok((plane_mesh, plane, maybe_name)) = planes.get(trigger.plane_entity) else {return};

    if maybe_name.is_none(){
        warn!("Plane should have name before chunking!");
        return;
    }

    // count faces:
    let side_edge_count = plane.subdivisions + 1;
    let face_count: u32 = (plane.subdivisions + 1)*(plane.subdivisions + 1);
    let cc = chunk_candidates(side_edge_count);
    let n_chunks = cc.last().unwrap();
    info!("Chunking Plane: {} face count: {} chunk options: {:?} final: {}", trigger.plane_entity, face_count, cc, n_chunks);

    let chunk_width = plane.width/(*n_chunks as f32);
    let chunk_height = plane.height/(*n_chunks as f32);

    let s = (*n_chunks as f32).sqrt() as u32;
    let chunk_edge_len = side_edge_count / s;

    let Some(original_mesh) = meshes.get(&plane_mesh.0) else {return};
    let Some(VertexAttributeValues::Float32x3(pos)) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {return};
    let Some(VertexAttributeValues::Float32x3(norm)) = original_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) else {return};
    let Some(VertexAttributeValues::Float32x2(uvs)) = original_mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {return};
    let maybe_colors = original_mesh.attribute(Mesh::ATTRIBUTE_COLOR);

    let mut new_meshes = Vec::new();
    for chunk_y in 0..s {
        for chunk_x in 0..s {
            let mut new_pos = Vec::new();
            let mut new_norm = Vec::new();
            let mut new_uvs = Vec::new();
            let mut chunk_colors = Vec::new();

            let start_v_x = chunk_x * chunk_edge_len;
            let start_v_y = chunk_y * chunk_edge_len;

            for y in 0..=chunk_edge_len {
                for x in 0..=chunk_edge_len {
                    let orig_idx = ((start_v_y + y) * (side_edge_count + 1) + (start_v_x + x)) as usize;
                    let v_pos = Vec3::from(pos[orig_idx]);
                    new_pos.push(v_pos);
                    new_norm.push(norm[orig_idx]);
                    new_uvs.push(uvs[orig_idx]);

                    if let Some(VertexAttributeValues::Float32x4(original_colors)) = maybe_colors {
                        chunk_colors.push(original_colors[orig_idx]);
                    }

                }
            }

            // --- Step 2: Calculate the center of this chunk ---
            let min = new_pos.iter().fold(Vec3::splat(f32::MAX), |acc, v| acc.min(*v));
            let max = new_pos.iter().fold(Vec3::splat(f32::MIN), |acc, v| acc.max(*v));
            let chunk_center = (min + max) / 2.0;

            // --- Step 3: Re-center vertices (Local Space) ---
            let centered_pos: Vec<[f32; 3]> = new_pos.into_iter()
                .map(|v| (v - chunk_center).to_array())
                .collect();


            // 4. Build Indices (Counter-Clockwise Winding)
            let mut new_indices = Vec::new();
            let v_per_side = chunk_edge_len + 1;

            for y in 0..chunk_edge_len {
                for x in 0..chunk_edge_len {
                    // This is the index of the top-left vertex of the current quad
                    let i = y * v_per_side + x;

                    // To point the normal towards +Y (Up), we use CCW winding:
                    // Triangle 1: Top-Left -> Bottom-Left -> Top-Right
                    // Triangle 2: Top-Right -> Bottom-Left -> Bottom-Right
                    new_indices.extend_from_slice(&[
                        i,                  // Top-Left
                        i + v_per_side,     // Bottom-Left
                        i + 1,              // Top-Right
                        
                        i + 1,              // Top-Right
                        i + v_per_side,     // Bottom-Left
                        i + v_per_side + 1, // Bottom-Right
                    ]);
                }
            }

            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, centered_pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_norm);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
            if !chunk_colors.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, chunk_colors);
            }

            mesh.insert_indices(Indices::U32(new_indices));

            new_meshes.push((mesh, chunk_center));

        }
    }
    // let mut rng = rand::rng();

    for (index, new_mesh_data) in new_meshes.iter().enumerate(){

        // let debug_color = Color::srgb(
        //     rng.random_range(0.0..1.0),
        //     rng.random_range(0.0..1.0),
        //     rng.random_range(0.0..1.0),
        // );

        commands.spawn(
            (
                Mesh3d(meshes.add(new_mesh_data.0.clone())),
                // MeshMaterial3d(materials.add(StandardMaterial {
                //     base_color: debug_color,
                //     unlit: true, // Optional: makes it easier to see colors without lighting
                //     ..default()
                // })),
                MeshMaterial3d(
                    materials.add(StandardMaterial::from_color(Color::WHITE))
                ),
                Transform::from_translation(new_mesh_data.1),
                Pickable{should_block_lower: true, ..default()},
                PlaneToEdit{width: chunk_width, height: chunk_height, subdivisions: 0, changes: 0}, // TODO probably calculate subdivisions inside chunk
                Name::from(format!("{}_{}", maybe_name.unwrap(), index))
            )
        );

    }

    commands.entity(trigger.plane_entity).despawn();

}


fn chunk_candidates(k: u32) -> Vec<u32> {
    let mut results = Vec::new();
    for s in 2..=k {
        if k % s == 0 {
            results.push(s.pow(2));
        }
    }
    results.sort_unstable();
    results
}


fn navmesh_generation(
    trigger:       On<NavMeshGeneration>,
    planes:        Query<&PlaneToEdit>,
    mut commands:  Commands
){
    if planes.contains(trigger.plane_entity){
        info!("Triggered navmesh generation for {}", trigger.plane_entity);
        commands.trigger(GenerateNavMesh{plane_entity: trigger.plane_entity});
    }
}

#[derive(Event)]
struct DeletePlane{
    plane_entity: Entity
}

#[derive(Event)]
struct SerializePlane{
    plane_entity: Entity
}

#[derive(Event)]
struct ChunkPlane{
    plane_entity: Entity
}

#[derive(Event)]
struct NavMeshGeneration{
    plane_entity: Entity
}

#[derive(Event)]
pub struct SpawnPlane;


#[derive(Component)]
struct PlaneNameInput {
    plane_entity: Entity
}

#[derive(Component)]
struct DeletePlaneButton {
    plane_entity: Entity
}

#[derive(Component)]
struct NavGenButton {
    plane_entity: Entity
}
#[derive(Component)]
struct SerializeButton {
    plane_entity: Entity
}
#[derive(Component)]
struct SaveSceneButton {
    plane_entity: Entity
}

#[derive(Component)]
struct ChunkButton {
    plane_entity: Entity
}

fn plane_buttons(
    plane_entity: &Entity, 
    commands: &mut Commands, 
    maybe_name: Option<&Name>
) -> Entity {

    let name_input = commands.spawn(
        (
            Node {
                width: Val::Px(200.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(2.0)),
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::from(BLACK)),
            BackgroundColor(WHITE.into()),
            TextInput,
            PlaneNameInput{plane_entity: *plane_entity},
            TextInputPlaceholder{value: "".to_string(), ..default()},
            TextInputTextFont(TextFont {
                font_size: 17.0,
                ..default()
            }),
            TextInputSettings{
                retain_on_submit: true,
                mask_character: None,
                max_length: Some(20)
            },
            TextInputTextColor(TextColor(BLACK.into())),
            TextInputInactive(true),
        )
    ).id();

    if let Some(name) = maybe_name {
        commands.entity(name_input).insert(TextInputValue(name.to_string()));
    }

    let local_root = commands.spawn(
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        children![
            (
                button(
                    ButtonProps::default(),
                    NavGenButton{plane_entity: *plane_entity},
                    Spawn((Text::new("Navmesh Generation"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, navgenbuttons: Query<&NavGenButton>, settings: Res<EditorSettings>, ghost_planes: Query<Entity, (With<PlaneToEdit>, With<Ghost>)>| {
                    if settings.plane_apply_to_all {
                        info!("Applying Navmesh Generation to All Ghost Planes");
                        for entity in ghost_planes.iter(){
                            commands.trigger(NavMeshGeneration{plane_entity: entity});
                        }
                    } else {
                        if let Ok(ngb) = navgenbuttons.get(_activate.entity){
                            commands.trigger(NavMeshGeneration{plane_entity: ngb.plane_entity});
                        }
                    }
                })       
            ),
            (
                button(
                    ButtonProps::default(),
                    ChunkButton{plane_entity: *plane_entity},
                    Spawn((Text::new("Chunk"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&ChunkButton>| {
                    if let Ok(btn) = sbuttons.get(_activate.entity){
                        commands.trigger(ChunkPlane{plane_entity: btn.plane_entity});
                    }
                })   
            ),
            (
                button(
                    ButtonProps::default(),
                    SerializeButton{plane_entity: *plane_entity},
                    Spawn((Text::new("Serialize"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&SerializeButton>, settings: Res<EditorSettings>, ghost_planes: Query<Entity, (With<PlaneToEdit>, With<Ghost>)>| {
                    if settings.plane_apply_to_all {
                        info!("Applying Serialize to All Ghost Planes");
                        for entity in ghost_planes.iter(){
                            commands.trigger(SerializePlane{plane_entity: entity});
                        }
                    } else {
                        if let Ok(btn) = sbuttons.get(_activate.entity){
                            commands.trigger(SerializePlane{plane_entity: btn.plane_entity});
                        }
                    }
                })   
            ),
            (
                button(
                    ButtonProps::default(),
                    SaveSceneButton{plane_entity: *plane_entity},
                    Spawn((Text::new("Save Scene"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&SaveSceneButton>, settings: Res<EditorSettings>, ghost_planes: Query<Entity, (With<PlaneToEdit>, With<Ghost>)> | {
                    if settings.plane_apply_to_all {
                        info!("Applying SaveScene to All Ghost Planes");
                        for entity in ghost_planes.iter(){
                            commands.trigger(SaveScene{plane_entity: entity});
                        }
                    } else {
                        if let Ok(btn) = sbuttons.get(_activate.entity){
                            commands.trigger(SaveScene{plane_entity: btn.plane_entity});
                        }
                    }
                })   
            ),
            (
                button(
                    ButtonProps::default(),
                    (DeletePlaneButton{plane_entity: *plane_entity}),
                    Spawn((Text::new("Delete"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&DeletePlaneButton>, settings: Res<EditorSettings>, ghost_planes: Query<Entity, (With<PlaneToEdit>, With<Ghost>)> | {
                    if settings.plane_apply_to_all {
                        info!("Applying Delete to All Ghost Planes");
                        for entity in ghost_planes.iter(){
                            commands.trigger(DeletePlane{plane_entity: entity});
                        }
                    } else {
                        if let Ok(btn) = sbuttons.get(_activate.entity){
                            commands.trigger(DeletePlane{plane_entity: btn.plane_entity});
                        }
                    }
                })   
            )
        ]
    )).id();

    commands.entity(local_root).add_child(name_input);
    return local_root;
}


fn on_click_plane(
    trigger:      On<Pointer<Press>>,
    mut commands: Commands,
    query:        Query<Entity, With<PlaneToEdit>>,
    pointer:      Res<PointerData>,
    popups:       Query<Entity, With<PlanesPopup>>,
    editor_settings: Res<EditorSettings>,
    keys:         Res<ButtonInput<KeyCode>>
){


    if editor_settings.mode != EditorMode::Plane{
        return;
    }

    if keys.pressed(KeyCode::KeyB) || keys.just_pressed(KeyCode::KeyB){
        return;
    }

    if trigger.pointer_id == PointerId::Mouse {
        if trigger.button == PointerButton::Middle {
            for popup_entity in popups.iter(){
                commands.entity(popup_entity).try_despawn();
            }
            if let Ok(entity) = query.get(trigger.entity){

                commands.trigger(
                    OpenPopup{
                        entity,
                        click_coords: pointer.cursor_pos.unwrap()
                    })
            } else {};
        } else {
            if trigger.button == PointerButton::Secondary {
                for popup_entity in popups.iter(){
                    commands.entity(popup_entity).try_despawn();
                }
            } 
        }
    }
}


#[derive(Component)]
struct PlanesPopup{
    plane_entity: Entity
}

fn popup_bundle(coords: Vec2, plane_entity: Entity) -> impl Bundle {
    return(
        PlanesPopup{plane_entity},
        Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            left: px(coords.x),
            top: px(coords.y),
            width: px(300.0),
            height: px(200.0),
            border: UiRect::all(px(2.5)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(5.0)),
            ..default()
        },
        BorderColor::all(Color::from(WHITE)),
        BoxShadow::default(),
        BackgroundColor(Color::from(GRAY_500.with_alpha(0.7))),
        DespawnOnExit(GameStatePlay::Editor)
    );
}


fn open_planes_popup(
    trigger:      On<OpenPopup>,
    mut commands: Commands,
    query:        Query<(Entity, Option<&Name>, &PlaneToEdit)>
){
    let Ok((plane_entity, maybe_name, _plane)) = query.get(trigger.entity) else {return};

    let popup_entity = commands.spawn(popup_bundle(trigger.click_coords, plane_entity)).id();
    let buttons = plane_buttons(&plane_entity, &mut commands, maybe_name);

     let mut child_entities: Vec<Entity> = Vec::new();
     child_entities.push(buttons);

    commands.entity(popup_entity).add_children(&child_entities);
}



#[derive(EntityEvent)]
pub struct OpenPopup{
    pub entity: Entity,
    pub click_coords: Vec2,
}


pub fn plane_mesh(
    width: f32,
    height: f32,
    subdivisions: u32,
    meshes: &mut ResMut<Assets<Mesh>>
) -> impl Bundle {
    (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(width, height).subdivisions(subdivisions))),
        Pickable{should_block_lower: true, ..default()},
        PlaneToEdit{width, height, subdivisions, changes: 0}
    )
}
