
use bevy::prelude::*;
use bevy::color::palettes::css::WHITE;
use bevy::color::palettes::tailwind::GRAY_500;
use bevy::mesh::SerializedMesh;
use bevy::picking::pointer::PointerId;
use bevy_pg_core::prelude::{GameStatePlay, PointerData};
use bevy_pg_nav::prelude::{GenerateNavMesh, PGNavmesh, NavConfig};
use bevy_pg_scenes::prelude::PGSerializedMesh;
use bevy::feathers::controls::{
    ButtonProps, SliderProps, ButtonVariant, ColorSliderProps, ColorChannel, ColorSwatch,  
    button, checkbox, radio, slider, color_slider, color_swatch, ColorSlider, SliderBaseColor
};
use bevy::feathers::theme::{ThemeBackgroundColor, ThemedText, UiTheme};
use bevy::ui::Checked;
use bevy::ui_widgets::{Activate, RadioButton, RadioGroup, 
    SliderStep, SliderValue, SliderPrecision, ValueChange, observe
};

use crate::tracker::{Changes, ChangePlaneDespawn, Change};
use crate::prelude::{EditorSettings, EditorMode};


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
        ;
    }
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
    planes:          Query<(&Mesh3d, Option<&Name>), With<PlaneToEdit>>,
    meshes:          Res<Assets<Mesh>>
){

    if let Ok((mesh3d, maybe_name)) = planes.get(trigger.plane_entity){
        let Some(mesh) = meshes.get(&mesh3d.0) else {return};
        let serialized_mesh = SerializedMesh::from_mesh(mesh.clone());
        let pg_serialized_mesh = PGSerializedMesh{data: serialized_mesh};
        let json = serde_json::to_string_pretty(&pg_serialized_mesh).unwrap();
        let mesh_path: String;
        if let Some(name) = maybe_name {
            mesh_path = format!("assets/meshes/{}.mesh.json", name);
        } else {
            mesh_path = format!("assets/meshes/_.mesh.json");
        }
        info!("serializing to path: {}", mesh_path);
        let res = std::fs::write(mesh_path, json);
        info!("{:?}", res);    
    }
}

fn chunk_plane(
    trigger:        On<ChunkPlane>,
    mut commands:   Commands,
    planes:         Query<&PlaneToEdit>
    // current_chunk:  Res<CurrentChunk>,
    // terrain_chunks: Query<(&TerrainChunk, &Name)>,
    // mapsdata:       Res<MapsData>
){

    if planes.contains(trigger.plane_entity){
        info!("Chunking Plane: {}", trigger.plane_entity);
    }

}


fn navmesh_generation(
    trigger:       On<NavMeshGeneration>,
    mut commands:   Commands,
    planes:         Query<&PlaneToEdit>
    // current_chunk:  Res<CurrentChunk>,
    // terrain_chunks: Query<(&TerrainChunk, &Name)>,
    // mapsdata:       Res<MapsData>
){

    if planes.contains(trigger.plane_entity){
        info!("Triggered navmesh generation for {}", trigger.plane_entity);
    }


    // for (terrain_chunk, name) in terrain_chunks.iter(){
    //     if (terrain_chunk.map_name == current_chunk.map_name) &
    //        (terrain_chunk.chunk_id == current_chunk.chunk_id) {

    //         commands.write_message(GenerateNavMesh::new(
    //             name.to_string(), 
    //             &current_chunk.map_name, 
    //             &current_chunk.chunk_id,
    //             mapsdata.chunk_size
            
    //         ));
    //         break;
    //     }
    // }
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
struct ChunkButton {
    plane_entity: Entity
}

fn plane_buttons(plane_entity: &Entity, commands: &mut Commands) -> Entity {
    commands.spawn(
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
                observe(|_activate: On<Activate>, mut commands: Commands, navgenbuttons: Query<&NavGenButton>| {
                    if let Ok(ngb) = navgenbuttons.get(_activate.entity){
                        commands.trigger(NavMeshGeneration{plane_entity: ngb.plane_entity});
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
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&SerializeButton>| {
                    if let Ok(btn) = sbuttons.get(_activate.entity){
                        commands.trigger(SerializePlane{plane_entity: btn.plane_entity});
                    }
                })   
            ),
            (
                button(
                    ButtonProps::default(),
                    (DeletePlaneButton{plane_entity: *plane_entity}),
                    Spawn((Text::new("Delete"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands, sbuttons: Query<&DeletePlaneButton>| {
                    if let Ok(btn) = sbuttons.get(_activate.entity){
                        commands.trigger(DeletePlane{plane_entity: btn.plane_entity});
                    }
                })   
            )
        ]
    )).id()
}


fn on_click_plane(
    trigger:      On<Pointer<Press>>,
    mut commands: Commands,
    query:        Query<Entity, With<PlaneToEdit>>,
    pointer:      Res<PointerData>,
    popups:       Query<Entity, With<PlanesPopup>>,
    editor_settings: Res<EditorSettings>
){


    if editor_settings.mode != EditorMode::Plane{
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
            height: px(150.0),
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
    query:        Query<(Entity, &PlaneToEdit)>
){
    let Ok((plane_entity, _plane)) = query.get(trigger.entity) else {return};

    let popup_entity = commands.spawn(popup_bundle(trigger.click_coords, plane_entity)).id();
    let buttons = plane_buttons(&plane_entity, &mut commands);

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
        PlaneToEdit{width, height, subdivisions}
    )
}


pub fn split_into_chunks(
    plane_mesh: &SerializedMesh, 
    planetoedit: &PlaneToEdit, 
    n_chunks: usize
)-> Option<Vec<SerializedMesh>>{
    return None;
}




#[derive(Component)]
pub struct PlaneToEdit{
    pub width: f32,
    pub height: f32,
    pub subdivisions: u32
}

impl PlaneToEdit {
    // Inserted into already created terrain
    pub fn dummy() -> Self {
        PlaneToEdit {
            width: 0.0,
            height: 0.0,
            subdivisions: 0
        }
    }
    pub fn new(width: f32, height: f32, subdivisions: u32) -> Self {
        PlaneToEdit {
            width, height, subdivisions
        }
    }
    pub fn ray_intersection(
        &self, 
        loc: Vec3, 
        scale: Vec3, 
        origin: Vec3A, 
        direction: Vec3A
    ) -> Option<f32> {

        let min_corner = Vec3A::new(loc.x - self.width*0.5*scale.x, loc.y, loc.z - self.height*0.5*scale.y);
        let max_corner = Vec3A::new(loc.x + self.width*0.5*scale.x, loc.y, loc.z + self.height*0.5*scale.y);

        let inv_dir = direction.recip();
        
        let t1 = (min_corner - origin) * inv_dir;
        let t2 = (max_corner - origin) * inv_dir;
        
        let t_min = Vec3A::min(t1, t2);
        let t_max = Vec3A::max(t1, t2);
        
        let t_enter = t_min.max_element();
        let t_exit = t_max.min_element();
        
        let hit: bool = t_enter <= t_exit && t_exit >= 0.0;
        if hit {
            return Some(t_enter.max(0.0));
        } else {
            return None;
        }
    }

    pub fn calculate_optimal_vertex_radius(&self, percentage: f32) -> f32 {
        let spacing_x = self.height / (self.subdivisions+1).max(1) as f32;
        let spacing_y = self.width / (self.subdivisions+1).max(1) as f32;
        let min_spacing = spacing_x.min(spacing_y);
        let max_radius = min_spacing / 2.0;
        let safe_fill = percentage.clamp(0.01, 0.99);
        max_radius * safe_fill
    }
}

