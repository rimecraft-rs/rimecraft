use std::fmt::Debug;

use ahash::AHashSet;
use component::{map::ComponentMap, ComponentType, RawErasedComponentType};
use rimecraft_global_cx::ProvideIdTy;

/// Access to components of a block entity.
pub struct ComponentsAccess<'env, 'a, Cx>
where
    Cx: ProvideIdTy,
{
    pub(crate) set: &'env mut AHashSet<RawErasedComponentType<'a, Cx>>,
    pub(crate) map: &'env ComponentMap<'a, Cx>,
}

impl<'a, Cx> ComponentsAccess<'_, 'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Gets a component of the given type.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get<T>(&mut self, ty: &ComponentType<'a, T>) -> Option<&T> { unsafe {
        self.set.insert(RawErasedComponentType::from(ty));
        self.map.get(ty)
    }}

    /// Reborrow this access.
    pub fn reborrow(&mut self) -> ComponentsAccess<'_, 'a, Cx> {
        ComponentsAccess {
            set: self.set,
            map: self.map,
        }
    }
}

impl<Cx> Debug for ComponentsAccess<'_, '_, Cx>
where
    Cx: ProvideIdTy<Id: Debug> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentsAccess")
            .field("set", &self.set)
            .field("map", &self.map)
            .finish()
    }
}
