//! Dynamic context providers.

#![cfg(feature = "dyn-cx")]

use std::{any::TypeId, borrow::Cow, fmt::Debug};

use ahash::AHashMap;

use crate::{BaseLocalContext, LocalContext};

type Caller<Cx> = fn(Cx, &mut (dyn FnMut(*const ()) + '_));

/// Local context that can be converted to a dynamic context.
pub trait AsDynamicContext: BaseLocalContext {
    /// The inner local context type.
    type InnerContext: BaseLocalContext;

    /// Converts this context to a dynamic context.
    fn as_dynamic_context(&self) -> DynamicContext<'_, Self::InnerContext>;
}

/// Function table for getting contexts.
#[derive(Debug)]
pub struct ContextTable<LocalCx> {
    map: AHashMap<TypeId, Caller<LocalCx>>,
}

impl<Cx> ContextTable<Cx> {
    /// Creates a new context table.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }

    /// Enables dynamic fetching of a type.
    pub fn enable<T>(&mut self)
    where
        Cx: LocalContext<T>,
    {
        let ty = typeid::of::<T>();
        self.map.insert(ty, |cx, f| {
            let val = <Cx as LocalContext<T>>::acquire(cx);
            f(std::ptr::from_ref(&val).cast::<()>())
        });
    }
}

impl<Cx> Default for ContextTable<Cx> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Cx> Clone for ContextTable<Cx> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

/// A dynamic context provider.
#[derive(Debug)]
pub struct DynamicContext<'a, LocalCx> {
    cx: LocalCx,
    table: Cow<'a, ContextTable<LocalCx>>,
}

impl<'a, Cx> DynamicContext<'a, Cx> {
    /// Creates a new dynamic context with owned context table.
    #[inline]
    pub fn new(cx: Cx, table: ContextTable<Cx>) -> Self {
        Self {
            cx,
            table: Cow::Owned(table),
        }
    }

    /// Creates a new dynamic context with borrowed context table.
    #[inline]
    pub fn from_borrowed_table(cx: Cx, table: &'a ContextTable<Cx>) -> Self {
        Self {
            cx,
            table: Cow::Borrowed(table),
        }
    }
}

impl<Cx> DynamicContext<'_, Cx>
where
    Cx: BaseLocalContext,
{
    /// Turns this context into an [`UnsafeDynamicContext`].
    ///
    /// # Safety
    ///
    /// The returned type is not safe enough to exist.
    #[inline(always)]
    pub unsafe fn as_unsafe_cx(&self) -> UnsafeDynamicContext<'_> {
        UnsafeDynamicContext(self)
    }
}

impl<Cx> BaseLocalContext for &DynamicContext<'_, Cx> {}

impl<Cx, T> LocalContext<T> for &DynamicContext<'_, Cx>
where
    Cx: LocalContext<T>,
{
    #[inline]
    fn acquire(self) -> T {
        self.cx.acquire()
    }
}

trait ErasedDynCx {
    fn erased_acquire(&self, ty: TypeId, f: &mut (dyn FnMut(*const ()) + '_));
}

impl<Cx> ErasedDynCx for DynamicContext<'_, Cx>
where
    Cx: BaseLocalContext,
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
