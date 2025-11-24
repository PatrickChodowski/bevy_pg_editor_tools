use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::camera::primitives::Aabb;
use bevy_enhanced_input::prelude::Fire;
use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};
use bevy_pg_core::prelude::{MainCamera, GameStatePlay, Player};

use crate::assets_panel::list_assets;
use crate::assets_panel::EditorAssetPanel;
use crate::controller::TriggerThumbnails;


const FRAME_DELAY: usize = 2;
const OBJLOC: [f32; 3] = [0.0, 10000.0, 0.0];

pub struct PGEditorThumbnailsPlugin;


impl Plugin for PGEditorThumbnailsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_message::<MakeThumbnail>()
        .insert_resource(DoneEntities::default())

        .add_systems(Update,     
            (
                (
                    track, 
                    assets_ready
                ).run_if(resource_exists::<AssetHandles>)
            ).run_if(in_state(GameStatePlay::Editor))
        )

        .add_systems(PreUpdate,  make_thumbnail.run_if(in_state(GameStatePlay::Editor)
                                               .and(on_message::<MakeThumbnail>)))
        .add_systems(PostUpdate, (
                despawn_captured.run_if(resource_exists::<DoneThumbnaililing>), 
                init_make_thumbnail.run_if(resource_exists::<AssetHandles>)
            ).run_if(in_state(GameStatePlay::Editor))
        )

        .add_systems(Last, check_if_done.run_if(in_state(GameStatePlay::Editor)
                                        .and(resource_exists::<DoneThumbnaililing>)))
        
        .add_observer(thumbnails)
        ;
    }
}


fn check_if_done(
    mut commands:           Commands,
    done:                   Res<DoneThumbnaililing>,
    mut done_entities:      ResMut<DoneEntities>,
    mut clear_color:        ResMut<ClearColor>,
    mut camera:             Query<(Entity, &mut Transform, &mut MainCamera), Without<Player>>,
    mut ui:                 Query<&mut Visibility, With<EditorAssetPanel>>,
    player:                 Query<&Transform, With<Player>>
){
    if done.done {
        info!("EDITOR: Done Thumbnails");
        for mut vis in ui.iter_mut(){
            *vis = Visibility::Inherited;
        }
        
        commands.remove_resource::<DoneThumbnaililing>();
        commands.remove_resource::<AssetHandles>();
        done_entities.data = Vec::new();
        clear_color.0 = Color::srgb(0.721, 0.863, 0.992);

        let Ok((camera_entity, mut camera_transform, mut camera_data)) = camera.single_mut() else {return};
        let Ok(player_transform) = player.single() else {return};
        camera_data.set_player(&mut commands, camera_entity, &mut camera_transform, player_transform.translation);
    }

}


#[derive(Resource)]
struct DoneEntities {
    data: Vec<(Entity, usize, bool)>
}
impl Default for DoneEntities {
    fn default() -> Self {
        DoneEntities{data: Vec::new()}
    }
}

#[derive(Resource)]
struct DoneThumbnaililing {
    done: bool
}
impl Default for DoneThumbnaililing {
    fn default() -> Self {
        DoneThumbnaililing{done: false}
    }
}


fn despawn_captured(
    mut commands:       Commands,
    mut done_entities:  ResMut<DoneEntities>
) { 
    for (entity, frame_delay, rm) in done_entities.data.iter_mut(){
        if *frame_delay == 0 {
            commands.entity(*entity).despawn();
            *rm = true;
        } else {
            *frame_delay -= 1;
        }
    }

    done_entities.data.retain(|x| x.2 == false);

}

fn make_thumbnail(
    mut commands:       Commands,
    mut make_thumbnail: MessageReader<MakeThumbnail>,
    mut query:          Query<(Entity, &Transform, &mut ThumbnailObject), Without<MainCamera>>,
    mut done_entities:  ResMut<DoneEntities>,
    mut camera:         Query<(&Projection, &mut Transform), With<MainCamera>>,
    children:           Query<&Children>,
    aabbs:              Query<&Aabb>

){
    for mt in make_thumbnail.read(){
        if let Ok((entity, transform, mut tobj)) = query.get_mut(mt.entity){

            let Ok((_projection, mut camera_transform)) = camera.single_mut() else {return};
            camera_transform.translation = Vec3::new(-50.0, 10050.0, -50.0);
            camera_transform.look_at(OBJLOC.into(), Vec3::Y);

            

            for descendant in children.iter_descendants_depth_first(entity){
                if let Ok(aabb) = aabbs.get(descendant){
                    let radius = aabb.half_extents.length()*transform.scale.y*1.25; //scale
                    let distance = radius / (FRAC_PI_4 * 0.5).tan();
                    let center = Vec3::new(0.0, 10000.0 + aabb.half_extents.y*0.5*transform.scale.y, 0.0);
                    let direction = Vec3::new(-FRAC_PI_8, FRAC_PI_8, FRAC_PI_4);
                    let camera_pos = center + direction * distance;
                    camera_transform.translation = camera_pos;
                    camera_transform.look_at(center, Vec3::Y);
                    break;
                }
            }
            *tobj = ThumbnailObject::TakePicture;
            let path = format!("./assets/editor/{}.png", mt.name.replace("objects/","").replace(".glb",""));
            // info!("Save path: {}", path);
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            done_entities.data.push((mt.entity, FRAME_DELAY, false));
        }
    }
}

#[derive(Message)]
struct MakeThumbnail {
    entity: Entity, 
    name: String
}

#[derive(Resource)]
struct DelaySwitch {
    frames_left: usize
}
impl Default for DelaySwitch {
    fn default() -> Self {
        DelaySwitch{frames_left: FRAME_DELAY}
    }
}

fn init_make_thumbnail(
    mut query:          Query<(Entity, &mut Visibility, &Name, &ThumbnailObject)>,
    mut make_thumbnail: MessageWriter<MakeThumbnail>,
    mut delay_switch:   Local<DelaySwitch>,
    done_thumbnailing:  Option<ResMut<DoneThumbnaililing>>
){
    if delay_switch.frames_left > 0 {
        delay_switch.frames_left -= 1;
        return;
    }

    if let Some(mut dt) = done_thumbnailing {
        if query.iter().len() == 0 {
            dt.done = true;
        }
    }


    for (entity, mut vis, name, tobj) in query.iter_mut(){
        if *tobj == ThumbnailObject::Init {
            *vis = Visibility::Visible;
            make_thumbnail.write(MakeThumbnail{entity, name: name.to_string()});
            delay_switch.frames_left = FRAME_DELAY;
            return;
        }
    }

}


#[derive(Component, PartialEq)]
enum ThumbnailObject {
    Init,
    TakePicture
}

fn assets_ready(
   mut commands:           Commands,
   mut handles:            ResMut<AssetHandles>,
   mut camera:             Query<(Entity, &mut Transform, &mut MainCamera)>
){
    if !handles.ready {
        return;
    } 
    if handles.spawned {
        return;
    }

    for (handle, name) in handles.data.iter(){
        commands.spawn((
            Transform::from_translation(OBJLOC.into())
                      .with_scale(Vec3::splat(10.0))
                      .with_rotation(Quat::from_euler(EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0)),
            Visibility::Hidden,
            ThumbnailObject::Init,
            name.clone(),
            SceneRoot(handle.clone())
        ));
    }
    handles.spawned = true;

    info!("EDITOR: Thumbnails asset count: {}", handles.data.len());

    let Ok((camera_entity, mut camera_transform, mut main_camera)) = camera.single_mut() else {return};
    camera_transform.translation = Vec3::new(-50.0, 1050.0, -50.0);
    camera_transform.look_at(OBJLOC.into(), Vec3::Y);
    main_camera.set_dev(&mut commands, camera_entity);
    commands.insert_resource(DoneThumbnaililing::default())

}


fn track(
    ass:         Res<AssetServer>,
    mut handles: ResMut<AssetHandles>
){
    if handles.ready {
        return;
    }

    let mut load_count: usize = 0;
    for (handle, _name) in handles.data.iter(){
        if let Some(handle_load_state) = ass.get_load_state(handle){
            if handle_load_state.is_loaded(){
                load_count += 1;
            }
        }
    }

    if load_count == handles.data.len(){
        handles.ready = true;
    }

}


#[derive(Resource)]
struct AssetHandles {
    data: Vec<(Handle<Scene>, Name)>,
    ready: bool,
    spawned: bool
}


fn thumbnails(
    _trigger: On<Fire<TriggerThumbnails>>,
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    mut ui: Query<&mut Visibility, With<EditorAssetPanel>>
){
    for mut vis in ui.iter_mut(){
        *vis = Visibility::Hidden;
    }

    clear_color.0 = WHITE.with_alpha(0.0).into();
    info!("EDITOR: Starts Thumbnails");

    let mut handles: Vec<(Handle<Scene>, Name)> = Vec::with_capacity(200);

    let assets = list_assets();
    for file_name in assets {
        let path = format!("objects/{}", file_name);
        handles.push(
            (
                ass.load(GltfAssetLabel::Scene(0).from_asset(path.clone())), 
                Name::from(path.clone())
            )
        ); 
    }
    commands.insert_resource(
        AssetHandles{
            data: handles, 
            ready: false, spawned: false
        });
}
