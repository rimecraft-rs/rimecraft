//! Primitive types with pointer-layout for use in descriptor sets.

#[allow(non_upper_case_globals)]
#[allow(unused)]
mod __paddings {
    macro_rules! ints {
        ($($t:ty,$n:ident),*$(,)?) => {
            $(
                pub const $n: $t = 0;
            )*
        };
    }

    ints! {
        u8,  PointerLikeU8,
        u16, PointerLikeU16,
        u32, PointerLikeU32,
        u64, PointerLikeU64,
        i8,  PointerLikeI8,
        i16, PointerLikeI16,
        i32, PointerLikeI32,
        i64, PointerLikeI64,
    }

    pub const PointerLikeBool: bool = false;
    pub const PointerLikeF32: f32 = 0.0;
    pub const PointerLikeF64: f64 = 0.0;
}

macro_rules! impl_primitives {
    ($($(#[$m:meta])*$n:ident($t:ty);)*) => {
    $(
        #[cfg_attr(target_pointer_width = "16", repr(align(2)))]
        #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
        #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
        $(#[$m])*
        pub struct $n(
            $t,
            [$t; { std::mem::size_of::<*const ()>() / std::mem::size_of::<$t>() - 1 }],
        );

        impl $n {
            #[inline]
            #[doc = "Creates a new pointer-like type with the given value."]
            pub const fn new(value: $t) -> Self {
                Self(value, [__paddings::$n; std::mem::size_of::<*const ()>() / std::mem::size_of::<$t>() - 1])
            }

            #[inline]
            #[doc = "Returns the inner value of this pointer-like type."]
            pub const fn inner(self) -> $t {
                self.0
            }
        }

        impl From<$t> for $n {
            #[inline]
            fn from(value: $t) -> Self {
                Self::new(value)
            }
        }

        impl From<$n> for $t {
            #[inline]
            fn from(value: $n) -> Self {
                value.0
            }
        }
    )*
    };
}

#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl_primitives! {
    #[doc = "Pointer-like u8 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeU8(u8);

    #[doc = "Pointer-like u16 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeU16(u16);

    #[doc = "Pointer-like i8 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeI8(i8);

    #[doc = "Pointer-like i16 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeI16(i16);


    #[doc = "Pointer-like bool type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeBool(bool);


}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_primitives! {
    #[doc = "Pointer-like u32 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeU32(u32);

    #[doc = "Pointer-like i32 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeI32(i32);

    #[doc = "Pointer-like f32 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq)]
    PointerLikeF32(f32);
}

#[cfg(target_pointer_width = "64")]
impl_primitives! {
    #[doc = "Pointer-like u64 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeU64(u64);

    #[doc = "Pointer-like i64 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq, Eq)]
    PointerLikeI64(i64);

    #[doc = "Pointer-like f64 type."]
    #[derive(Debug, Copy, Default, Clone, PartialEq)]
    PointerLikeF64(f64);
}
