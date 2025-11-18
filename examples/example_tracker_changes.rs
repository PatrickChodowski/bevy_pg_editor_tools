use bevy::prelude::*;
use bevy::platform::collections::HashMap;

use bevy_pg_editor_tools::prelude::{TrackerChange, Changes};



#[derive(Clone)]
pub struct ChangeSpawn {
    entity: Entity,
    transform: Transform
}
impl ChangeSpawn {
    pub fn new(
        entity: Entity, 
        transform: Transform
    ) -> ChangeSpawn {
        Self {
            entity, transform
        }
    }
}

impl TrackerChange for ChangeSpawn {
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
        world.resource_scope(|world1: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world1.resource_scope(|world2: &mut World, mut materials: Mut<Assets<StandardMaterial>>|{
                let ass = world2.resource::<AssetServer>();
                let entity = world2.spawn(self.transform).id();
                self.entity = entity;
            })
        });        
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
    transform: Transform    // Last Transform
}
impl ChangeDespawn {
    pub fn new(
        entity: Entity, 
        transform: Transform
    ) -> ChangeDespawn {
        Self {
            entity, transform
        }
    }
}


impl TrackerChange for ChangeDespawn {
    fn undo(
        &mut self, 
        world:      &mut World
    ) {

        world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world: &mut World, mut materials: Mut<Assets<StandardMaterial>>|{
                let ass = world.resource::<AssetServer>();
                let entity = world.spawn(
                    self.transform
                ).id();
                self.entity = entity;
            })
        });   
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


impl TrackerChange for ChangeTransform {
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
