use bevy::prelude::*;
use bevy::feathers::*;
use bevy::feathers::controls::{
    ButtonProps, SliderProps, ButtonVariant, ColorSliderProps, ColorChannel, ColorSwatch,  
    button, checkbox, radio, slider, color_slider, color_swatch, ColorSlider, SliderBaseColor
};
use bevy::feathers::theme::{ThemeBackgroundColor, ThemedText, UiTheme};
use bevy::ui::Checked;
use bevy::ui_widgets::{
    slider_self_update, Activate, RadioButton, RadioGroup, 
    SliderStep, SliderValue, SliderPrecision, ValueChange, observe
};
use bevy::feathers::dark_theme::create_dark_theme;
use bevy_pg_core::prelude::GameStatePlay;

use crate::controller::SerializePlane;
use crate::prelude::{
    ToggleMarkersVis, ToggleMultiGhost, ToggleSnapNav, EditorSettings, 
    ToggleSpawnersVis, ToggleGhostAxis, ToggleGhostMode, GhostTransformAxis, 
    GhostTransformMode, ChangeBrush, ChangeEditorMode, SaveScene, NavMeshGeneration, 
    EditorMode, UnghostAll, TriggerThumbnails, SpawnPlane
};

pub struct PGEditorUIPlugin;

impl Plugin for PGEditorUIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameStatePlay::Editor), init_editor_ui)
        .add_systems(Update, update_colors.run_if(in_state(GameStatePlay::Editor)))
        ;
    }
}

#[derive(Component)]
pub struct EditorControls;


#[derive(Component)]
pub struct SceneControls;

#[derive(Component)]
pub struct BrushControls;

#[derive(Component)]
pub struct PlaneControls;

#[derive(Component)]
struct RadioButtonAxis {
    value: GhostTransformAxis
}
#[derive(Component)]
struct RadioButtonMode {
    value: GhostTransformMode
}

#[derive(Component)]
struct RadioButtonBrush {
    value: usize
}

#[derive(Component, Clone, Copy)]
struct RadioButtonEditorMode {
    value: EditorMode
}


#[derive(Component)]
pub struct EditorControlsPanel;

fn init_editor_ui(
    mut commands:       Commands,
    editor_settings:    Res<EditorSettings>
){
    info!(" [EDITOR] Init UI");
    commands.insert_resource(UiTheme(create_dark_theme()));
    let root = commands.spawn((
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            padding: UiRect::all(px(8)),
            row_gap: px(8),
            column_gap: px(8),
            left: px(0.0),
            top: px(0.0),
            width:px(300.0),
            height: px(980.0),
            ..default()
        },
        DespawnOnExit(GameStatePlay::Editor),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(5.0)),
        EditorControlsPanel,
        Pickable::default()
    )).id();

    let children: Vec<Entity> = vec![
        label_section(&mut commands, "Editor Mode"),
        editor_mode_radio(&mut commands, &editor_settings),
        empty_row(&mut commands, 15.0),
        label_section(&mut commands, "Scene Settings"),
        ghost_checkboxes1(&mut commands, &editor_settings),
        ghost_checkboxes2(&mut commands, &editor_settings),
        value_scale_slider(&mut commands, &editor_settings),
        ghost_transform_radio(&mut commands, &editor_settings),
        ghost_transform_mode(&mut commands, &editor_settings),
        ghost_buttons(&mut commands),
        empty_row(&mut commands, 15.0),
        label_section(&mut commands, "Brush Settings"),
        brush_radio(&mut commands, &editor_settings),
        brush_radius_slider(&mut commands, &editor_settings),
        terrain_color(&mut commands, &editor_settings),
        empty_row(&mut commands, 15.0),
        label_section(&mut commands, "Plane Settings"),
        new_plane_settings(&mut commands, &editor_settings),
        empty_row(&mut commands, 40.0),
        buttons(&mut commands),
        empty_row(&mut commands, 10.0),
        other_buttons(&mut commands)
    ];
    commands.entity(root).add_children(&children);

    // To set enable/disable on widget
    commands.trigger(ChangeEditorMode{value: editor_settings.mode});

}

#[derive(Component)]
struct TerrainColorWidget;

#[derive(Resource)]
struct ColorWidgetState {
    hsl_color: Hsla,
}

fn update_colors(
    colors: Res<ColorWidgetState>,
    mut sliders: Query<(Entity, &ColorSlider, &mut SliderBaseColor)>,
    swatches: Query<&Children, With<ColorSwatch>>,
    mut commands: Commands,
    mut editor_settings: ResMut<EditorSettings>
) {
    if colors.is_changed() {
        for (slider_ent, slider, mut base) in sliders.iter_mut() {
            match slider.channel {
                ColorChannel::HslHue => {
                    base.0 = colors.hsl_color.into();
                    commands
                        .entity(slider_ent)
                        .insert(SliderValue(colors.hsl_color.hue));
                }
                ColorChannel::HslSaturation => {
                    base.0 = colors.hsl_color.into();
                    commands
                        .entity(slider_ent)
                        .insert(SliderValue(colors.hsl_color.saturation));
                }
                ColorChannel::HslLightness => {
                    base.0 = colors.hsl_color.into();
                    commands
                        .entity(slider_ent)
                        .insert(SliderValue(colors.hsl_color.lightness));
                }
                _ => {}
            }
        }
        for children in swatches.iter() {
            let clr: Color = colors.hsl_color.into();
            commands.entity(children[0]).insert(BackgroundColor(clr));
            editor_settings.color = clr;
            // Restart brush
            commands.trigger(ChangeBrush{value: editor_settings.brush_id});
        }
    }
}

fn editor_mode_radio(
    commands: &mut Commands,
    editor_settings:    &Res<EditorSettings>
) -> Entity {

    let data: Vec<(EditorMode, &str)> = vec![
        (EditorMode::Scene, "Scene"),
        (EditorMode::Brushes, "Brushes"),
        (EditorMode::Plane, "Plane"),
    ];

    let mut radios: Vec<Entity> = Vec::with_capacity(3);
    for (editor_mode, label) in data.iter(){
        if editor_settings.mode == *editor_mode {
            radios.push(commands.spawn(radio((Checked, RadioButtonEditorMode{value: *editor_mode}), Spawn((Text::new(*label), ThemedText)))).id());
        } else {
            radios.push(commands.spawn(radio(RadioButtonEditorMode{value: *editor_mode}, Spawn((Text::new(*label), ThemedText)))).id());
        }
    }

    let radio_group = commands.spawn((
        Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                row_gap: px(4),
                ..default()
            },
            RadioGroup,
            observe(
                |value_change: On<ValueChange<Entity>>,
                q_radio: Query<(Entity, &RadioButtonEditorMode), With<RadioButton>>,
                mut commands: Commands| {
                    for (entity, radio) in q_radio.iter() {
                        if entity == value_change.value {
                            commands.trigger(ChangeEditorMode{value: radio.value});
                            commands.entity(entity).insert(Checked);
                        } else {
                            commands.entity(entity).remove::<Checked>();
                        }
                    }
            }
        )
    )).id();

    commands.entity(radio_group).add_children(&radios);


    let local_root = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        children![
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    row_gap: px(4),
                    ..default()
                },
                RadioGroup,
                observe(
                    |value_change: On<ValueChange<Entity>>,
                    q_radio: Query<(Entity, &RadioButtonEditorMode), With<RadioButton>>,
                    mut commands: Commands| {
                        for (entity, radio) in q_radio.iter() {
                            if entity == value_change.value {
                                commands.trigger(ChangeEditorMode{value: radio.value});
                                commands.entity(entity).insert(Checked);
                            } else {
                                commands.entity(entity).remove::<Checked>();
                            }
                        }
                    }
                ),
            )
        ]
    )).id();

    commands.entity(local_root).add_child(radio_group);
    return local_root;
}


fn label_section(commands: &mut Commands, text: &str) -> Entity {
    commands.spawn(
        (
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                column_gap: px(8),
                ..default()
            },
            children![
                (Text::new(text), ThemedText),
            ]
        )
    ).id()
}

fn empty_row(commands: &mut Commands, height: f32) -> Entity {
    commands.spawn(
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        column_gap: px(8),
        height: px(height),
        ..default()
    }).id()
}


fn terrain_color(
    commands:        &mut Commands,
    editor_settings: &Res<EditorSettings>
) -> Entity {

    commands.insert_resource(ColorWidgetState {
        hsl_color: editor_settings.color.into()
    });

    return commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        TerrainColorWidget,
        children![
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                EditorControls,
                BrushControls,
                children![color_swatch(()),]
            ),
            (
                color_slider(ColorSliderProps {value: 0.5,channel: ColorChannel::HslHue}, (BrushControls, EditorControls)),
                observe(|change: On<ValueChange<f32>>, mut color: ResMut<ColorWidgetState>| {color.hsl_color.hue = change.value; },)
            ),
            (
                color_slider(ColorSliderProps {value: 0.5,channel: ColorChannel::HslSaturation},(BrushControls, EditorControls)),
                observe(|change: On<ValueChange<f32>>, mut color: ResMut<ColorWidgetState>| {color.hsl_color.saturation = change.value;},)
            ),
            (
                color_slider(ColorSliderProps {value: 0.5,channel: ColorChannel::HslLightness},(BrushControls, EditorControls)),
                observe(|change: On<ValueChange<f32>>, mut color: ResMut<ColorWidgetState>| {color.hsl_color.lightness = change.value;},)
            )
        ]
    )).id();
}

fn value_scale_slider(
    commands: &mut Commands,
    editor_settings: &Res<EditorSettings>
) -> Entity {
    return commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        SceneControls,
        children![
            (
                slider(
                    SliderProps {
                        max: 1.0,
                        value: editor_settings.change_value_scale,
                        min: 0.0,
                        ..default()
                    },
                    (SliderStep(0.01), SliderPrecision(2), SceneControls, EditorControls),
                ),
                observe(slider_self_update),
                observe(
                    |value_change: On<ValueChange<f32>>, mut editor_settings: ResMut<EditorSettings>| {
                        editor_settings.change_value_scale = value_change.value ;
                    }
                )
            ),
        ]
    )).id();
}


fn brush_radio(
    commands: &mut Commands,
    editor_settings: &Res<EditorSettings>
) -> Entity {
    let local_root = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        BrushControls
    )).id();

    let radio_group = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        RadioGroup,
        observe(
            |value_change: On<ValueChange<Entity>>,
            q_radio: Query<(Entity, &RadioButtonBrush), With<RadioButton>>,
            mut commands: Commands| {
                for (entity, radio) in q_radio.iter() {
                    if entity == value_change.value {
                        commands.trigger(ChangeBrush{value: radio.value});
                        commands.entity(entity).insert(Checked);
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            }
        ),
    )).id();

    let mut radios: Vec<Entity> = Vec::with_capacity(3);
    for (editor_brush_id, label) in editor_settings.brush_id_labels.iter(){
        if editor_settings.brush_id == *editor_brush_id {
            radios.push(commands.spawn(radio((Checked, RadioButtonBrush{value: *editor_brush_id}, BrushControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        } else {
            radios.push(commands.spawn(radio((RadioButtonBrush{value: *editor_brush_id}, BrushControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        }
    }
    commands.entity(radio_group).add_children(&radios);
    commands.entity(local_root).add_child(radio_group);
    return local_root;
}


fn ghost_transform_radio(commands: &mut Commands, editor_settings: &Res<EditorSettings>) -> Entity {

    let data: Vec<(GhostTransformAxis, &str)> = vec![
        (GhostTransformAxis::X, "X"),
        (GhostTransformAxis::Y, "Y"),
        (GhostTransformAxis::Z, "Z"),
        (GhostTransformAxis::OriginY, "OriginY"),
        (GhostTransformAxis::All, "All"),
        (GhostTransformAxis::XZ, "XZ"),
        (GhostTransformAxis::XY, "XY"),
    ];

    let mut radios: Vec<Entity> = Vec::with_capacity(7);
    for (value, label) in data.iter(){
        if editor_settings.ghost_transform_axis == *value {
            radios.push(commands.spawn(radio((Checked, RadioButtonAxis{value: *value}, SceneControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        } else {
            radios.push(commands.spawn(radio((RadioButtonAxis{value: *value}, SceneControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        }
    };

    let radio_group = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            row_gap: px(4),
            ..default()
        },
        SceneControls,
        RadioGroup,
        observe(
            |value_change: On<ValueChange<Entity>>,
            q_radio: Query<(Entity, &RadioButtonAxis), With<RadioButton>>,
            mut commands: Commands| {
                for (entity, radio) in q_radio.iter() {
                    if entity == value_change.value {
                        commands.trigger(ToggleGhostAxis{value: radio.value});
                        commands.entity(entity).insert(Checked);
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            }
    ))).id();

    commands.entity(radio_group).add_children(&radios);

    let local_root = commands.spawn(
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        }
    ).id();

    commands.entity(local_root).add_child(radio_group);
    return local_root;
}

fn ghost_transform_mode(
    commands: &mut Commands, 
    editor_settings: &Res<EditorSettings>
) -> Entity {
    let data: Vec<(GhostTransformMode, &str)> = vec![
        (GhostTransformMode::Translation, "Translation"),
        (GhostTransformMode::Rotation, "Rotation"),
        (GhostTransformMode::Scale, "Scale"),
    ];

    let mut radios: Vec<Entity> = Vec::with_capacity(3);
    for (value, label) in data.iter(){
        if editor_settings.ghost_transform_mode == *value {
            radios.push(commands.spawn(radio((Checked, RadioButtonMode{value: *value}, SceneControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        } else {
            radios.push(commands.spawn(radio((RadioButtonMode{value: *value}, SceneControls, EditorControls), Spawn((Text::new(*label), ThemedText)))).id());
        }
    };

    let radio_group = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            row_gap: px(4),
            ..default()
        },
        SceneControls,
        RadioGroup,
        observe(
            |value_change: On<ValueChange<Entity>>,
            q_radio: Query<(Entity, &RadioButtonMode), With<RadioButton>>,
            mut commands: Commands| {
                for (entity, radio) in q_radio.iter() {
                    if entity == value_change.value {
                        commands.trigger(ToggleGhostMode{value: radio.value});
                        commands.entity(entity).insert(Checked);
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            }
    ))).id();

    commands.entity(radio_group).add_children(&radios);

    let local_root = commands.spawn(
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        }).id();
    
    commands.entity(local_root).add_child(radio_group);
    return local_root;

}

fn ghost_checkboxes1(
    commands: &mut Commands, 
    editor_settings: &Res<EditorSettings>
) -> Entity {

    let local_root = commands.spawn(
        (
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                column_gap: px(8),
                ..default()
            },
            SceneControls
        )
    ).id();

    let entity1: Entity = if editor_settings.show_spawners {
        commands.spawn(checkbox((Checked, SceneControls, EditorControls), Spawn((Text::new("Show Spawners"), ThemedText)))).id()
    } else {
        commands.spawn(checkbox((SceneControls, EditorControls), Spawn((Text::new("Show Spawners"), ThemedText)))).id()
    };
    commands.entity(entity1).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                commands.trigger(ToggleSpawnersVis{visible: change.value});
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                } else {
                    checkbox.remove::<Checked>();
                }
            }
        )   
    );


    let entity2 = if editor_settings.show_markers {
        commands.spawn(checkbox((Checked, SceneControls, EditorControls), Spawn((Text::new("Show Markers"), ThemedText)))).id()
    } else {
        commands.spawn(checkbox((SceneControls, EditorControls), Spawn((Text::new("Show Markers"), ThemedText)))).id()
    };
    commands.entity(entity2).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                commands.trigger(ToggleMarkersVis{visible: change.value});
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                } else {
                    checkbox.remove::<Checked>();
                }
            }
        )   
    );

    commands.entity(local_root).add_children(&vec![entity1, entity2]);
    return local_root;

}



fn ghost_checkboxes2(
    commands: &mut Commands, 
    editor_settings: &Res<EditorSettings>
) -> Entity {

    let local_root = commands.spawn(
        (
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                column_gap: px(8),
                ..default()
            },
            SceneControls
        )
    ).id();

    let entity3 = if editor_settings.snap_nav {
        commands.spawn(checkbox((Checked, SceneControls, EditorControls), Spawn((Text::new("Snap To NavMesh"), ThemedText)))).id()
    } else {
        commands.spawn(checkbox((SceneControls, EditorControls), Spawn((Text::new("Snap To NavMesh"), ThemedText)))).id()
    };
    commands.entity(entity3).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                commands.trigger(ToggleSnapNav{value: change.value});
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                } else {
                    checkbox.remove::<Checked>();
                }
            }
        )   
    );

    let entity4 = if editor_settings.multi_ghost {
        commands.spawn(checkbox((Checked, SceneControls, EditorControls), Spawn((Text::new("MultiGhost"), ThemedText)))).id()
    } else {
        commands.spawn(checkbox((SceneControls, EditorControls), Spawn((Text::new("MultiGhost"), ThemedText)))).id()
    };
    commands.entity(entity4).insert(
        observe(
            |change: On<ValueChange<bool>>, mut commands: Commands| {
                commands.trigger(ToggleMultiGhost{value: change.value});
                let mut checkbox = commands.entity(change.source);
                if change.value {
                    checkbox.insert(Checked);
                } else {
                    checkbox.remove::<Checked>();
                }
            }
        )   
    );

    commands.entity(local_root).add_children(&vec![entity3, entity4]);
    return local_root;

}


fn buttons(commands: &mut Commands) -> Entity {
    commands.spawn(
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        children![
            (
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new("Navmesh Generation"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(NavMeshGeneration);
                })       
            ),
            (
                button(
                    ButtonProps {
                        variant: ButtonVariant::Primary,
                        ..default()
                    },
                    (),
                    Spawn((Text::new("Save Scene"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(SaveScene);
                })
            ),
        ]
    )).id()
}

fn ghost_buttons(
    commands: &mut Commands
) -> Entity {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            max_width: px(250.0),
            ..default()
        },
        SceneControls,
        children![
            (
                button(
                    ButtonProps::default(),
                    (SceneControls, EditorControls),
                    Spawn((Text::new("Unghost All"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(UnghostAll);
                })       
            )
        ]
    )).id()
}


fn other_buttons(
    commands: &mut Commands
) -> Entity {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            max_width: px(250.0),
            ..default()
        },
        children![
            (
                button(
                    ButtonProps::default(),(),
                    Spawn((Text::new("Trigger Thumbnails"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(TriggerThumbnails);
                })       
            )
        ]
    )).id()
}


fn brush_radius_slider(
    commands: &mut Commands,
    editor_settings: &Res<EditorSettings>
) -> Entity {
    commands.spawn(
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            ..default()
        },
        BrushControls,
        children![
            (
                slider(
                    SliderProps {
                        max: 100.0,
                        value: editor_settings.brush_radius,
                        min: 0.0,
                        ..default()
                    },
                    (SliderStep(1.0), SliderPrecision(0), BrushControls, EditorControls),
                ),
                observe(slider_self_update),
                observe(
                    |value_change: On<ValueChange<f32>>, mut editor_settings: ResMut<EditorSettings>| {
                        editor_settings.brush_radius = value_change.value;
                    }
                )
            ),
        ]
    )).id()
}


fn new_plane_settings(
    commands: &mut Commands,
    editor_settings: &Res<EditorSettings>
) -> Entity {

    let local_root = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            row_gap: px(5.0),
            ..default()
        },
        children![
            (Text::new("Width")),
            (
                slider(
                    SliderProps {
                        max: 100.0,
                        value: editor_settings.plane_width,
                        min: 0.0,
                        ..default()
                    },
                    (SliderStep(1.0), SliderPrecision(0), PlaneControls, EditorControls),
                ),
                observe(slider_self_update),
                observe(
                    |value_change: On<ValueChange<f32>>, mut editor_settings: ResMut<EditorSettings>| {
                        editor_settings.plane_width = value_change.value;
                    }
                )
            ),
            (Text::new("Height")),
            (
                slider(
                    SliderProps {
                        max: 100.0,
                        value: editor_settings.plane_height,
                        min: 0.0,
                        ..default()
                    },
                    (SliderStep(1.0), SliderPrecision(0), PlaneControls, EditorControls),
                ),
                observe(slider_self_update),
                observe(
                    |value_change: On<ValueChange<f32>>, mut editor_settings: ResMut<EditorSettings>| {
                        editor_settings.plane_height = value_change.value;
                    }
                )
            ),
            (Text::new("Subdivisions")),
            (
                slider(
                    SliderProps {
                        max: 50.0,
                        value: editor_settings.plane_subdivisions as f32,
                        min: 0.0,
                        ..default()
                    },
                    (SliderStep(1.0), SliderPrecision(0), PlaneControls, EditorControls),
                ),
                observe(slider_self_update),
                observe(
                    |value_change: On<ValueChange<f32>>, mut editor_settings: ResMut<EditorSettings>| {
                        editor_settings.plane_subdivisions = value_change.value as u32;
                    }
                )
            ),
            (
                button(
                    ButtonProps::default(),(PlaneControls, EditorControls),
                    Spawn((Text::new("Spawn Plane"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(SpawnPlane);
                })    
            ),
            (
                button(
                    ButtonProps {
                        variant: ButtonVariant::Primary,
                        ..default()
                    },(PlaneControls, EditorControls),
                    Spawn((Text::new("Serialize Planes"), ThemedText))
                ),
                observe(|_activate: On<Activate>, mut commands: Commands| {
                    commands.trigger(SerializePlane);
                })    
            )
        ]
    )).id();


    return local_root;

}