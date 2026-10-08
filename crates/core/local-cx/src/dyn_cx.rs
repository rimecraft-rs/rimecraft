//! Dynamic context providers.

#![cfg(feature = "dyn-cx")]

use std::{any::TypeId, borrow::Cow, collections::HashMap, fmt::Debug};

use global_cx::{
    ProvideHasherTy,
    hash::{Specific, Unsecure},
};

use crate::{BaseLocalContext, LocalContext};

type Caller<Cx> = fn(Cx, &mut (dyn FnMut(*const ()) + '_));

/// Global context types for dynamic local contexts.
pub trait DynamicContextCx: ProvideHasherTy<Specific<TypeId, Unsecure>> {}

impl<T: ProvideHasherTy<Specific<TypeId, Unsecure>>> DynamicContextCx for T {}

type Hasher<Cx> = <Cx as ProvideHasherTy<Specific<TypeId, Unsecure>>>::BuildHasher;

/// Local context that can be converted to a dynamic context.
pub trait AsDynamicContext<Cx>: BaseLocalContext
where
    Cx: DynamicContextCx,
{
    /// The inner local context type.
    type InnerContext: BaseLocalContext;

    /// Converts this context to a dynamic context.
    fn as_dynamic_context(&self) -> DynamicContext<'_, Self::InnerContext, Cx>;
}

/// Function table for getting contexts.
#[derive(Debug)]
pub struct ContextTable<LocalCx, Cx>
where
    Cx: DynamicContextCx,
{
    map: HashMap<TypeId, Caller<LocalCx>, Hasher<Cx>>,
}

impl<L, Cx: DynamicContextCx> ContextTable<L, Cx> {
    /// Creates a new context table.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    /// Enables dynamic fetching of a type.
    pub fn enable<T>(&mut self)
    where
        L: LocalContext<T>,
    {
        let ty = typeid::of::<T>();
        self.map.insert(ty, |cx, f| {
            let val = <L as LocalContext<T>>::acquire(cx);
            f(std::ptr::from_ref(&val).cast::<()>())
        });
    }
}

impl<L, Cx: DynamicContextCx> Default for ContextTable<L, Cx> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<L, Cx: DynamicContextCx> Clone for ContextTable<L, Cx> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

/// A dynamic context provider.
#[derive(Debug)]
pub struct DynamicContext<'a, LocalCx, Cx>
where
    Cx: DynamicContextCx,
{
    cx: LocalCx,
    table: Cow<'a, ContextTable<LocalCx, Cx>>,
}

impl<'a, L, Cx: DynamicContextCx> DynamicContext<'a, L, Cx> {
    /// Creates a new dynamic context with owned context table.
    #[inline]
    pub fn new(cx: L, table: ContextTable<L, Cx>) -> Self {
        Self {
            cx,
            table: Cow::Owned(table),
        }
    }

    /// Creates a new dynamic context with borrowed context table.
    #[inline]
    pub fn from_borrowed_table(cx: L, table: &'a ContextTable<L, Cx>) -> Self {
        Self {
            cx,
            table: Cow::Borrowed(table),
        }
    }
}

impl<L, Cx> DynamicContext<'_, L, Cx>
where
    L: BaseLocalContext,
    Cx: DynamicContextCx,
{
    /// Turns this context into an [`UnsafeDynamicContext`].
    ///
    /// # Safety
    ///
    /// The returned type is not safe enough to exist.
    #[inline]
    pub unsafe fn as_unsafe_cx(&self) -> UnsafeDynamicContext<'_> {
        UnsafeDynamicContext(self)
    }
}

impl<L, Cx: DynamicContextCx> BaseLocalContext for &DynamicContext<'_, L, Cx> {}

impl<L, Cx: DynamicContextCx, T> LocalContext<T> for &DynamicContext<'_, L, Cx>
where
    L: LocalContext<T>,
{
    #[inline]
    fn acquire(self) -> T {
        self.cx.acquire()
    }
}

trait ErasedDynCx {
    fn erased_acquire(&self, ty: TypeId, f: &mut (dyn FnMut(*const ()) + '_));
}

impl<L, Cx: DynamicContextCx> ErasedDynCx for DynamicContext<'_, L, Cx>
where
    L: BaseLocalContext,
{
    #[inline]
    fn erased_acquire(&self, ty: TypeId, f: &mut (dyn FnMut(*const ()) + '_)) {
        if let Some(g) = self.table.map.get(&ty) {
            g(self.cx, f)
        }
    }
}

/// A dynamic context provider that is **unsafe to exist**.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UnsafeDynamicContext<'a>(&'a (dyn ErasedDynCx + 'a));

impl BaseLocalContext for UnsafeDynamicContext<'_> {}

impl<T> LocalContext<T> for UnsafeDynamicContext<'_>
where
    T: Copy,
{
    fn acquire(self) -> T {
        let mut val = None;
        self.0.erased_acquire(typeid::of::<T>(), &mut |obj| {
            val = Some(unsafe { *obj.cast::<T>() })
        });
        val.unwrap_or_else(|| {
            panic!(
                "type {} not found for dynamic context",
                std::any::type_name::<T>()
            )
        })
    }
}

impl Debug for UnsafeDynamicContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("UnsafeDynamicContext").finish()
    }
}
