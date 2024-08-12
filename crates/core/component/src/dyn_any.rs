use std::any::TypeId;

pub trait Any {
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

impl<T: ?Sized> Any for T {}

impl dyn Any + Send + Sync + '_ {
    pub unsafe fn downcast_ref<T>(&self) -> Option<&T> {
        if self.type_id() == typeid::of::<T>() {
            unsafe { Some(&*(self as *const dyn Any as *const T)) }
        } else {
            None
        }
    }

    pub unsafe fn downcast_mut<T>(&mut self) -> Option<&mut T> {
        if self.type_id() == typeid::of::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Any as *mut T)) }
        } else {
            None
        }
    }
}

pub unsafe fn downcast<'a, T>(
    any: Box<dyn Any + Send + Sync + 'a>,
) -> Result<Box<T>, Box<dyn Any + Send + Sync + 'a>> {
    if any.type_id() == typeid::of::<T>() {
        unsafe { Ok(Box::from_raw(Box::into_raw(any) as *mut T)) }
    } else {
        Err(any)
    }
}
