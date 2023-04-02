use std::collections::HashMap;

pub struct ResourceFileSystem {
    store_name: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ResourcePath {
    name: String,
    parent: Option<Box<ResourcePath>>,
    names: Option<Vec<String>>,
    path_string: Option<String>,
    file: ResourceFile,
}

impl ResourcePath {
    pub fn new(name: String, parent: Option<ResourcePath>, file: ResourceFile) -> Self {
        Self {
            name,
            parent: parent.map(|r| Box::new(r)),
            names: None,
            path_string: None,
            file,
        }
    }

    fn relativize(path: Option<ResourcePath>, name: String) -> Self {
        Self::new(name, path, ResourceFile::Relative)
    }

    pub fn is_absolute(&self) -> bool {
        self.file != ResourceFile::Relative
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ResourceFile {
    Empty,
    Relative,
    Directory(HashMap<String, ResourcePath>),
    File(String),
}

impl ToString for ResourceFile {
    fn to_string(&self) -> String {
        match &self {
            ResourceFile::Empty => String::from("empty"),
            ResourceFile::Relative => String::from("relative"),
            _ => String::new(),
        }
    }
}
