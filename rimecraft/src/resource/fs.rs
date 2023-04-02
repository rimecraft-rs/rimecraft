use std::{collections::HashMap, path::Path};

pub struct ResourceFileSystem {
    store_name: String,
    root: ResourcePath,
}

impl ResourceFileSystem {
    pub fn new(name: String, root: Directory) -> Self {
        Self {
            store_name: name,
            root: Self::to_resource_path(root, String::new(), None),
        }
    }

    fn to_resource_path(
        root: Directory,
        name: String,
        parent: Option<ResourcePath>,
    ) -> ResourcePath {
        let map: HashMap<String, ResourcePath> = HashMap::new();
        let resource_path = ResourcePath::new(name, parent, ResourceFile::Directory(()));
        todo!()
    }
}

pub struct Directory {
    pub children: HashMap<String, Directory>,
    pub files: HashMap<String, String>,
}

impl Default for Directory {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            files: HashMap::new(),
        }
    }
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
    pub fn new(name: String, parent: Option<Self>, file: ResourceFile) -> Self {
        let mut s = Self {
            name,
            parent: parent.map(|r| Box::new(r)),
            names: None,
            path_string: None,
            file,
        };
        s.refresh_names();
        s
    }

    fn relativize(path: Option<Self>, name: String) -> Self {
        Self::new(name, path, ResourceFile::Relative)
    }

    fn refresh_names(&mut self) {
        if self.names.is_none() {
            let mut vec = Vec::new();
            if self.parent.is_some() {
                vec.append(&mut self.parent.unwrap().get_names())
            }
            vec.push(&self.name);
            self.names = Some(vec.iter().map(|s| String::from(s.to_owned())).collect());
        }
    }

    fn get(&self, name: &str) -> Self {
        match &self.file {
            ResourceFile::Empty | ResourceFile::Relative => {
                Self::new(name.to_owned(), Some(self.clone()), self.file.clone())
            }
            ResourceFile::Directory(children) => children
                .get(name)
                .unwrap_or(&Self::new(
                    name.to_owned(),
                    Some(self.clone()),
                    ResourceFile::Empty,
                ))
                .clone(),
            ResourceFile::File(_) => {
                Self::new(name.to_owned(), Some(self.clone()), ResourceFile::Empty)
            }
            _ => unreachable!(),
        }
    }

    pub fn is_absolute(&self) -> bool {
        self.file != ResourceFile::Relative
    }

    pub fn to_file_path(&self) -> Option<&str> {
        if let ResourceFile::File(s) = &self.file {
            Some(&s)
        } else {
            None
        }
    }

    pub fn get_file_name(&self) -> Self {
        Self::relativize(None, self.name.clone())
    }

    pub fn get_parent(&self) -> Option<&Self> {
        self.parent.as_deref()
    }

    pub fn get_name_count(self) -> usize {
        self.get_names().len()
    }

    pub fn get_names(&self) -> Vec<&str> {
        if self.name.is_empty() {
            return Vec::new();
        }

        let mut vec = Vec::new();
        for s in &self.names.unwrap() {
            vec.push(s as &str);
        }
        vec
    }

    pub fn sub_path(&self, i: usize, j: usize) -> Option<Self> {
        let list = self.get_names();
        if j > list.len() || i >= j {
            None
        } else {
            let mut resource_path: Option<Self> = None;
            let mut k = i;
            while k < j {
                resource_path = Some(Self::relativize(
                    resource_path.clone(),
                    list.get(k).unwrap().to_string(),
                ));
                k += 1
            }
            resource_path
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ResourceFile {
    Empty,
    Relative,
    Directory(HashMap<String, ResourcePath>),
    File(String),
}

impl ResourceFile {
    pub fn is_special(&self) -> bool {
        match &self {
            ResourceFile::Empty | ResourceFile::Relative => true,
            _ => false,
        }
    }
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
