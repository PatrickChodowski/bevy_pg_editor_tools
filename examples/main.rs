
use bevy::input::common_conditions::input_just_pressed;
use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy::picking::pointer::PointerId;
use bevy::picking::hover::HoverMap;
use bevy::window::PrimaryWindow;
use bevy_enhanced_input::prelude::*;
use bevy_pg_editor_tools::prelude::{WorldPos, PGEditorToolsPlugin, PGEditorBrushSelectPlugin, BrushSelectController, BrushSettings, brush_select_controller};
use bevy_pg_terrain_editor_tools::noises::Noise;
use bevy_pg_terrain_editor_tools::prelude::{HeightBrushType, ColorBrushType, PlaneToEdit, SpawnVertices, 
    TerrainColorBrush, TerrainEditorVertexPlugin, TerrainHeightBrush, 
    TerrainVertexController, plane_mesh, terrain_vertex_controller
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<BrushSelectController>()
        .add_input_context::<TerrainVertexController>()
        .insert_resource(AmbientLight{color: Color::from(WHITE), brightness: 900.0, ..default()})
        .add_plugins(TerrainEditorVertexPlugin::new(1.0))
        .add_plugins(PGEditorToolsPlugin)
        .add_plugins(PGEditorBrushSelectPlugin)
        .add_systems(Startup, init)
        .add_systems(Update, hover_plane)
        .add_systems(Update, switch.run_if(input_just_pressed(KeyCode::KeyS)))
        .run();

}

#[derive(Resource)]
pub enum CurrentBrush {
    HeightsValue,
    HeightsNoise,
    Color
}


fn init(
    mut commands:      Commands,
    mut meshes:        ResMut<Assets<Mesh>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    mut brushsettings: ResMut<BrushSettings>
){

    brushsettings.radius = 1.0;
    brushsettings.typ = Box::new(TerrainHeightBrush{typ: HeightBrushType::Value(1.0), reselection: true});
    commands.insert_resource(CurrentBrush::HeightsValue);
    // brushsettings.typ = Box::new(TerrainColorBrush{color: [0.5, 0.5, 0.8, 1.0]});
    // brushsettings.typ = Box::new(TerrainColorBrush{typ: ColorBrushType::Range { min: 0.0, max: 5.0, min_clr: [0.0, 0.0, 0.0, 1.0], max_clr:[1.0, 1.0, 1.0, 1.0] }});
    // brushsettings.typ = Box::new(TerrainHeightBrush{typ: HeightBrushType::Noise((vec![Noise::new()], 1.0)), reselection: true});

    commands.spawn((
        brush_select_controller(),
        terrain_vertex_controller()
    ));

    let plane_entity = commands.spawn(
        (
            plane_mesh(40.0, 40.0, 12, &mut meshes),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
        )
    ).id();

    commands.spawn(
        (
            Camera3d::default(),
            Transform::from_xyz(25.0, 50.0, 25.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y)
        )
    );

    commands.trigger(SpawnVertices{plane_entity});
}

fn switch(
    mut brushsettings: ResMut<BrushSettings>,
    mut current_brush: ResMut<CurrentBrush>
){
    match *current_brush {
        CurrentBrush::HeightsValue => {
            *current_brush = CurrentBrush::HeightsNoise;
            brushsettings.typ = Box::new(TerrainHeightBrush{typ: HeightBrushType::Noise((vec![Noise::new()], 1.0)), reselection: true});
            info!("changed to height noise");
        }
        CurrentBrush::HeightsNoise => {
            *current_brush = CurrentBrush::Color;
            brushsettings.typ = Box::new(TerrainColorBrush{typ: ColorBrushType::Range { min: -5.0, max: 5.0, min_clr: [0.0, 0.0, 0.0, 1.0], max_clr:[1.0, 1.0, 1.0, 1.0] }});
            info!("changed to color range");
        }
        CurrentBrush::Color => {
            *current_brush = CurrentBrush::HeightsValue;
            brushsettings.typ = Box::new(TerrainHeightBrush{typ: HeightBrushType::Value(1.0), reselection: true});
            info!("changed to height value");
        }
    }
}

fn hover_plane(
    hovermap:           Res<HoverMap>,
    primary:            Single<&Window, With<PrimaryWindow>>,
    camera:             Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    nodes:              Query<Entity, With<Node>>,
    planes:             Query<(&PlaneToEdit, &Transform)>,
    mut worldpos:       ResMut<WorldPos>
) {
    worldpos.reset();
    let mut any_ui_hit: Option<Entity> = None;
    let hit_data = hovermap.0.get(&PointerId::Mouse).unwrap();

    if hit_data.len() > 0 {
        let hit_entities: Vec<Entity> = hit_data.keys().cloned().collect::<Vec<Entity>>();
        for entity in hit_entities.iter(){
            if let Ok(_node_entity) = nodes.get(*entity){
                any_ui_hit = Some(*entity);
            }
        }
    }

    let (main_camera, camera_transform) = camera.into_inner();
    let Some(cursor_position) = primary.cursor_position() else {return;};

    if any_ui_hit.is_some(){
        return;
    }

    if let Ok(ray) = main_camera.viewport_to_world(camera_transform, cursor_position) {
        let ray_origin: Vec3A = ray.origin.into();
        let ray_dir: Vec3A = Vec3A::from(*ray.direction);

        for (plane, plane_transform) in planes.iter(){
            if let Some(distance) = plane.ray_intersection(plane_transform.translation, plane_transform.scale, ray_origin, ray_dir){
                if distance > 0.0 {
                    let position: Vec3 = (ray_origin + ray_dir * distance).into();
                    worldpos.set(position);
                    return;
                }
            }
        }
    }
}
