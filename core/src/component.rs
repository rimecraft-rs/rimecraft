use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use crate::net::NetSync;

/// Represents a type of component that can be attached
/// on [`Components`].
pub trait Attach {
    /// Actions to perform before attaching this component
    /// on components.
    /// By this, you can interact with other components to
    /// implement functions like syncing components.
    ///
    /// Don't attach this component in this function.
    fn on_attach(&mut self, components: &mut Components);
}

pub struct Components {
    components: std::collections::HashMap<crate::Id, (Box<dyn Attach + Send + Sync>, TypeId)>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    pub fn attach<T>(&mut self, id: crate::Id, mut component: T)
    where
        T: Attach + Send + Sync + 'static,
    {
        if self.components.contains_key(&id) {
            panic!("component with id {id} already exist in this components!");
        }

        component.on_attach(self);

        self.components
            .insert(id, (Box::new(component), TypeId::of::<T>()));
    }

    pub fn get<T>(&self, id: &crate::Id) -> Option<&T>
    where
        T: Attach + Send + Sync + 'static,
    {
        self.components
            .get(id)
            .map(|value| {
                if value.1 == TypeId::of::<T>() {
                    Some(unsafe {
                        &*(value.0.deref() as *const (dyn Attach + Send + Sync) as *const T)
                    })
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn get_mut<T>(&mut self, id: &crate::Id) -> Option<&mut T>
    where
        T: Attach + Send + Sync + 'static,
    {
        self.components
            .get_mut(id)
            .map(|value| {
                if value.1 == TypeId::of::<T>() {
                    Some(unsafe {
                        &mut *(value.0.deref_mut() as *mut (dyn Attach + Send + Sync) as *mut T)
                    })
                } else {
                    None
                }
            })
            .flatten()
    }
}

pub struct Synced<T>(pub T)
where
    T: Attach + NetSync;

impl<T> Attach for Synced<T>
where
    T: Attach + NetSync,
{
    fn on_attach(&mut self, components: &mut Components) {}
}
