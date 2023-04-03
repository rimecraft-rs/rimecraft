use crate::{
    resource::fs::ResourceFileSystem,
    util::{into_option, json_helper},
};
use std::{fs::File, io::Read, path::Path, str::FromStr};
use substring::Substring;

pub fn build_resource_file_system(
    assets_dir: &Path,
    index_name: &str,
) -> Option<ResourceFileSystem> {
    let path = assets_dir.join("objects");
    let mut builder = ResourceFileSystem::builder();
    let path2 = assets_dir.join(format!("indexes/{index_name}.json"));
    let mut file = into_option(File::open(path2))?;
    let json = into_option(serde_json::Value::from_str(&{
        let mut s = String::new();
        let _result = file.read_to_string(&mut s);
        s
    }))?;
    let json_object = json.as_object()?;
    drop(file);
    if let Some(j) = into_option(json_helper::get_object(&json_object, "objects")) {
        for entry in j {
            let json_object_3 = entry.1.as_object()?;
            let list: Vec<&str> = entry.0.split('/').collect();
            let string = into_option(json_helper::get_str(json_object_3, "hash")).unwrap();
            let path3 = path.join(format!("{}/{string}", string.substring(0, 2)));
            builder.with_file_default(list, path3.to_str().unwrap().to_string());
        }
    }
    Some(builder.build(format!("index-{index_name}")))
}
