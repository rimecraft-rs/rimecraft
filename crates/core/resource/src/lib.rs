//! Minecraft resource management.

pub trait ResourceReloader {
    type Result;

    async fn reload(&self) -> Self::Result;
}

pub trait ResourceFactory {
    fn get(&self, id: &str);
}
