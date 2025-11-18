use bevy::prelude::*;

// WorldPosition under cursor
#[derive(Resource)]
pub struct WorldPos {
    loc: Option<Vec3>
}
// WorldPosition under cursor

impl WorldPos {
    pub fn new() -> Self {
        Self {
            loc: None
        }
    }
    pub fn set(&mut self, loc: Vec3){
        self.loc = Some(loc);
    }
    pub fn reset(&mut self){
        self.loc = None;
    }
    pub fn get(&self) -> Option<Vec3> {
        self.loc
    }
}