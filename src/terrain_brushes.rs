use bevy::prelude::*;
use bevy::ecs::system::SystemState;

use crate::brushes::BrushType;
use crate::prelude::{PlaneVertex, SelectedVertex, Noise};

#[derive(Clone)]
pub struct Terrace {
    pub min: f32,
    pub max: f32,
    pub value: f32
}
impl Terrace {
    fn new() -> Terrace {
        Terrace { min: 0.0, max: 1.0, value: 0.5}
    }
}

#[derive(Clone)]
pub enum HeightBrushType {
    Value(f32),
    Terraces(Vec<Terrace>),
    Noise((Vec<Noise>, f32))
}

#[derive(Clone)]
pub struct TerrainHeightBrush {
    pub typ: HeightBrushType,
    pub reselection: bool
}

impl BrushType for TerrainHeightBrush {
    fn apply(&mut self, world: &mut World, loc: Vec3, radius: f32) {

        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, &mut PlaneVertex, &mut Transform, &GlobalTransform, Option<&SelectedVertex>)>
        )> = SystemState::new(world);
        let (mut commands, mut plane_vertices) = system_state.get_mut(world);

        for (vertex_entity, mut plane_vertex, mut vertex_transform, global_transform, maybe_selected) in plane_vertices.iter_mut(){

            let global_loc = global_transform.translation();
            let near: bool = global_loc.xz().distance(loc.xz()) <= (radius + plane_vertex.radius);

            if near & maybe_selected.is_none() {
                commands.entity(vertex_entity).insert(SelectedVertex);

                match &self.typ {
                    HeightBrushType::Value(y) => {
                        vertex_transform.translation.y += y;
                    }
                    HeightBrushType::Terraces(terraces) => {
                        for terrace in terraces {
                            if vertex_transform.translation.y >= terrace.min && vertex_transform.translation.y <= terrace.max {
                                vertex_transform.translation.y = terrace.value;
                                continue;
                            }
                        }
                    }
                    HeightBrushType::Noise(noises) => {
                        let mut combined_noise: f32 = 0.0;
                        for noise in noises.0.iter(){
                            let noise_value = noise.apply(global_loc);
                            combined_noise += noise_value;
                        }
                        let new_y: f32 = combined_noise*noises.1;
                        vertex_transform.translation.y = new_y;
                    }
                }

                plane_vertex.loc = vertex_transform.translation.into();
            } else if  near & maybe_selected.is_some() {

            } else {
                if self.reselection {
                    commands.entity(vertex_entity).remove::<SelectedVertex>();
                }
            }
        }
        system_state.apply(world);
    }
    fn done(&mut self, world: &mut World) {
        let mut system_state: SystemState<(
            Commands,
            Query<Entity, With<SelectedVertex>>
        )> = SystemState::new(world);
        let (mut commands, plane_vertices) = system_state.get_mut(world);
        for entity in plane_vertices.iter(){
            commands.entity(entity).remove::<SelectedVertex>();
        }
        system_state.apply(world);
    }
    fn started(&mut self, world: &mut World) {}
}

#[derive(Clone)]
pub struct TerrainColorBrush {
    pub typ: ColorBrushType
}

#[derive(Clone)]
pub enum ColorBrushType {
    Value{clr: [f32;4]},
    Range{min: f32, max: f32, min_clr: [f32;4], max_clr: [f32;4]},
    Noise{data: Vec<Noise>, value: f32, clr: [f32;4]}
}


impl BrushType for TerrainColorBrush {
    fn apply(&mut self, world: &mut World, loc: Vec3, radius: f32) {
        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, &mut PlaneVertex, &GlobalTransform, Option<&SelectedVertex>)>
        )> = SystemState::new(world);
        let (mut commands, mut plane_vertices) = system_state.get_mut(world);

        for (vertex_entity, mut plane_vertex, global_transform, maybe_selected) in plane_vertices.iter_mut(){

            let global_loc = global_transform.translation();
            let near: bool = global_loc.xz().distance(loc.xz()) <= (radius + plane_vertex.radius);

            if near & maybe_selected.is_none() {
                commands.entity(vertex_entity).insert(SelectedVertex);
                match &self.typ {
                    ColorBrushType::Value{clr} => {plane_vertex.clr = *clr;}
                    ColorBrushType::Noise { data, value, clr} => {

                        let mut combined_noise: f32 = 0.0;
                        for noise in data.iter(){
                            let noise_value = noise.apply(global_loc);
                            combined_noise += noise_value;
                        }
                        let alpha: f32 = combined_noise*value;
                        plane_vertex.clr = [clr[0], clr[1], clr[2], alpha.clamp(0.0, 1.0)];
                    }
                    ColorBrushType::Range { min, max, min_clr, max_clr } => {
                        if &global_loc.y >= min && &global_loc.y <= max {
                            let norm_y = (global_loc.y - min)/(max-min);
                            let interpolated_clr = [
                                min_clr[0] + (max_clr[0] - min_clr[0]) * norm_y,
                                min_clr[1] + (max_clr[1] - min_clr[1]) * norm_y,
                                min_clr[2] + (max_clr[2] - min_clr[2]) * norm_y,
                                min_clr[3] + (max_clr[3] - min_clr[3]) * norm_y,
                            ];
                            plane_vertex.clr = interpolated_clr;
                        }
                    }
                }
            } else if  near & maybe_selected.is_some() {

            } else {
                commands.entity(vertex_entity).remove::<SelectedVertex>();
            }
        }
        system_state.apply(world);
    }
    fn done(&mut self, world: &mut World) {

        let mut system_state: SystemState<(
            Commands,
            Query<Entity, With<SelectedVertex>>
        )> = SystemState::new(world);
        let (mut commands, plane_vertices) = system_state.get_mut(world);
        for entity in plane_vertices.iter(){
            commands.entity(entity).remove::<SelectedVertex>();
        }
        system_state.apply(world);

    }
    fn started(&mut self, world: &mut World) {}
}
