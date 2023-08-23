use std::any::TypeId;

pub trait Attach {
    fn pre_attach(&mut self, components: &mut Components);
}

pub struct Components {
    components: <(TypeId, Box<dyn Attach + Send + Sync>)>,
}

impl Components {}
