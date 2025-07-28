//! Dsyn support for local context.

#![cfg(feature = "dsyn")]

/// Acquires a descriptor type from the given local context.
///
/// # Usage
///
/// `$(cached $cache_storage,)? $local_context => $descriptor_inner_type`
#[macro_export]
macro_rules! dsyn_ty {
    ($l:expr=>$t:ty) => {
        <_ as $crate::LocalContext<$crate::__dsyn::Type<$t>>>::acquire($l)
    };
    (cached $c:expr,$l:expr=>$t:ty) => {
        <_ as $crate::dsyn::DescriptorTypeCache<$t>>::get_or_cache($c, || {
            <_ as $crate::LocalContext<$crate::__dsyn::Type<$t>>>::acquire($l)
        })
    };
}

/// Checks whether the given object is an instance of the given descriptor type.
///
/// # Usage
///
/// `$(cached $cache_storage,)? $local_context, $object => $(export)? $descriptor_inner_type`
#[macro_export]
macro_rules! dsyn_instanceof {
    ($l:expr,$o:expr=>$t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o).contains($crate::dsyn_ty!($l=>$t))
    };
    ($l:expr,$o:expr=>export $t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o).get($crate::dsyn_ty!($l=>$t))
    };
    (cached $c:expr,$l:expr,$o:expr=>$t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o).contains($crate::dsyn_ty!(cached $c,$l=>$t))
    };
    (cached $c:expr,$l:expr,$o:expr=>export $t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o).get($crate::dsyn_ty!(cached $c,$l=>$t))
    };
}

/// A cache for descriptor types.
pub trait DescriptorTypeCache<T> {
    /// Gets a descriptor type from the cache or caches it.
    fn get_or_cache<F>(&self, f: F) -> dsyn::Type<T>
    where
        F: FnOnce() -> dsyn::Type<T>;
}
