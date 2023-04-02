pub mod fs;

#[derive(PartialEq, Eq)]
pub enum ResourceType {
    CLIENT_RESOURCES,
    SERVER_DATA,
}

impl ResourceType {
    pub fn get_dictionary(&self) -> String {
        match self {
            ResourceType::CLIENT_RESOURCES => String::from("assets"),
            ResourceType::SERVER_DATA => String::from("data"),
        }
    }
}
