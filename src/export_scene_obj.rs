use bevy::prelude::*;
use bevy::mesh::{Mesh, VertexAttributeValues, Indices};
use std::fs::File;
use std::io::Write;

pub fn export_mesh_to_obj(mesh: &Mesh, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Write header
    writeln!(file, "# Exported from Bevy")?;
    writeln!(file, "# Vertices: {}", mesh.count_vertices())?;
    writeln!(file)?;
    
    // Extract positions
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions,
        _ => return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Mesh missing positions or wrong format"
        )),
    };
    
    // Write vertices
    for pos in positions {
        writeln!(file, "v {} {} {}", pos[0], pos[1], pos[2])?;
    }
    writeln!(file)?;
    
    // Extract and write normals (if present)
    if let Some(VertexAttributeValues::Float32x3(normals)) = 
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL) 
    {
        for normal in normals {
            writeln!(file, "vn {} {} {}", normal[0], normal[1], normal[2])?;
        }
        writeln!(file)?;
    }
    
    // Extract and write UVs (if present)
    if let Some(VertexAttributeValues::Float32x2(uvs)) = 
        mesh.attribute(Mesh::ATTRIBUTE_UV_0) 
    {
        for uv in uvs {
            writeln!(file, "vt {} {}", uv[0], uv[1])?;
        }
        writeln!(file)?;
    }
    
    // Extract indices and write faces
    if let Some(indices) = mesh.indices() {
        let indices: Vec<u32> = match indices {
            Indices::U16(idx) => idx.iter().map(|&i| i as u32).collect(),
            Indices::U32(idx) => idx.clone(),
        };
        
        // OBJ indices are 1-based
        for chunk in indices.chunks(3) {
            if chunk.len() == 3 {
                // Format: f v/vt/vn v/vt/vn v/vt/vn
                write!(file, "f")?;
                for &idx in chunk {
                    let i = idx + 1; // OBJ uses 1-based indexing
                    write!(file, " {}/{}/{}", i, i, i)?;
                }
                writeln!(file)?;
            }
        }
    }
    
    Ok(())
}

// For exporting multiple meshes
pub fn export_meshes_to_obj(
    meshes: &[(String, &Mesh)], 
    path: &str
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, "# Exported from Bevy")?;
    writeln!(file, "# Multiple meshes")?;
    writeln!(file)?;
    
    let mut vertex_offset = 0u32;
    
    for (name, mesh) in meshes {
        writeln!(file, "o {}", name)?;
        writeln!(file)?;
        
        // Extract and write positions
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(p)) => p,
            _ => continue,
        };
        
        for pos in positions {
            writeln!(file, "v {} {} {}", pos[0], pos[1], pos[2])?;
        }
        writeln!(file)?;
        
        // Write normals
        if let Some(VertexAttributeValues::Float32x3(normals)) = 
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL) 
        {
            for normal in normals {
                writeln!(file, "vn {} {} {}", normal[0], normal[1], normal[2])?;
            }
            writeln!(file)?;
        }
        
        // Write UVs
        if let Some(VertexAttributeValues::Float32x2(uvs)) = 
            mesh.attribute(Mesh::ATTRIBUTE_UV_0) 
        {
            for uv in uvs {
                writeln!(file, "vt {} {}", uv[0], uv[1])?;
            }
            writeln!(file)?;
        }
        
        // Write faces with offset
        if let Some(indices) = mesh.indices() {
            let indices: Vec<u32> = match indices {
                Indices::U16(idx) => idx.iter().map(|&i| i as u32).collect(),
                Indices::U32(idx) => idx.clone(),
            };
            
            for chunk in indices.chunks(3) {
                if chunk.len() == 3 {
                    write!(file, "f")?;
                    for &idx in chunk {
                        let i = idx + vertex_offset + 1;
                        write!(file, " {}/{}/{}", i, i, i)?;
                    }
                    writeln!(file)?;
                }
            }
        }
        
        vertex_offset += positions.len() as u32;
        writeln!(file)?;
    }
    
    Ok(())
}


fn export_system(
    meshes: Res<Assets<Mesh>>,
    query: Query<(&Mesh3d, &Name)>,
) {
    // Export single mesh
    if let Some((mesh_handle, _)) = query.iter().next() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            export_mesh_to_obj(mesh, "output.obj").unwrap();
        }
    }
    
    // Export multiple meshes
    let mesh_data: Vec<(String, &Mesh)> = query
        .iter()
        .filter_map(|(handle, name)| {
            meshes.get(handle).map(|m| (name.to_string(), m))
        })
        .collect();
    
    export_meshes_to_obj(&mesh_data, "scene.obj").unwrap();
}