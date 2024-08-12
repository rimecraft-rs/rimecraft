use std::any::TypeId;

pub trait Any {
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

impl<T: ?Sized> Any for T {}
