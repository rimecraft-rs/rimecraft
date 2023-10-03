pub mod math;
pub mod formatting;

/// Cell forbids immutable access to the inner value.
pub struct MutOnly<T> {
    value: T,
}

impl<T> MutOnly<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> From<T> for MutOnly<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

pub trait StringIdentifiable {
    fn as_string(&self) -> String;

    ///2 createCodec() ignored.
    fn crate_codec() -> () {}

    ///1 toKeyable ignored.
    fn to_keyable() -> () {}
}
