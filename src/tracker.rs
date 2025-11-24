use bevy::prelude::*;
use bevy::ecs::system::SystemState;
use bevy_enhanced_input::prelude::*;
use bevy::platform::collections::HashMap;

use crate::ghost::{EditorAsset, EditorGhostSettings, editor_asset_bundle};
pub struct PGEditorTrackerPlugin;


impl Plugin for PGEditorTrackerPlugin {
    fn build(&self, app: &mut App) {
        app
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
