// All credits go to https://github.com/jbuehler23/bevy/tree/transform-gizmo
// Slightly modified to deal with multiple entities in the same time

use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, Local};
use bevy::camera::Camera;
use bevy::color::{Color};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    reflect::{ReflectComponent, ReflectResource},
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Query, Res, ResMut, Single},
};
use bevy::input::{mouse::MouseButton, ButtonInput};
use bevy::math::{Quat, Vec2, Vec3, Ray3d};
use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window};
use bevy::platform::collections::HashMap;

use bevy_pg_core::prelude::{MainCamera, GameStatePlay};

use crate::prelude::Ghost;
use crate::tracker::{Changes, Change, ChangeTransform, ChangesSet};

pub const AXIS_LENGTH: f32 = 1.0;
pub const AXIS_TIP_LENGTH: f32 = 0.2;
pub const AXIS_START_OFFSET: f32 = 0.2;
pub const ROTATE_RING_RADIUS: f32 = 1.0;
pub const SCALE_CUBE_SIZE: f32 = 0.07;

pub const COLOR_X: Color = Color::srgb(1.0, 0.0, 0.49);
pub const COLOR_Y: Color = Color::srgb(0.0, 1.0, 0.49);
pub const COLOR_Z: Color = Color::srgb(0.0, 0.49, 1.0);
pub const COLOR_VIEW: Color = Color::WHITE;
pub const INACTIVE_ALPHA: f32 = 0.5;

const MIN_SCALE: f32 = 0.01;
pub const AXIS_HIT_DISTANCE: f32 = 35.0;
pub const SHAFT_RADIUS: f32 = 0.015;
pub const SHAFT_LENGTH: f32 = 0.6;
pub const CONE_RADIUS: f32 = 0.05;
pub const CONE_HEIGHT: f32 = 0.2;
pub const VIEW_CIRCLE_MINOR: f32 = 0.01;
pub const VIEW_CIRCLE_MAJOR: f32 = 0.15;
pub const VIEW_RING_MINOR: f32 = 0.01;
pub const VIEW_RING_MAJOR: f32 = 1.15;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component, Default)]
pub struct TransformGizmoFocus;

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Reflect)]
pub enum TransformGizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Reflect)]
pub enum TransformGizmoSpace {
    #[default]
    World,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Reflect)]
pub enum TransformGizmoAxis {
    X,
    Y,
    Z,
    /// The view-plane / view-axis (white).
    View
}

/// Exposes the current drag state so external systems (undo, UI) can observe it.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct TransformGizmoState {
    /// `true` while the user is actively dragging.
    pub active: bool,
    /// The axis under the cursor, if any.
    pub hovered_axis: Option<TransformGizmoAxis>,
    /// The axis being dragged, if any.
    pub axis: Option<TransformGizmoAxis>,
    /// Screen position where the drag started.
    pub drag_start_screen: Vec2,
    /// The transform snapshot taken when the drag started.
    pub data: HashMap<Entity, Transform>,
    /// World-space point (or normalized direction for rotation) where the drag started.
    pub drag_start_world: Vec3,
    /// World-space gizmo origin at drag start.
    pub gizmo_origin: Vec3,
}


/// Configuration and preferences for the transform gizmo.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TransformGizmoConfig {
    /// Which manipulation mode the gizmo is in.
    pub mode: TransformGizmoMode,
    /// Whether the gizmo transforms the object using world or local space axes.
    pub space: TransformGizmoSpace,
    /// Length of the axis handles.
    pub axis_length: f32,
    /// Radius of the rotation rings.
    pub rotate_ring_radius: f32,
    /// Screen-space pixel distance for hover detection.
    pub axis_hit_distance: f32,
    /// If set, translation snaps to this increment.
    pub snap_translate: Option<f32>,
    /// If set, rotation snaps to this increment (radians).
    pub snap_rotate: Option<f32>,
    /// If set, scale snaps to this increment.
    pub snap_scale: Option<f32>,
    /// Whether to confine the cursor during drag.
    pub confine_cursor: bool,
    /// Screen-space scale factor. Set to 0.0 to disable constant-size behavior.
    pub screen_scale_factor: f32,
}

impl Default for TransformGizmoConfig {
    fn default() -> Self {
        Self {
            mode: TransformGizmoMode::default(),
            space: TransformGizmoSpace::default(),
            axis_length: AXIS_LENGTH,
            rotate_ring_radius: ROTATE_RING_RADIUS,
            axis_hit_distance: AXIS_HIT_DISTANCE,
            snap_translate: None,
            snap_rotate: None,
            snap_scale: None,
            confine_cursor: true,
            screen_scale_factor: 0.1,
        }
    }
}

/// Marker component for the root entity of the gizmo mesh hierarchy.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TransformGizmoRoot;

/// Marker component for individual gizmo mesh parts.
#[derive(Component, Debug, Clone, Copy)]
pub struct TransformGizmoMeshMarker {
    /// Which axis this mesh part represents.
    pub axis: TransformGizmoAxis,
    /// Which mode this mesh part is used in.
    pub mode: TransformGizmoMode,
}


pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<TransformGizmoState>()
        .init_resource::<TransformGizmoConfig>()
        .register_type::<TransformGizmoFocus>()
        .register_type::<TransformGizmoMode>()
        .register_type::<TransformGizmoSpace>()
        .register_type::<TransformGizmoState>()
        .register_type::<TransformGizmoConfig>()
        .add_systems(
            Update,
            (
                transform_gizmo_hover,
                transform_gizmo_drag
            )
            .chain().run_if(in_state(GameStatePlay::Editor)),
        );
    }
}


fn transform_gizmo_hover(
    focus: Option<Single<&GlobalTransform, With<TransformGizmoFocus>>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    settings: Res<TransformGizmoConfig>,
    mut state: ResMut<TransformGizmoState>,
) {
    state.hovered_axis = None;

    if state.active {
        return;
    }

    let Some(global_tf) = focus else {
        return;
    };
    let (camera, cam_tf) = *camera;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let gizmo_pos = global_tf.translation();
    let space = effective_space(&settings);
    let rotation = gizmo_rotation(*global_tf, space);

    let scale = if settings.screen_scale_factor > 0.0 {
        (cam_tf.translation() - gizmo_pos).length() * settings.screen_scale_factor
    } else {
        1.0
    };

    let axes = [
        (TransformGizmoAxis::X, rotation * Vec3::X),
        (TransformGizmoAxis::Y, rotation * Vec3::Y),
        (TransformGizmoAxis::Z, rotation * Vec3::Z),
    ];

    let mut best_axis = None;
    let mut best_dist = f32::MAX;
    let threshold = settings.axis_hit_distance;

    for (axis, dir) in &axes {
        let dist = match settings.mode {
            TransformGizmoMode::Translate | TransformGizmoMode::Scale => {
                let start = gizmo_pos + *dir * (AXIS_START_OFFSET * scale);
                let endpoint = gizmo_pos + *dir * (settings.axis_length * scale);
                let Some(start_screen) = camera.world_to_viewport(cam_tf, start).ok() else {
                    continue;
                };
                let Some(end_screen) = camera.world_to_viewport(cam_tf, endpoint).ok() else {
                    continue;
                };
                point_to_segment_dist(cursor_pos, start_screen, end_screen)
            }
            TransformGizmoMode::Rotate => point_to_ring_screen_dist(
                cursor_pos,
                camera,
                cam_tf,
                gizmo_pos,
                *dir,
                settings.rotate_ring_radius * scale,
            ),
        };
        if dist < threshold && dist < best_dist {
            best_dist = dist;
            best_axis = Some(*axis);
        }
    }

    // View handle hover detection
    let view_dist = match settings.mode {
        TransformGizmoMode::Translate => {
            // Check if cursor is within the view-circle radius in screen space
            if let Ok(center_screen) = camera.world_to_viewport(cam_tf, gizmo_pos) {
                let screen_radius = VIEW_CIRCLE_MAJOR * scale;
                // Approximate screen-space radius: project a point on the circle edge
                let edge_world = gizmo_pos + cam_tf.right() * screen_radius;
                if let Ok(edge_screen) = camera.world_to_viewport(cam_tf, edge_world) {
                    let r = (edge_screen - center_screen).length();
                    let d = (cursor_pos - center_screen).length();
                    // Hit if within the torus ring area
                    (d - r).abs()
                } else {
                    f32::MAX
                }
            } else {
                f32::MAX
            }
        }
        TransformGizmoMode::Rotate => {
            // View ring: check distance to a screen-space circle
            let cam_forward = cam_tf.forward().as_vec3();
            point_to_ring_screen_dist(
                cursor_pos,
                camera,
                cam_tf,
                gizmo_pos,
                cam_forward,
                VIEW_RING_MAJOR * scale,
            )
        }
        TransformGizmoMode::Scale => f32::MAX, // no view handle for scale
    };

    if view_dist < threshold && view_dist < best_dist {
        best_axis = Some(TransformGizmoAxis::View);
    }

    state.hovered_axis = best_axis;
}

fn transform_gizmo_drag(
    mut ghosts: Query<(Entity, &GlobalTransform, &mut Transform, Option<&TransformGizmoFocus>), With<Ghost>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    primary_window: Single<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    config: Res<TransformGizmoConfig>,
    mut state: ResMut<TransformGizmoState>,
    mut saved_grab_mode: Local<CursorGrabMode>,
    mut changes: ResMut<Changes>
) {
    let (camera, cam_tf) = *camera;
    let (window, mut cursor_opts) = primary_window.into_inner();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let mut focus_entity: Option<Entity> = None;
    let mut focus_global_transform: Option<GlobalTransform> = None;

    for (ghost_entity, ghost_global_transform, _ghost_transform, maybe_gizmo) in ghosts.iter(){
        if maybe_gizmo.is_some(){
            focus_entity = Some(ghost_entity);
            focus_global_transform = Some(*ghost_global_transform);
            break;
        }
    }

    // Start drag
    if mouse.just_pressed(MouseButton::Left) && !state.active {
        if let Some(axis) = state.hovered_axis && let Some(focus_global_transform) = focus_global_transform{

            let space = effective_space(&config);
            let rotation = gizmo_rotation(&focus_global_transform, space);
            let axis_dir = axis_direction(axis, rotation, cam_tf);
            let gizmo_pos = focus_global_transform.translation();

            let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else {
                return;
            };

            let drag_start_world = match config.mode {
                TransformGizmoMode::Translate => {
                    if axis == TransformGizmoAxis::View {
                        // View-plane translate: use camera forward as normal
                        let plane_normal = cam_tf.forward().as_vec3();
                        let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_pos)
                        else {
                            return;
                        };
                        intersection
                    } else {
                        let plane_normal = translation_plane_normal(ray, axis_dir);
                        let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_pos)
                        else {
                            return;
                        };
                        let cursor_vec = intersection - gizmo_pos;
                        cursor_vec.dot(axis_dir.normalize()) * axis_dir.normalize() + gizmo_pos
                    }
                }
                TransformGizmoMode::Scale => {
                    let plane_normal = translation_plane_normal(ray, axis_dir);
                    let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_pos) else {
                        return;
                    };
                    let cursor_vec = intersection - gizmo_pos;
                    cursor_vec.dot(axis_dir.normalize()) * axis_dir.normalize() + gizmo_pos
                }
                TransformGizmoMode::Rotate => {
                    let rot_axis = if axis == TransformGizmoAxis::View {
                        cam_tf.forward().as_vec3()
                    } else {
                        axis_dir.normalize()
                    };
                    let Some(intersection) = intersect_plane(ray, rot_axis, gizmo_pos) else {
                        return;
                    };
                    (intersection - gizmo_pos).normalize()
                }
            };

            state.data.clear();
            state.active = true;
            state.axis = Some(axis);
            state.drag_start_world = drag_start_world;
            state.gizmo_origin = gizmo_pos;

            for (ghost_entity, _ghost_global_transform, ghost_transform, _maybe_gizmo) in ghosts.iter(){
                state.data.insert(ghost_entity, *ghost_transform);
            }
            
            if config.confine_cursor {
                *saved_grab_mode = cursor_opts.grab_mode;
                cursor_opts.grab_mode = CursorGrabMode::Confined;
            }
        }

        return;
    }

    // Continue drag
    if state.active && mouse.pressed(MouseButton::Left) && let Some(global_tf) = focus_global_transform {
        if focus_entity.is_none(){
            return;
        };
        let Some(axis) = state.axis else {
            return;
        };

        let space = effective_space(&config);
        let rotation = gizmo_rotation(&global_tf, space);
        let axis_dir = axis_direction(axis, rotation, cam_tf);
        let gizmo_origin = state.gizmo_origin;

        let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else {
            return;
        };

        match config.mode {
            TransformGizmoMode::Translate => {
                if axis == TransformGizmoAxis::View {
                    // View-plane translate
                    let plane_normal = cam_tf.forward().as_vec3();
                    let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_origin)
                    else {
                        return;
                    };
                    let delta = intersection - state.drag_start_world;
                    for (dragged_entity, start_transform) in state.data.iter(){
                        if let Ok((_ghost_entity, _ghost_global_transform, mut ghost_transform, _maybe_gizmo)) = ghosts.get_mut(*dragged_entity){
                            let new_pos = start_transform.translation + delta;
                            ghost_transform.translation = match config.snap_translate {
                                Some(inc) => Vec3::new(
                                    snap_value(new_pos.x, inc),
                                    snap_value(new_pos.y, inc),
                                    snap_value(new_pos.z, inc),
                                ),
                                None => new_pos,
                            };
                        }
                    }

                } else {
                    let plane_normal = translation_plane_normal(ray, axis_dir);
                    let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_origin)
                    else {
                        return;
                    };
                    let cursor_vec = intersection - gizmo_origin;
                    let axis_norm = axis_dir.normalize();
                    let new_projected = cursor_vec.dot(axis_norm) * axis_norm + gizmo_origin;
                    let delta = new_projected - state.drag_start_world;

                    for (dragged_entity, start_transform) in state.data.iter(){
                        if let Ok((_ghost_entity, _ghost_global_transform, mut ghost_transform, _maybe_gizmo)) = ghosts.get_mut(*dragged_entity){
                            let new_pos = start_transform.translation + delta;
                            ghost_transform.translation = match config.snap_translate {
                                Some(inc) => {
                                    snap_axis(new_pos, start_transform.translation, axis, inc)
                                }
                                None => new_pos,
                            };
                        }
                    }
                }
            }
            TransformGizmoMode::Rotate => {
                let rot_axis = if axis == TransformGizmoAxis::View {
                    cam_tf.forward().as_vec3()
                } else {
                    axis_dir.normalize()
                };
                let Some(intersection) = intersect_plane(ray, rot_axis, gizmo_origin) else {
                    return;
                };
                let cursor_vector = (intersection - gizmo_origin).normalize();
                let drag_start = state.drag_start_world; // normalized direction

                let dot = drag_start.dot(cursor_vector);
                let det = rot_axis.dot(drag_start.cross(cursor_vector));
                let raw_angle = bevy::math::ops::atan2(det, dot);
                let angle = match config.snap_rotate {
                    Some(inc) => snap_value(raw_angle, inc),
                    None => raw_angle,
                };
                let rotation_delta = Quat::from_axis_angle(rot_axis, angle);
                for (dragged_entity, start_transform) in state.data.iter(){
                    if let Ok((_ghost_entity, _ghost_global_transform, mut ghost_transform, _maybe_gizmo)) = ghosts.get_mut(*dragged_entity){
                        ghost_transform.rotation = rotation_delta * start_transform.rotation;
                    }
                }
            }
            TransformGizmoMode::Scale => {
                let plane_normal = translation_plane_normal(ray, axis_dir);
                let Some(intersection) = intersect_plane(ray, plane_normal, gizmo_origin) else {
                    return;
                };
                let axis_norm = axis_dir.normalize();
                let cursor_projected = (intersection - gizmo_origin).dot(axis_norm);
                let start_projected = (state.drag_start_world - gizmo_origin).dot(axis_norm);

                let scale_factor = if start_projected.abs() > f32::EPSILON {
                    cursor_projected / start_projected
                } else {
                    1.0
                };

                for (dragged_entity, start_transform) in state.data.iter(){
                    let mut new_scale = start_transform.scale;
                    match axis {
                        TransformGizmoAxis::X => {
                            new_scale.x = (new_scale.x * scale_factor).max(MIN_SCALE);
                        }
                        TransformGizmoAxis::Y => {
                            new_scale.y = (new_scale.y * scale_factor).max(MIN_SCALE);
                        }
                        TransformGizmoAxis::Z => {
                            new_scale.z = (new_scale.z * scale_factor).max(MIN_SCALE);
                        }
                        TransformGizmoAxis::View => {
                            // Uniform scale on view axis
                            new_scale *= scale_factor;
                            new_scale = new_scale.max(Vec3::splat(MIN_SCALE));
                        }
                    }

                     if let Ok((_ghost_entity, _ghost_global_transform, mut ghost_transform, _maybe_gizmo)) = ghosts.get_mut(*dragged_entity){
                        ghost_transform.scale = match config.snap_scale {
                            Some(inc) => {
                                let mut snapped = start_transform.scale;
                                match axis {
                                    TransformGizmoAxis::X => snapped.x = snap_value(new_scale.x, inc),
                                    TransformGizmoAxis::Y => snapped.y = snap_value(new_scale.y, inc),
                                    TransformGizmoAxis::Z => snapped.z = snap_value(new_scale.z, inc),
                                    TransformGizmoAxis::View => {
                                        snapped = Vec3::splat(snap_value(new_scale.x, inc));
                                    }
                                }
                                snapped
                            }
                            None => new_scale,
                        };
                    }
                }
            }
        }
        return;
    }

    // End drag
    if state.active && mouse.just_released(MouseButton::Left) {
        state.active = false;
        state.axis = None;

        // Record changes
        let mut changeset = ChangesSet::new();
        for (dragged_entity, start_transform) in state.data.iter(){
            if let Ok((_ghost_entity, _ghost_global_transform, ghost_transform, _maybe_gizmo)) = ghosts.get_mut(*dragged_entity){
                let ct = ChangeTransform{entity: *dragged_entity, old: *start_transform, new: *ghost_transform};
                changeset.add(ct);
            }
        }
        changeset.record(&mut changes);

        state.data.clear();
        if config.confine_cursor {
            cursor_opts.grab_mode = *saved_grab_mode;
        }
    }
}


/// Get the world-space direction for a given axis.
pub fn axis_direction(axis: TransformGizmoAxis, rotation: Quat, cam_tf: &GlobalTransform) -> Vec3 {
    match axis {
        TransformGizmoAxis::X => rotation * Vec3::X,
        TransformGizmoAxis::Y => rotation * Vec3::Y,
        TransformGizmoAxis::Z => rotation * Vec3::Z,
        TransformGizmoAxis::View => cam_tf.forward().as_vec3(),
    }
}

/// Construct the constraint plane normal for axis translation/scale.
///
/// The plane contains the drag axis and is oriented to face the camera as much
/// as possible, matching the approach from `bevy_transform_gizmo`.
pub fn translation_plane_normal(ray: Ray3d, axis: Vec3) -> Vec3 {
    let vertical = Vec3::from(ray.direction).cross(axis);
    if vertical.length_squared() < f32::EPSILON {
        // Ray is nearly parallel to the axis -- pick an arbitrary perpendicular.
        return axis.any_orthonormal_vector();
    }
    axis.cross(vertical.normalize()).normalize()
}

/// Intersect a ray with a plane defined by a normal and a point on the plane.
pub fn intersect_plane(ray: Ray3d, plane_normal: Vec3, plane_origin: Vec3) -> Option<Vec3> {
    let denominator = Vec3::from(ray.direction).dot(plane_normal);
    if denominator.abs() > f32::EPSILON {
        let point_to_point = plane_origin - ray.origin;
        let intersect_dist = plane_normal.dot(point_to_point) / denominator;
        Some(Vec3::from(ray.direction) * intersect_dist + ray.origin)
    } else {
        None
    }
}

/// Distance from a point to a line segment in 2D.
pub fn point_to_segment_dist(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let t = (ap.dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (point - closest).length()
}

/// Minimum screen-space distance from a cursor position to a 3D ring projected onto screen.
pub fn point_to_ring_screen_dist(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    center: Vec3,
    normal: Vec3,
    radius: f32,
) -> f32 {
    // Quick reject: if cursor is far from the ring center in screen space, skip sampling
    if let Ok(center_screen) = camera.world_to_viewport(cam_tf, center)
        && let Ok(edge_screen) = camera.world_to_viewport(cam_tf, center + cam_tf.right() * radius)
    {
        let screen_radius = (edge_screen - center_screen).length();
        let cursor_dist = (cursor - center_screen).length();
        if (cursor_dist - screen_radius).abs() > screen_radius * 0.5 {
            return f32::MAX;
        }
    }

    const RING_SAMPLES: usize = 64;
    let rot = Quat::from_rotation_arc(Vec3::Z, normal);
    let mut min_dist = f32::MAX;
    let mut prev_screen = None;

    for i in 0..=RING_SAMPLES {
        let angle = (i % RING_SAMPLES) as f32 * core::f32::consts::TAU / RING_SAMPLES as f32;
        let local = Vec3::new(
            bevy::math::ops::cos(angle) * radius,
            bevy::math::ops::sin(angle) * radius,
            0.0,
        );
        let world = center + rot * local;
        let Some(screen) = camera.world_to_viewport(cam_tf, world).ok() else {
            prev_screen = None;
            continue;
        };
        if let Some(prev) = prev_screen {
            let dist = point_to_segment_dist(cursor, prev, screen);
            if dist < min_dist {
                min_dist = dist;
            }
        }
        prev_screen = Some(screen);
    }

    min_dist
}

/// Return the effective space for the gizmo: scale always uses local space.
pub fn effective_space(settings: &TransformGizmoConfig) -> &TransformGizmoSpace {
    if settings.mode == TransformGizmoMode::Scale {
        &TransformGizmoSpace::Local
    } else {
        &settings.space
    }
}

/// Compute the gizmo rotation based on the space setting.
pub fn gizmo_rotation(global_tf: &GlobalTransform, space: &TransformGizmoSpace) -> Quat {
    match space {
        TransformGizmoSpace::World => Quat::IDENTITY,
        TransformGizmoSpace::Local => {
            let (_, rotation, _) = global_tf.to_scale_rotation_translation();
            rotation
        }
    }
}

fn snap_value(value: f32, increment: f32) -> f32 {
    (value / increment).round() * increment
}

/// Snap only the component along the dragged axis, leaving others unchanged.
fn snap_axis(position: Vec3, original: Vec3, axis: TransformGizmoAxis, increment: f32) -> Vec3 {
    match axis {
        TransformGizmoAxis::X => {
            Vec3::new(snap_value(position.x, increment), original.y, original.z)
        }
        TransformGizmoAxis::Y => {
            Vec3::new(original.x, snap_value(position.y, increment), original.z)
        }
        TransformGizmoAxis::Z => {
            Vec3::new(original.x, original.y, snap_value(position.z, increment))
        }
        TransformGizmoAxis::View => {
            // Snap all axes uniformly
            Vec3::new(
                snap_value(position.x, increment),
                snap_value(position.y, increment),
                snap_value(position.z, increment),
            )
        }
    }
}