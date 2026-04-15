
use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy_pg_core::prelude::GameStatePlay;
use bevy::color::palettes::tailwind::GRAY_500;
use bevy::color::palettes::css::{WHITE, BLACK};
use bevy_pg_scenes::maps::LoadMap;
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputSettings, 
    TextInputTextColor, TextInputTextFont, TextInputValue,
    TextInputPlaceholder, TextInputSubmitMessage
};
use bevy::feathers::theme::{ThemeBackgroundColor, ThemedText, UiTheme};
use bevy::ui_widgets::{
    slider_self_update, Activate, RadioButton, RadioGroup, 
    SliderStep, SliderValue, SliderPrecision, ValueChange, observe
};
use bevy::feathers::controls::{
    ButtonProps, SliderProps, ColorSliderProps, ColorChannel, ColorSwatch,  
    button, checkbox, radio, slider, color_slider, color_swatch, ColorSlider, SliderBaseColor
};
use bevy::ui::Checked;

use bevy_pg_scenes::prelude::{LoadPlaneScene, LoadTerrainPlane};

use crate::EditorSettings;
use crate::editor_pointer::EditorPointer;
use crate::prelude::EditorMode;


pub struct PGEditorLoadPlanePlugin;

impl Plugin for PGEditorLoadPlanePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, spawn_load_plane_popup.run_if(in_state(GameStatePlay::Editor).and(input_just_pressed(KeyCode::KeyL))))
        .add_systems(Update, read_plane_to_load_on_submit.run_if(in_state(GameStatePlay::Editor).and(on_message::<TextInputSubmitMessage>)))
        ;
    }
}



fn read_plane_to_load_on_submit(
    mut msgs:     MessageReader<TextInputSubmitMessage>,
    mut commands: Commands,
    forms:        Query<&TextInputValue, With<LoadPlaneTextInput>>,
    query:        Query<Entity, With<LoadPlanePopup>>,
    editor_pointer: Res<EditorPointer>,
    load_terrain_toggle: Single<&LoadTerrainToggle>,
    load_scene_toggle: Single<&LoadSceneToggle>,
    load_on_pointer: Single<&LoadOnPointerToggle>,
){
    info!("Read plane to load on submit: {} {}", load_terrain_toggle.0, load_scene_toggle.0 );
    for msg in msgs.read(){
        if let Ok(value) = forms.get(msg.entity){

            if value.0.contains("map:"){
                // Read and load from config map
                let map_name = value.0.replace("map:", "");
                let full_map_path: String  = format!("scenes/maps/{}.map.json", map_name);
                info!("Triggering {}", full_map_path);
                commands.trigger(
                    LoadMap{
                        map_path: full_map_path.clone(), 
                        maybe_loc: None, 
                        load_terrains: load_terrain_toggle.0,
                        load_scenes: load_scene_toggle.0,
                        for_editor: true
                    });

            } else {
                // Load a Plane and scene

                let mut spawn_loc: Option<Vec3> = None;
                if load_on_pointer.0 {
                    if let Some(world_pos) = editor_pointer.center_screen_ypos {
                        spawn_loc = Some(world_pos);
                    }
                }
                if load_terrain_toggle.0 {
                    let full_terrain_path: String  = format!("scenes/terrains/{}.mesh.json", value.0);
                    info!("Triggering {}", full_terrain_path);
                    commands.trigger(LoadTerrainPlane{mesh_path: full_terrain_path.clone(), maybe_loc: spawn_loc, for_editor: true});
                }
                if load_scene_toggle.0 {
                    let full_scene_path: String  = format!("scenes/scenes/{}.scene.json", value.0);
                    info!("Triggering {}", full_scene_path);
                    commands.trigger(LoadPlaneScene{scene_path: full_scene_path.clone(), maybe_loc: spawn_loc, for_editor: true});                    
                }

            }

        } 
        for entity in query.iter(){
            commands.entity(entity).despawn();
        }
    }
}


#[derive(Component)]
struct LoadPlanePopup;

#[derive(Component)]
struct LoadPlaneTextInput;

#[derive(Component)]
struct LoadTerrainToggle(bool);

#[derive(Component)]
struct LoadSceneToggle(bool);

#[derive(Component)]
struct LoadOnPointerToggle(bool);



fn spawn_load_plane_popup(
    mut commands:    Commands,
    editor_settings: Res<EditorSettings>,
    query:           Query<Entity, With<LoadPlanePopup>>
){

    if editor_settings.mode != EditorMode::Plane{
        return;
    }

    if !query.is_empty(){
        return;
    }

    let root = commands.spawn(
        (
            LoadPlanePopup,
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: percent(35.0),
                top: percent(45.0),
                width: px(600.0),
                height: px(100.0),
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
        )
    ).id();


    let text_input = commands.spawn(
        (
            Node {
                width: Val::Px(500.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(2.0)),
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::from(BLACK)),
            BackgroundColor(WHITE.into()),
            TextInput,
            LoadPlaneTextInput,
            TextInputPlaceholder{value: "".to_string(), ..default()},
            TextInputTextFont(TextFont {
                font_size: 17.0,
                ..default()
            }),
            TextInputSettings{
                retain_on_submit: true,
                mask_character: None,
                max_length: Some(100)
            },
            TextInputTextColor(TextColor(BLACK.into())),
            TextInputInactive(true),
        )
    ).id();


    let entity1: Entity = commands.spawn(checkbox((Checked, LoadTerrainToggle(true)), Spawn((Text::new("Load Terrain"), ThemedText)))).id();
    commands.entity(entity1).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                    checkbox.insert(LoadTerrainToggle(true));
                } else {
                    checkbox.remove::<Checked>();
                    checkbox.insert(LoadTerrainToggle(false));
                }
            }
        )   
    );

    let entity2: Entity = commands.spawn(checkbox((Checked, LoadSceneToggle(true)), Spawn((Text::new("Load Scene"), ThemedText)))).id();
    commands.entity(entity2).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                    checkbox.insert(LoadSceneToggle(true));
                } else {
                    checkbox.remove::<Checked>();
                    checkbox.insert(LoadSceneToggle(false));
                }
            }
        )   
    );

    let entity3: Entity = commands.spawn(checkbox((LoadOnPointerToggle(false)), Spawn((Text::new("Use Editor Pointer"), ThemedText)))).id();
    commands.entity(entity3).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                    checkbox.insert(LoadOnPointerToggle(true));
                } else {
                    checkbox.remove::<Checked>();
                    checkbox.insert(LoadOnPointerToggle(false));
                }
            }
        )   
    );


    commands.entity(root).add_children(&[text_input, entity1, entity2, entity3]);

}

