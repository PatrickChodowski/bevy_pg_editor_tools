use bevy::prelude::*;
use bevy::mesh::{Mesh, VertexAttributeValues, Indices};
use std::fs::File;
use std::io::Write;


// For exporting multiple meshes
pub fn export_scene_to_obj(
    meshes: &Assets<Mesh>,
    query: &Query<(&Mesh3d, &GlobalTransform, &Name)>,
    path: &str,
) -> std::io::Result<()> {

    let mut file = File::create(path)?;
    writeln!(file, "# Exported from Bevy")?;
    writeln!(file, "# Multiple meshes")?;
    writeln!(file)?;
    
    let mut vertex_offset = 0u32;
    let mut normal_offset = 0u32;
    let mut uv_offset = 0u32;

    const OFFSET_X: f32 = -22471.0;
    const OFFSET_Z: f32 = -17500.0;
    
    for (mesh_handle, global_transform, name) in query.iter() {

        let lname = name.to_lowercase();

        if !(lname.contains("tree") | lname.contains("hedeby") | lname.contains("bld") | lname.contains("prop_fence") | lname.contains("mountain")){
            continue;
        }
        // if !(lname.contains("hedeby")){
        //     continue;
        // }

        let Some(mesh) = meshes.get(mesh_handle) else {
            continue;
        };

        info!(".obj writing {}...", name.to_string());

        let (scale, rotation, translation) = global_transform.to_scale_rotation_translation();
        writeln!(file, "o {}", name.to_string())?;

        // Extract and write positions
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(p)) => p,
            _ => continue,
        };
        // Write transformed vertices
        for pos in positions {
            let v = Vec3::new(pos[0], pos[1], pos[2]);
            // Apply scale, rotation, then translation
            let mut transformed = translation + rotation * (v * scale);
            transformed.x += OFFSET_X;
            transformed.z += OFFSET_Z;

            writeln!(file, "v {} {} {}", transformed.x, transformed.y, transformed.z)?;
        }

        let has_normals = if let Some(VertexAttributeValues::Float32x3(normals)) = 
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL) 
        {
            for normal in normals {
                let n = Vec3::new(normal[0], normal[1], normal[2]);
                // Normals are rotated but not translated or scaled uniformly
                let transformed = rotation * n;
                let normalized = transformed.normalize();
                writeln!(file, "vn {} {} {}", normalized.x, normalized.y, normalized.z)?;
            }
            true
        } else {
            false
        };

        let has_uvs = if let Some(VertexAttributeValues::Float32x2(uvs)) = 
            mesh.attribute(Mesh::ATTRIBUTE_UV_0) 
        {
            for uv in uvs {
                writeln!(file, "vt {} {}", uv[0], uv[1])?;
            }
            true
        } else {
            false
        };

        

        // Write faces with proper offsets
        if let Some(indices) = mesh.indices() {
            let indices: Vec<u32> = match indices {
                Indices::U16(idx) => 
                    idx.iter().map(|&i| i as u32).collect(),
                Indices::U32(idx) => idx.clone(),
            };
            
            for chunk in indices.chunks(3) {
                if chunk.len() == 3 {
                    write!(file, "f")?;
                    for &idx in chunk {
                        let v_idx = idx + vertex_offset + 1;
                        let vt_idx = idx + uv_offset + 1;
                        let vn_idx = idx + normal_offset + 1;
                        
                        match (has_uvs, has_normals) {
                            (true, true) => write!(file, " {}/{}/{}", v_idx, vt_idx, vn_idx)?,
                            (true, false) => write!(file, " {}/{}", v_idx, vt_idx)?,
                            (false, true) => write!(file, " {}//{}", v_idx, vn_idx)?,
                            (false, false) => write!(file, " {}", v_idx)?,
                        }
                    }
                    writeln!(file)?;
                }
            }
        }
        
        // Update offsets for next mesh
        vertex_offset += positions.len() as u32;
        if has_normals {
            normal_offset += positions.len() as u32;
        }
        if has_uvs {
            uv_offset += positions.len() as u32;
        }
        
        writeln!(file)?;
    }
    
    Ok(())
}


pub fn export_obj_system(
    meshes: Res<Assets<Mesh>>,
    query: Query<(&Mesh3d, &GlobalTransform, &Name)>,
) {
    info!("Exporting Scene to OBJ");
    if let Err(e) = export_scene_to_obj(&meshes, &query, "assets/obj/scene.obj") {
        error!("Failed to export scene: {}", e);
    } else {
        info!("Scene exported successfully!");
    }
}
