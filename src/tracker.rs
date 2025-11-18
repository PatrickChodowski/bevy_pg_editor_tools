use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

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



