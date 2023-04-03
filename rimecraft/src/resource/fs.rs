use std::collections::HashMap;

pub struct ResourceFileSystem {
    store_name: String,
    storage: HashMap<String, ResourcePath>,
}

impl ResourceFileSystem {
    pub fn new(name: String, root: &Directory) -> Self {
        let mut storage = HashMap::new();
        let e =
            Self::to_resource_path(&root, String::new(), None, &mut storage, String::from("!r"));
        storage.insert(String::from("!r"), e);
        Self {
            store_name: name,
            storage,
        }
    }

    fn to_resource_path(
        root: &Directory,
        name: String,
        parent: Option<String>,
        map: &mut HashMap<String, ResourcePath>,
        save: String,
    ) -> ResourcePath {
        let resource_path = ResourcePath::new(name, parent.clone(), ResourceFile::Directory, map);
        for ele in &root.files {
            map.insert(
                ele.0.to_owned(),
                ResourcePath::new(
                    ele.0.to_owned(),
                    Some(save.clone()),
                    ResourceFile::File(ele.1.to_owned()),
                    map,
                ),
            );
        }
        for ele in &root.children {
            let e =
                Self::to_resource_path(ele.1, ele.0.clone(), parent.clone(), map, ele.0.clone());
            map.insert(ele.0.clone(), e);
        }
        resource_path
    }

    pub fn builder<'a>() -> ResourceFileSystemBuilder<'a> {
        ResourceFileSystemBuilder::new()
    }
}

pub struct ResourceFileSystemBuilder<'a> {
    root: Directory<'a>,
}

impl<'a> ResourceFileSystemBuilder<'a> {
    pub fn new() -> Self {
        Self {
            root: Directory::default(),
        }
    }

    pub fn with_file(&mut self, directories: Vec<&str>, name: String, path: String) {
        for string in directories {
            self.root = {
                self.root
                    .children
                    .get(string)
                    .unwrap_or(&&Directory::default())
                    .clone()
                    .clone()
            };
        }
        self.root.files.insert(name, path);
    }

    pub fn with_file_default(&mut self, directories: Vec<&str>, path: String) {
        if !directories.is_empty() {
            let i = directories.len() - 1;
            self.with_file(
                {
                    let mut ii = 0;
                    let mut vec: Vec<&str> = Vec::new();
                    while ii < i {
                        vec.push(directories.get(i).unwrap());
                        ii += 1
                    }
                    vec
                },
                directories.get(i).unwrap().to_string(),
                path,
            )
        }
    }

    pub fn build(self, name: String) -> ResourceFileSystem {
        ResourceFileSystem::new(name, &self.root)
    }
}

#[derive(Clone)]
pub struct Directory<'a> {
    pub children: HashMap<String, &'a Directory<'a>>,
    pub files: HashMap<String, String>,
}

impl Default for Directory<'_> {
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
    parent: Option<String>,
    names: Option<Vec<String>>,
    path_string: Option<String>,
    file: ResourceFile,
}

impl ResourcePath {
    pub fn new(
        name: String,
        parent: Option<String>,
        file: ResourceFile,
        map: &HashMap<String, ResourcePath>,
    ) -> Self {
        let mut s = Self {
            name,
            parent,
            names: None,
            path_string: None,
            file,
        };
        s.refresh_names(map);
        s
    }

    fn relativize(path: Option<String>, name: String, map: &HashMap<String, ResourcePath>) -> Self {
        Self::new(name, path, ResourceFile::Relative, map)
    }

    fn refresh_names(&mut self, map: &HashMap<String, ResourcePath>) {
        if self.names.is_none() {
            let mut vec = Vec::new();
            if self.parent.is_some() {
                vec.append(&mut map.get(&self.parent.clone().unwrap()).unwrap().get_names())
            }
            vec.push(self.name.clone());
            self.names = Some(vec.iter().map(|s| String::from(s.to_owned())).collect());
        }
    }

    fn get(&self, name: &str, map: &HashMap<String, ResourcePath>) -> Self {
        let key: String = {
            let mut n = String::new();
            for m in map {
                if self == m.1 {
                    n = m.0.clone()
                }
            }
            n
        };
        match &self.file {
            ResourceFile::Empty | ResourceFile::Relative => Self::new(
                name.to_owned(),
                Some(key.to_owned()),
                self.file.clone(),
                map,
            ),
            ResourceFile::Directory => map
                .get(name)
                .unwrap_or(&Self::new(
                    name.to_owned(),
                    Some(key.to_owned()),
                    ResourceFile::Empty,
                    map,
                ))
                .clone(),
            ResourceFile::File(_) => Self::new(
                name.to_owned(),
                Some(key.to_owned()),
                ResourceFile::Empty,
                map,
            ),
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

    pub fn get_file_name(&self, map: &HashMap<String, ResourcePath>) -> Self {
        Self::relativize(None, self.name.clone(), map)
    }

    pub fn get_parent(&self) -> Option<&str> {
        self.parent.as_deref()
    }

    pub fn get_name_count(self) -> usize {
        self.get_names().len()
    }

    pub fn get_names(&self) -> Vec<String> {
        if self.name.is_empty() {
            return Vec::new();
        }

        let mut vec = Vec::new();
        for s in &self.names.clone().unwrap() {
            vec.push(s.clone());
        }
        vec
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ResourceFile {
    Empty,
    Relative,
    Directory,
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
