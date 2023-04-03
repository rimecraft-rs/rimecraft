use std::path::Path;

use crate::resource::fs::ResourceFileSystem;

pub fn build_resource_file_system(assets_dir: &Path, index_name: &str) -> Option<String> {
    let path = assets_dir.join("objects");
    let builder = ResourceFileSystem::builder();
    let path2 = assets_dir.join(format!("indexes/{index_name}.json"));
    todo!()
}
