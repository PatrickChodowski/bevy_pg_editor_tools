use bevy::prelude::*;
use bevy_enhanced_input::prelude::Fire;
use bevy_pg_core::prelude::GameStatePlay;

use crate::controller::SpawnHelpDisplay;

pub struct PGEditorControlsDisplayPlugin;


impl Plugin for PGEditorControlsDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_observer(toggle_help_display)
        ;
    }
}

#[derive(Component)]
struct HelpDisplay;

fn toggle_help_display(
    _trigger:     On<Fire<SpawnHelpDisplay>>,
    mut commands: Commands,
    query:        Query<Entity, With<HelpDisplay>>
){

    if let Ok(entity) = query.single() {
        commands.entity(entity).despawn();
    } else {

        commands.spawn((
            HelpDisplay,
            DespawnOnExit(GameStatePlay::Editor),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(10.0),
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::WHITE.with_alpha(0.5)),
            children![
                (
                    Node {
                        position_type: PositionType::Relative,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(15.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    children![
                        (Text(" Transform: ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (1) Translation ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (2) Rotation ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (3) Scale ".to_string()), TextColor(Color::BLACK)),
                        (Text(" Axis: ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (7) X ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (8) Y ".to_string()), TextColor(Color::BLACK)),
                        (Text("     (9) Z ".to_string()), TextColor(Color::BLACK)),
                        (Text(" Change Value: (<-) (->) ".to_string()), TextColor(Color::BLACK)),
                        (Text(" Change Scale: (^) (v) ".to_string()), TextColor(Color::BLACK))
                    ]
                ),
                (
                    Node {
                        position_type: PositionType::Relative,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(15.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    children![
                        (Text(" Right click -> Toggle Ghost ".to_string()), TextColor(Color::BLACK)),
                        (Text(" Left click+drag -> Drag around ".to_string()), TextColor(Color::BLACK)),
                        (Text(" DEL: -> Remove object ".to_string()), TextColor(Color::BLACK)),
                        (Text(" (L): -> Thumbnails ".to_string()), TextColor(Color::BLACK)),
                        (Text(" < >: Change Debug Animation ".to_string()), TextColor(Color::BLACK)),
                        (Text(" ENTER: Save Scene".to_string()), TextColor(Color::BLACK)),

                        (Text(" LeftShift: Toggle MultiGhost".to_string()), TextColor(Color::BLACK)),
                        (Text(" N: Toggle Snapping to NavMesh".to_string()), TextColor(Color::BLACK)),
                        (Text(" G: Generate NavMesh".to_string()), TextColor(Color::BLACK)),

                        (Text(" Z: Undo".to_string()), TextColor(Color::BLACK)),
                        (Text(" X: Redo".to_string()), TextColor(Color::BLACK)),
                        (Text(" SPACE: Focus on Asset filter".to_string()), TextColor(Color::BLACK)),
                    ]

                ),
                (
                    Node {
                        position_type: PositionType::Relative,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(15.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    children![
                        (Text(" Left Click + LeftControl: Copy Object(s) with Transform".to_string()), TextColor(Color::BLACK)),
                        (Text(" Left Click + T: Write to Transform Memory".to_string()), TextColor(Color::BLACK)),
                        (Text(" Left Click + LeftAlt: Spawn with Transform".to_string()), TextColor(Color::BLACK)),
                        (Text(" U: Unghost All".to_string()), TextColor(Color::BLACK)),
                        (Text(" M: Toggle Brush".to_string()), TextColor(Color::BLACK)),
                    ]
                )

            ]

        ));

        

    }
}

