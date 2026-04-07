
use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy_pg_core::prelude::GameStatePlay;
use bevy::color::palettes::tailwind::GRAY_500;
use bevy::color::palettes::css::{WHITE, BLACK};
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputSettings, 
    TextInputSystem, TextInputTextColor, TextInputTextFont, TextInputValue,
    TextInputPlaceholder, TextInputSubmitMessage
};

use crate::EditorSettings;
use crate::prelude::EditorMode;


pub struct PGEditorLoadPlanePlugin;

impl Plugin for PGEditorLoadPlanePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, spawn_load_plane.run_if(in_state(GameStatePlay::Editor).and(input_just_pressed(KeyCode::KeyL))))
        .add_systems(Update, read_plane_to_load_on_submit.run_if(in_state(GameStatePlay::Editor).and(on_message::<TextInputSubmitMessage>)))
        ;
    }
}



fn read_plane_to_load_on_submit(
    mut msgs:     MessageReader<TextInputSubmitMessage>,
    mut commands: Commands,
    forms:        Query<&TextInputValue, With<LoadPlaneTextInput>>,
    query:        Query<Entity, With<LoadPlanePopup>>
){
    for msg in msgs.read(){
        if let Ok(value) = forms.get(msg.entity){
            info!("Spawn Plane scene: {}", value.0);
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

fn spawn_load_plane(
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

    commands.entity(root).add_child(text_input);

}

