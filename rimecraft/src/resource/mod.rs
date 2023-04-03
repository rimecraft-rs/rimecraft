pub mod fs;

#[derive(PartialEq, Eq)]
pub enum ResourceType {
    ClientResources,
    ServerData,
}

impl ResourceType {
    pub fn get_dictionary(&self) -> String {
        match self {
            ResourceType::ClientResources => String::from("assets"),
            ResourceType::ServerData => String::from("data"),
        }
    }
}
