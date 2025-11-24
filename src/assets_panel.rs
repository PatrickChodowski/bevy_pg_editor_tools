
use std::fs;
use bevy::color::palettes::css::*;
use bevy_enhanced_input::prelude::ContextActivity;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::picking::hover::HoverMap;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::pointer::PointerId;
use bevy_seedling::sample::OnComplete;
use bevy_simple_text_input::{
    TextInput, TextInputPlugin, TextInputSystem, TextInputTextColor,
    TextInputTextFont, TextInputInactive, TextInputValue
};
use bevy_seedling::prelude::{SamplePlayer, Volume, PlaybackSettings};
use bevy_pg_core::prelude::{FlyCamController, GameStatePlay};

use crate::ghost::EditorSpawnAsset;
use crate::ghost::EditorAsset;
use crate::controller::EditorController;
use crate::ghost::EditorGhostTransformMemory;

const IMG_DIM_FOCUS: f32 = 170.0;
const IMG_DIM: f32 = 150.0;
const IMG_MARGIN: f32 = 10.0;
const IMG_MARGIN_FOCUS: f32 = 0.0;

pub struct PGEditorAssetsPanelPlugin;


impl Plugin for PGEditorAssetsPanelPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(TextInputPlugin)
        .add_systems(OnEnter(GameStatePlay::Editor), init)
        .add_systems(OnExit(GameStatePlay::Editor), clear)
        .add_systems(Update,
            (
                update_scroll_position,
                (
                    (
                        focus.run_if(input_just_pressed(MouseButton::Left)),
                        activate_input.run_if(input_just_pressed(KeyCode::Space)),
                    ),
                    switch_controllers
                ).chain().before(TextInputSystem),
                update_assets_bar.after(TextInputSystem)
            ).run_if(in_state(GameStatePlay::Editor))
        )
        ;
    }
}

#[derive(Resource)]
struct AssetList {
    data: Vec<String>
}

fn update_assets_bar(
    mut commands:   Commands,
    asset_list:     Res<AssetList>,
    asset_filter:   Single<&TextInputValue, (With<AssetSearch>, Changed<TextInputValue>)>,
    asset_bar:      Single<Entity, With<AssetsBar>>,
    ass:            Res<AssetServer>
) {
    commands.entity(*asset_bar).despawn_related::<Children>();

    let filter_text = asset_filter.0.clone().replace(" ","").to_lowercase();
    let assets: Vec<String> = asset_list.data.clone().into_iter().filter(|x| x.to_lowercase().contains(&filter_text)).collect::<Vec<String>>();

    commands.entity(*asset_bar).with_children(|bar|{
        for file_name in assets.iter() {
            spawn_asset_button(bar, file_name.to_string(), &ass);  
        }
    });

}


// Emergency activate input if not active :)
fn activate_input(
    mut inactive: Single<&mut TextInputInactive, With<AssetSearch>>
){
    if inactive.0 {
        inactive.0 = false;
    }
}


fn focus(
    hover_map: Res<HoverMap>,
    query:     Single<(Entity, &mut TextInputInactive), With<AssetSearch>>
){
    let hit_data = hover_map.0.get(&PointerId::Mouse).unwrap();
    if hit_data.len() > 0 {
        let hit_entities: Vec<Entity> = hit_data.keys().cloned().collect::<Vec<Entity>>();

        let mut found_entity: bool = false;
        let (text_input_entity, mut inactive) = query.into_inner();
        for entity in hit_entities.iter(){
            if *entity == text_input_entity {
                found_entity = true;
                break; 
            }
        }
        inactive.0 = !found_entity;
    }
}


fn switch_controllers(
    mut commands:       Commands,
    inactive:           Single<&TextInputInactive, Changed<TextInputInactive>>,
    camera_controller:  Single<Entity, With<FlyCamController>>,
    editor_controller:  Single<Entity, With<EditorController>>,
){
    if inactive.0 {
        commands.entity(*camera_controller).insert(ContextActivity::<FlyCamController>::ACTIVE);
        commands.entity(*editor_controller).insert(ContextActivity::<EditorController>::ACTIVE);
    } else {
        commands.entity(*camera_controller).insert(ContextActivity::<FlyCamController>::INACTIVE);
        commands.entity(*editor_controller).insert(ContextActivity::<EditorController>::INACTIVE);
    }
}


#[derive(Component)]
pub struct EditorSounds;

fn init(
    mut commands: Commands,
    ass:          Res<AssetServer>,
){

    commands.spawn((
        EditorSounds, 
        DespawnOnExit(GameStatePlay::Editor)
    ));

    let panel = commands.spawn(vertical_right_panel()).id();
    let text_input = commands.spawn(text_panel()).id();

    let mut assets = list_assets();
    let spawner_list: Vec<&str> = vec!["Spawner_NPCs","Spawner_Boats","Spawner_Items"];
    let marker_list: Vec<&str> = vec!["Marker_Locations", "Marker_EntryPort", "Marker_ExitPort"]; // Things that are static, not visible, but have location, range, logic etc.

    assets.extend(spawner_list.iter().map(|a| a.to_string()));
    assets.extend(marker_list.iter().map(|a| a.to_string()));
    assets.sort();

    commands.insert_resource(AssetList{data: assets.clone()});
 
    let assets_bar = commands.spawn(assets_bar()).with_children(|bar| {
        for file_name in assets.iter() {
            spawn_asset_button(bar, file_name.to_string(), &ass);  
        }
    }).id();

    commands.entity(panel).add_children(&[text_input, assets_bar]);
}

fn clear(
    mut commands: Commands
){
    commands.remove_resource::<AssetList>();
}

const LINE_HEIGHT: f32 = 21.;

#[derive(Component)]
pub(super) struct EditorAssetPanel;

#[derive(Component)]
struct AssetButton {
    label: String,
    spawner: bool,
    marker: bool
}

#[derive(Component)]
struct AssetButtonLabel;

#[derive(Component)]
struct AssetsBar;

#[derive(Component)]
struct AssetSearch;


fn spawn_asset_button(
    builder:    &mut ChildSpawnerCommands, 
    file_name:  String,
    ass:        &Res<AssetServer>
){
    let mut path = format!("editor/{}", file_name.replace(".glb", ".png"));
    let spawner: bool = file_name.contains("Spawner_");
    let marker: bool = file_name.contains("Marker_");

    // For Spawners or markers
    if spawner | marker {
        path = format!("{}.png", path);
    }
    let image = ass.load(path);
    builder.spawn((
        Node {
            margin: UiRect::all(Val::Px(IMG_MARGIN)),
            align_content: AlignContent::Center,
            width: Val::Px(IMG_DIM),
            height: Val::Px(IMG_DIM),
            ..default()
        },
        ImageNode::new(image.clone()),
        BorderRadius::all(Val::Px(5.0)),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        AssetButton{label: file_name.replace(".glb", ""), spawner, marker}
    ))
    .observe(asset_button_over)
    .observe(asset_button_out)
    .observe(asset_button_pressed)
    ;
}

fn vertical_right_panel() -> impl Bundle {
    (
        Node {
            padding: UiRect::all(Val::Px(12.0)),
            display: Display::Flex,
            width: Val::Percent(16.0),
            height: Val::Percent(100.0),
            top: Val::Percent(0.0),
            left: Val::Percent(84.0),
            border: UiRect { left: Val::Px(5.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0)},
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        BackgroundColor(DARK_GRAY.into()),
        BorderColor::all(Color::from(BLACK)),
        EditorAssetPanel,
        Pickable{should_block_lower: true, ..default()},
        DespawnOnExit(GameStatePlay::Editor)
    )

}

fn text_panel() -> impl Bundle {
    (
        Node {
            top: Val::Percent(4.0),
            width: Val::Px(200.0),
            border: UiRect::all(Val::Px(5.0)),
            padding: UiRect::all(Val::Px(10.0)),
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BorderColor::all(Color::from(BLACK)),
        BackgroundColor(WHITE.into()),
        TextInput,
        TextInputTextFont(TextFont {
            font_size: 34.,
            ..default()
        }),
        AssetSearch,
        TextInputTextColor(TextColor(BLACK.into())),
        TextInputInactive(true),
    )
}



fn assets_bar() -> impl Bundle {
    (
        Node {
            top: Val::Percent(5.0),
            left: Val::Percent(5.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        Pickable {
            should_block_lower: false,
            ..default()
        },
        AssetsBar
    )
}




fn update_scroll_position(
    mut mouse_wheel_events:  MessageReader<MouseWheel>,
    hover_map:               Res<HoverMap>,
    mut scroll_position:     Single<&mut ScrollPosition, With<AssetsBar>>,
    panel:                   Query<&EditorAssetPanel>
){
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (_dx, dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (
                mouse_wheel_event.x * LINE_HEIGHT,
                mouse_wheel_event.y * LINE_HEIGHT,
            ),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };
        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(_panel) = panel.get(*entity){
                    scroll_position.0.y += dy*4.0;
                }
            }
        }
    }

}


fn asset_button_over(
    trigger:      On<Pointer<Over>>,
    mut query:    Query<(&mut Node, &AssetButton)>,
    mut commands: Commands,
    ass:          Res<AssetServer>,
    sound_entity: Single<Entity, With<EditorSounds>>
){
    let entity = trigger.entity;
    if let Ok((mut node, assbutton)) = query.get_mut(entity){
        node.width = Val::Px(IMG_DIM_FOCUS);
        node.height = Val::Px(IMG_DIM_FOCUS);
        node.margin = UiRect::all(Val::Px(IMG_MARGIN_FOCUS));

        let _label_entity = commands.spawn((
            Text::new(assbutton.label.clone()),
            TextColor(BLACK.into()),
            Node{
                position_type: PositionType::Absolute,
                top: Val::Percent(1.0),
                left: Val::Percent(85.0),
                ..default()
            },
            AssetButtonLabel,
            DespawnOnExit(GameStatePlay::Editor)
        )).id();

        commands.entity(entity).insert(
            BoxShadow::new(
                Color::BLACK.with_alpha(0.8),
                Val::Percent(5.0),
                Val::Percent(5.0),
                Val::ZERO,
                Val::Percent(10.0)
            ),
        );

        commands.entity(*sound_entity).insert((
            SamplePlayer::new(ass.load(format!("sounds/items/metal-small1.ogg"))).with_volume(Volume::Linear(0.25)),
            PlaybackSettings::default().with_on_complete(OnComplete::Remove)
        ));
    }
}

fn asset_button_out(
    trigger:        On<Pointer<Out>>,
    mut query:      Query<&mut Node, With<AssetButton>>,
    button_label:   Query<Entity, With<AssetButtonLabel>>,
    mut commands:   Commands
){
    let entity = trigger.entity;
    if let Ok(mut node) = query.get_mut(entity){
        node.width = Val::Px(IMG_DIM);
        node.height = Val::Px(IMG_DIM);
        node.margin = UiRect::all(Val::Px(IMG_MARGIN));
    }
    for label_entity in button_label.iter(){
        commands.entity(label_entity).despawn();
    }
    commands.entity(entity).remove::<BoxShadow>();
}


fn asset_button_pressed(
    trigger:        On<Pointer<Press>>,
    query:           Query<&AssetButton>,
    mut writer:      MessageWriter<EditorSpawnAsset>,
    keys:            Res<ButtonInput<KeyCode>>,
    transform_memo:  Option<Res<EditorGhostTransformMemory>>
){
 
    let mut rotation: Option<Quat> = None;
    let mut scale: Option<Vec3> = None;

    if let Some(transform_memo) = transform_memo {
        for key in keys.get_pressed() {
            match key {
                KeyCode::AltLeft => {
                    rotation = Some(transform_memo.rotation);
                    scale = Some(transform_memo.scale);
                }
                _ => {}
            }
        }
    }

    if let Ok(asset_button) = query.get(trigger.entity){

        let event = match (asset_button.spawner, asset_button.marker) {
            (false, false) => {
                EditorSpawnAsset::new(
                    EditorAsset::Asset(asset_button.label.clone()), 
                    None, rotation, scale
                )
            },
            (true, false) => {
                EditorSpawnAsset::new(
                    EditorAsset::Spawner(asset_button.label.clone()), 
                    None, rotation, scale
                )
            },
            (false, true) => {
                EditorSpawnAsset::new(
                    EditorAsset::Marker(asset_button.label.clone()), 
                    None, rotation, scale
                )
            },
            (true, true) => {panic!("Wrong Editor Button: spawner and marker")}
        };
        writer.write(event);
    }   
}

pub(super) fn list_assets() -> Vec<String>{
    let Ok(entries) = fs::read_dir("./assets/objects") else {return Vec::new();};
    let mut v: Vec<String> = Vec::new();
    for entry in entries {
        let Ok(raw_path) = entry else {continue;};
        let file_name = raw_path.file_name().into_string().ok().unwrap();

        if file_name.contains("Bld") | 
           file_name.contains("Wep") | 
           file_name.contains("Prop") | 
           file_name.contains("Veh") | 
           file_name.contains("Path") | 
           file_name.contains("Env"){
            v.push(file_name);
        }

    }

    return v;
}