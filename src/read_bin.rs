use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

pub fn print_mesh_dimensions(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>, &Name), With<crate::UnprocessedMesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh_handle, name) in &query {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let mut min = Vec3::splat(f32::MAX);
                let mut max = Vec3::splat(f32::MIN);

                for pos in positions {
                    let p = Vec3::from_slice(pos);
                    min = min.min(p);
                    max = max.max(p);
                }

                let size = max - min;
                let center = (max + min) / 2.0;

                println!("--- [PHÂN TÍCH MESH: {}] ---", name);
                println!("Entity: {:?}", entity);
                println!(
                    "Kích thước thực tế: {:.3} x {:.3} x {:.3}",
                    size.x, size.y, size.z
                );
                println!("Tâm cục bộ: {:?}", center);
                println!("----------------------------------------------");

                commands.entity(entity).remove::<crate::UnprocessedMesh>();
            }
        }
    }
}
