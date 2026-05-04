use bevy::prelude::*;
use bevy::ecs::system::SystemState;
use bevy_enhanced_input::prelude::*;
use bevy::platform::collections::HashMap;
use bevy_pg_core::prelude::EditorAsset;

use crate::ghost::{EditorGhostSettings, editor_asset_bundle};
use crate::planes::plane_mesh;
use crate::vertex::PlaneVertex;
pub struct PGEditorTrackerPlugin;


impl Plugin for PGEditorTrackerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(Changes::new())
        .add_message::<UndoMessage>()
        .add_message::<RedoMessage>()
        .add_observer(on_undo)
        .add_observer(on_redo)
        .add_systems(Update, 
            (
                undo.run_if(on_message::<UndoMessage>),
                redo.run_if(on_message::<RedoMessage>),
            )
        )
        ;
    }
}

pub trait Change:  Send + Sync {
    fn undo(&mut self, world:&mut World){}
    fn redo(&mut self, world: &mut World){}
    fn record(&self, changes: &mut ResMut<Changes>){}
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct Undo;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Redo;

#[derive(Message)]
pub struct UndoMessage;

#[derive(Message)]
pub struct RedoMessage;


fn on_undo(
    _trigger: On<Fire<Undo>>,
    mut writer: MessageWriter<UndoMessage>
){
    writer.write(UndoMessage);
}

fn on_redo(
    _trigger: On<Fire<Redo>>,
    mut writer: MessageWriter<RedoMessage>
){
    writer.write(RedoMessage);
}

fn undo(
    world:     &mut World,
){
    world.resource_scope(|_world: &mut World, mut changes: Mut<Changes>| {
        if let Some(change_index) = changes.undo_index() {
            changes.undo(change_index, _world);
        }
    });
}

fn redo(
    world:     &mut World,
){
    world.resource_scope(|_world: &mut World, mut changes: Mut<Changes>| {
        if let Some(change_index) = changes.redo_index() {
            changes.redo(change_index, _world);
        }
    });
}


#[derive(Resource)]
pub struct Changes {
    pub index: isize, // Current position of redo/undo
    pub data: Vec<Box<dyn Change>>
}
impl Changes {
    pub fn new() -> Self {
        Self {
            index: 0,
            data: Vec::with_capacity(1000)
        }
    }

    pub fn undo(&mut self, index: usize, world: &mut World){
        self.data[index].undo(world);
    }

    pub fn redo(&mut self, index: usize, world: &mut World){
        self.data[index].redo(world);
    }

    pub fn record(&mut self, change: Box<dyn Change>){
        self.data.push(change);
        self.index = self.len();
    }

    fn undo_index(&mut self) -> Option<usize> {
        if self.len() == 0{
            return None;
        }
        if self.index > 0 {
            let change_index = (self.index -1) as usize;
            self.index -= 1;
            self.manage_index();
            return Some(change_index);
        }
        return None;
    }

    fn redo_index(&mut self) -> Option<usize> {
        if self.len() == 0{
            return None;
        }
        if self.index < self.len() {
            let change_index = self.index as usize;
            self.index += 1;
            self.manage_index();
            return Some(change_index);
        }
        return None;
    }

    fn len(&self) -> isize {
        self.data.len() as isize
    }
    fn manage_index(&mut self){
        self.index = self.index.clamp(0, self.len());
    }

}


#[derive(Clone)]
pub struct ChangesSet<T: Change + Clone + 'static> {
    data: Vec<T>
}
impl<T: Change + Clone + 'static> ChangesSet<T> {
    pub fn new() -> Self {
        ChangesSet { data: Vec::new() }
    }
    pub fn add(&mut self, change: T){
        self.data.push(change);
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: Change + Clone + 'static> Change for ChangesSet<T> {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
       for change in self.data.iter_mut(){
            change.undo(world);
       }
    }
    fn redo(
        &mut self, 
        world: &mut World
    ){
        for change in self.data.iter_mut(){
            change.redo(world);
       }
    }
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}





#[derive(Clone)]
pub struct ChangeSpawn {
    asset:  EditorAsset,
    entity: Entity,
    transform: Transform
}
impl ChangeSpawn {
    pub fn new(
        entity: Entity, 
        asset: EditorAsset, 
        transform: Transform
    ) -> ChangeSpawn {
        Self {
            entity, asset, transform
        }
    }
}

impl Change for ChangeSpawn {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        world.despawn(self.entity);
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {

        let mut system_state: SystemState<(
            ResMut<Assets<Mesh>>,
            ResMut<Assets<StandardMaterial>>,
            Res<AssetServer>,
            Commands,
            Res<EditorGhostSettings>
        )> = SystemState::new(world);

        let (mut meshes, mut materials, ass, mut commands, ghost_settings) = system_state.get_mut(world);
        let entity = commands.spawn(
            editor_asset_bundle(
                self.asset.clone(),
                &ass,
                &mut meshes,
                &mut materials,
                &self.transform,
                &ghost_settings
            ).unwrap()  // WILL FAIL WITH WATER
        ).id();
        self.entity = entity;    
        system_state.apply(world);   
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}


#[derive(Clone)]
pub struct ChangeDespawn {
    entity:    Entity,
    asset:     EditorAsset,
    transform: Transform    // Last Transform
}
impl ChangeDespawn {
    pub fn new(
        entity: Entity, 
        asset: EditorAsset, 
        transform: Transform
    ) -> ChangeDespawn {
        Self {
            entity, asset, transform
        }
    }
}


impl Change for ChangeDespawn {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {

        let mut system_state: SystemState<(
            ResMut<Assets<Mesh>>,
            ResMut<Assets<StandardMaterial>>,
            Res<AssetServer>,
            Commands,
            Res<EditorGhostSettings>
        )> = SystemState::new(world);

        let (mut meshes, mut materials, ass, mut commands, ghost_settings) = system_state.get_mut(world);
        let entity = commands.spawn(
            editor_asset_bundle(
                self.asset.clone(),
                &ass,
                &mut meshes,
                &mut materials,
                &self.transform,
                &ghost_settings
            ).unwrap() // WILL FAIL WITH WATER
        ).id();
        self.entity = entity;
        system_state.apply(world);
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {
        world.despawn(self.entity);
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}


#[derive(Copy, Clone)]
pub struct ChangeTransform {
    pub entity: Entity,
    pub old: Transform,
    pub new: Transform
}
impl ChangeTransform {
    pub fn new(
        entity: Entity, 
        transform: Transform
    ) -> Self {
        Self {
            entity,
            old: transform,
            new: transform
        }
    }
}

impl Change for ChangeTransform {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        if let Some(mut transform) = world.entity_mut(self.entity).get_mut::<Transform>(){
            *transform = self.old;
        }
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {
        if let Some(mut transform) = world.entity_mut(self.entity).get_mut::<Transform>(){
            *transform = self.new;
        }
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}


// Used for Dragging and Pressing changes
#[derive(Resource)]
pub struct CurrentTransformChanges {
    pub data: HashMap<Entity, ChangeTransform>
}
impl CurrentTransformChanges {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }
    pub fn add(&mut self, entity: Entity, transform: &Transform){
        self.data.insert(entity, ChangeTransform::new(entity, *transform));
    }
    pub fn get(&mut self, entity: Entity) -> &mut ChangeTransform {
        self.data.get_mut(&entity).unwrap()
    }
}




#[derive(Clone)]
pub struct ChangePlaneSpawn {
    entity: Entity,
    width: f32,
    height: f32,
    subdivisions: u32,
    loc: Vec3
}
impl ChangePlaneSpawn {
    pub fn new(
        entity: Entity, 
        width: f32,
        height: f32,
        subdivisions: u32,
        loc: Vec3
    ) -> ChangePlaneSpawn {
        Self {
            entity, width, height, subdivisions, loc
        }
    }
}

impl Change for ChangePlaneSpawn {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        world.despawn(self.entity);
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {

        let mut system_state: SystemState<(
            ResMut<Assets<Mesh>>,
            ResMut<Assets<StandardMaterial>>,
            Commands,
        )> = SystemState::new(world);

        let (mut meshes, mut materials, mut commands) = system_state.get_mut(world);

        let entity = commands.spawn(
            (
                plane_mesh(self.width, self.height, self.subdivisions, &mut meshes),
                MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
                Transform::from_translation(self.loc)
            )
        ).id();
        self.entity = entity;    
        system_state.apply(world);   
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}


#[derive(Clone)]
pub struct ChangePlaneDespawn {
    entity: Entity,
    width: f32,
    height: f32,
    subdivisions: u32,
    loc: Vec3
}
impl ChangePlaneDespawn {
    pub fn new(
        entity: Entity, 
        width: f32,
        height: f32,
        subdivisions: u32,
        loc: Vec3
    ) -> ChangePlaneDespawn {
        Self {
            entity, width, height, subdivisions, loc
        }
    }
}

impl Change for ChangePlaneDespawn {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        let mut system_state: SystemState<(
            ResMut<Assets<Mesh>>,
            ResMut<Assets<StandardMaterial>>,
            Commands,
        )> = SystemState::new(world);

        let (mut meshes, mut materials, mut commands) = system_state.get_mut(world);

        let entity = commands.spawn(
            (
                plane_mesh(self.width, self.height, self.subdivisions, &mut meshes),
                MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
                Transform::from_translation(self.loc)
            )
        ).id();
        self.entity = entity;    
        system_state.apply(world);  
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {
        world.despawn(self.entity); 
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}




#[derive(Clone)]
pub struct ChangePlaneHeight {
    entity: Entity,
    previous_y: f32,
    new_y: f32
}
impl ChangePlaneHeight {
    pub fn new(
        entity: Entity, 
        previous_y: f32,
        new_y: f32
    ) -> ChangePlaneHeight {
        Self {
            entity,
            previous_y,
            new_y
        }
    }
}


impl Change for ChangePlaneHeight {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        if let Some(mut transform) = world.entity_mut(self.entity).get_mut::<Transform>(){
            transform.translation.y = self.previous_y;
        }
        if let Some(mut plane_vertex) = world.entity_mut(self.entity).get_mut::<PlaneVertex>(){
            plane_vertex.loc[1] = self.previous_y;
        }
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {
        if let Some(mut transform) = world.entity_mut(self.entity).get_mut::<Transform>(){
            transform.translation.y = self.new_y;
        }
        if let Some(mut plane_vertex) = world.entity_mut(self.entity).get_mut::<PlaneVertex>(){
            plane_vertex.loc[1] = self.new_y;
        }
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}



#[derive(Clone)]
pub struct ChangePlaneColor {
    entity: Entity,
    old_color: [f32;4],
    new_color: [f32;4]
}
impl ChangePlaneColor {
    pub fn new(
        entity: Entity, 
        old_color: [f32;4],
        new_color: [f32;4],
    ) -> ChangePlaneColor {
        Self {
            entity,
            old_color,
            new_color
        }
    }
}


impl Change for ChangePlaneColor {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {
        if let Some(mut plane_vertex) = world.entity_mut(self.entity).get_mut::<PlaneVertex>(){
            plane_vertex.clr = self.old_color;
        }
    }

    fn redo(
        &mut self, 
        world: &mut World
    ) {
        if let Some(mut plane_vertex) = world.entity_mut(self.entity).get_mut::<PlaneVertex>(){
            plane_vertex.clr = self.new_color;
        }
    }
    
    fn record(
        &self,
        changes: &mut ResMut<Changes> 
    ) {
        changes.record(Box::new(self.clone()));
    }
}
