use mesh_loader::Scene;
use std::path::Path;

fn load_art_gallery_model() {
    let path = Path::new("../Models/The art gallery.stl");

    // Sử dụng trực tiếp Scene::from_path
    match Scene::from_path(path) {
        Ok(scene) => {
            println!("Đã nạp thành công file STL!");
            for mesh in scene.meshes {
                println!("Mesh name: {}", mesh.name);
                println!("Số lượng vertices: {}", mesh.vertices.len());
                // Dữ liệu này bạn sẽ dùng để vẽ trong OpenGL [cite: 5, 7]
            }
        }
        Err(e) => {
            eprintln!("Lỗi khi nạp file: {:?}", e);
        }
    }
}
