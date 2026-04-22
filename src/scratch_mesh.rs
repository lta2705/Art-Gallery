#[cfg(test)]
mod tests {
    use mesh_loader::Scene;
    use std::path::Path;

    #[test]
    fn test_mesh_loader() {
        let p = Path::new("Models/The art gallery.stl");
        if p.exists() {
            let scene = Scene::from_path(p).unwrap();
            let m = &scene.meshes[0];
            // let's print fields to provoke a compiler error showing the struct
            let _name: () = m.name;
        }
    }
}
