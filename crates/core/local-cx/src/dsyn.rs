#![cfg(feature = "dsyn")]

/// Checks whether the given object is an instance of the given descriptor type.
///
/// # Usage
///
/// `$local_context, $object => $descriptor_inner_type`
#[macro_export]
macro_rules! instanceof {
    ($l:expr,$o:expr=>$t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o)
            .contains(<_ as $crate::LocalContext<$crate::__dsyn::Type<$t>>>::acquire($l))
    };
    ($l:expr,$o:expr=>export $t:ty) => {
        <_ as $crate::__dsyn::HoldDescriptors<'_, '_>>::descriptors($o)
            .get(<_ as $crate::LocalContext<$crate::__dsyn::Type<$t>>>::acquire($l))
    };
}
