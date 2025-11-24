use bevy::prelude::*;

pub fn plane_mesh(
    width: f32,
    height: f32,
    subdivisions: u32,
    meshes: &mut ResMut<Assets<Mesh>>
) -> impl Bundle {
    (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(width, height).subdivisions(subdivisions))),
        PlaneToEdit{width, height, subdivisions}
    )
}



#[derive(Component)]
pub struct PlaneToEdit{
    pub width: f32,
    pub height: f32,
    pub subdivisions: u32
}

impl PlaneToEdit {
    pub fn dummy() -> Self {
        PlaneToEdit {
            width: 0.0,
            height: 0.0,
            subdivisions: 0
        }
    }

    pub fn ray_intersection(
        &self, 
        loc: Vec3, 
        scale: Vec3, 
        origin: Vec3A, 
        direction: Vec3A
    ) -> Option<f32> {

        let min_corner = Vec3A::new(loc.x - self.width*0.5*scale.x, loc.y, loc.z - self.height*0.5*scale.y);
        let max_corner = Vec3A::new(loc.x + self.width*0.5*scale.x, loc.y, loc.z + self.height*0.5*scale.y);

        let inv_dir = direction.recip();
        
        let t1 = (min_corner - origin) * inv_dir;
        let t2 = (max_corner - origin) * inv_dir;
        
        let t_min = Vec3A::min(t1, t2);
        let t_max = Vec3A::max(t1, t2);
        
        let t_enter = t_min.max_element();
        let t_exit = t_max.min_element();
        
        let hit: bool = t_enter <= t_exit && t_exit >= 0.0;
        if hit {
            return Some(t_enter.max(0.0));
        } else {
            return None;
        }
    }
}

