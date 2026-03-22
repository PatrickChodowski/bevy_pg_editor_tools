
use bevy_enhanced_input::prelude::ContextActivity;
use bevy::{color::palettes::tailwind::GRAY_800, input::common_conditions::input_just_pressed};
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy_simple_text_input::{
    TextInputPlugin, TextInputSystem,
    TextInputInactive, TextInput, TextInputTextFont, TextInputTextColor, TextInputSettings
};

use bevy_pg_core::prelude::{FlyCamController, GameStatePlay};

use crate::controller::EditorController;

pub struct PGEditorTextInputs;

#[derive(Component)]
pub(crate) struct LocInput;

#[derive(Component)]
pub(crate) struct LocInputX;

#[derive(Component)]
pub(crate) struct LocInputY;

#[derive(Component)]
pub(crate) struct LocInputZ;


pub(crate) fn loc_input_field(
    font_size: f32, 
    border_size: f32, 
    padding: f32, 
    margin: f32, 
) -> impl Bundle {

    let input_node_width: f32 = 60.0;

    (
        Node {
            width: Val::Percent(90.0),
            border: UiRect::all(Val::Px(border_size)),
            padding: UiRect::all(Val::Px(padding)),
            margin: UiRect::all(Val::Px(margin)),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        LocInput,
        BorderColor::all(Color::from(BLACK)),
        BackgroundColor(GRAY_800.into()),
        children![
            (
                Node {
                    width: Val::Px(input_node_width),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::from(BLACK)),
                BackgroundColor(WHITE.into()),
                TextInput,
                LocInputX,
                TextInputTextFont(TextFont {
                    font_size: font_size,
                    ..default()
                }),
                TextInputSettings{
                    retain_on_submit: true,
                    mask_character: None,
                    max_length: Some(5)
                },
                TextInputTextColor(TextColor(BLACK.into())),
                TextInputInactive(true),
            ),
            (
                Node {
                    width: Val::Px(input_node_width),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::from(BLACK)),
                BackgroundColor(WHITE.into()),
                TextInput,
                LocInputY,
                TextInputTextFont(TextFont {
                    font_size: font_size,
                    ..default()
                }),
                TextInputSettings{
                    retain_on_submit: true,
                    mask_character: None,
                    max_length: Some(5)
                },
                TextInputTextColor(TextColor(BLACK.into())),
                TextInputInactive(true),
            ),
            (
                Node {
                    width: Val::Px(input_node_width),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::from(BLACK)),
                BackgroundColor(WHITE.into()),
                TextInput,
                LocInputZ,
                TextInputTextFont(TextFont {
                    font_size: font_size,
                    ..default()
                }),
                TextInputSettings{
                    retain_on_submit: true,
                    mask_character: None,
                    max_length: Some(5)
                },
                TextInputTextColor(TextColor(BLACK.into())),
                TextInputInactive(true),
            )
        ]
    )
}




pub fn text_input_field(
    top: f32, 
    width: f32, 
    font_size: f32, 
    border_size: f32, 
    padding: f32, 
    margin: f32, 
    max_length: u16,
    retain_on_submit: bool
) -> impl Bundle {
    (
        Node {
            top: Val::Percent(top),
            width: Val::Px(width),
            border: UiRect::all(Val::Px(border_size)),
            padding: UiRect::all(Val::Px(padding)),
            margin: UiRect::all(Val::Px(margin)),
            ..default()
        },
        BorderColor::all(Color::from(BLACK)),
        BackgroundColor(WHITE.into()),
        TextInput,
        TextInputTextFont(TextFont {
            font_size: font_size,
            ..default()
        }),
        TextInputSettings{
            retain_on_submit: retain_on_submit,
            mask_character: None,
            max_length: Some(max_length)
        },
        TextInputTextColor(TextColor(BLACK.into())),
        TextInputInactive(true),
    )
}



impl Plugin for PGEditorTextInputs {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(TextInputPlugin)
        .add_systems(Update,
            (
                (
                    focus.run_if(input_just_pressed(MouseButton::Left)),
                    switch_controllers_on_text_input_active
                ).chain().before(TextInputSystem),
            ).run_if(in_state(GameStatePlay::Editor))
        )
        ;
    }
}


fn focus(
    hover_map: Res<HoverMap>,
    mut query:     Query<(Entity, &mut TextInputInactive, &mut BorderColor)>
){
    let hit_data = hover_map.0.get(&PointerId::Mouse).unwrap();
    if hit_data.len() > 0 {
        let hit_entities: Vec<Entity> = hit_data.keys().cloned().collect::<Vec<Entity>>();
        for (text_input_entity, mut inactive, mut border_color) in query.iter_mut(){
            let mut found_entity: bool = false;
            for entity in hit_entities.iter(){
                if *entity == text_input_entity {
                    found_entity = true;
                    break; 
                }
            }
            inactive.0 = !found_entity;

            if inactive.0 == false {
                *border_color = BorderColor::all(GREY);
            } else {
                *border_color = BorderColor::all(BLACK);
            }

        }
    }
}


#[derive(Resource)]
struct TextFocused(bool);

impl Default for TextFocused {
    fn default() -> Self {
        TextFocused(false)
    }
}

fn switch_controllers_on_text_input_active(
    mut commands:       Commands,
    text_inputs:        Query<&TextInputInactive>,
    camera_controller:  Single<Entity, With<FlyCamController>>,
    editor_controller:  Single<Entity, With<EditorController>>,
    mut text_focused:   Local<TextFocused>
){
    
    let mut any_text_input_active: bool = false;
    for inactive in text_inputs.iter(){
        if inactive.0 == false {
            any_text_input_active = true;
            break;
        }
    }

    match (any_text_input_active, text_focused.0) {
        (true, true) => {}
        (false, false) => {}
        (true, false) => {
            text_focused.0 = true;
            commands.entity(*camera_controller).insert(ContextActivity::<FlyCamController>::INACTIVE);
            commands.entity(*editor_controller).insert(ContextActivity::<EditorController>::INACTIVE);
        }
        (false, true) => {
            text_focused.0 = false;
            commands.entity(*camera_controller).insert(ContextActivity::<FlyCamController>::ACTIVE);
            commands.entity(*editor_controller).insert(ContextActivity::<EditorController>::ACTIVE);
        }
    }
}