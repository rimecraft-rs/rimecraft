use std::path::Path;

pub fn build_resource_file_system(assets_dir: &Path, index_name: &str) -> Option<impl AsRef<Path>> {
    let path = assets_dir.join("objects");
    todo!()
}
