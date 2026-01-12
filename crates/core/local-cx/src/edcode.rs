//! edcode support for local context.

#![cfg(feature = "edcode")]

use bytes::buf::{Limit, Take};
use edcode2::{Buf, BufMut};

use crate::{ForwardToWithLocalCx, WithLocalCx};

impl<T, Cx> Buf for WithLocalCx<T, Cx>
where
    T: Buf,
{
    #[inline]
    fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.inner.chunk()
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        self.inner.advance(cnt)
    }

    #[inline]
    fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize {
        self.inner.chunks_vectored(dst)
    }
}

unsafe impl<T, Cx> BufMut for WithLocalCx<T, Cx>
where
    T: BufMut,
{
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.inner.remaining_mut()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        unsafe { self.inner.advance_mut(cnt) }
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut edcode2::UninitSlice {
        self.inner.chunk_mut()
    }

    #[inline]
    fn put<T1: Buf>(&mut self, src: T1)
    where
        Self: Sized,
    {
        self.inner.put(src)
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.inner.put_slice(src)
    }

    #[inline]
    fn put_bytes(&mut self, val: u8, cnt: usize) {
        self.inner.put_bytes(val, cnt)
    }
}

impl<T> ForwardToWithLocalCx for Limit<T>
where
    T: ForwardToWithLocalCx<Forwarded: BufMut>,
{
    type Forwarded = Limit<T::Forwarded>;
    type LocalCx = T::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.into_inner().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.limit(limit),
        }
    }
}

impl<'a, T> ForwardToWithLocalCx for &'a Limit<T>
where
    &'a T: ForwardToWithLocalCx<Forwarded: BufMut>,
{
    type Forwarded = Limit<<&'a T as ForwardToWithLocalCx>::Forwarded>;
    type LocalCx = <&'a T as ForwardToWithLocalCx>::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.get_ref().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.limit(limit),
        }
    }
}

impl<'a, T> ForwardToWithLocalCx for &'a mut Limit<T>
where
    &'a mut T: ForwardToWithLocalCx<Forwarded: BufMut>,
{
    type Forwarded = Limit<<&'a mut T as ForwardToWithLocalCx>::Forwarded>;
    type LocalCx = <&'a mut T as ForwardToWithLocalCx>::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.get_mut().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.limit(limit),
        }
    }
}

impl<T> ForwardToWithLocalCx for Take<T>
where
    T: ForwardToWithLocalCx<Forwarded: Buf>,
{
    type Forwarded = Take<T::Forwarded>;
    type LocalCx = T::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.into_inner().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.take(limit),
        }
    }
}

impl<'a, T> ForwardToWithLocalCx for &'a Take<T>
where
    &'a T: ForwardToWithLocalCx<Forwarded: Buf>,
{
    type Forwarded = Take<<&'a T as ForwardToWithLocalCx>::Forwarded>;
    type LocalCx = <&'a T as ForwardToWithLocalCx>::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.get_ref().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.take(limit),
        }
    }
}

impl<'a, T> ForwardToWithLocalCx for &'a mut Take<T>
where
    &'a mut T: ForwardToWithLocalCx<Forwarded: Buf>,
{
    type Forwarded = Take<<&'a mut T as ForwardToWithLocalCx>::Forwarded>;
    type LocalCx = <&'a mut T as ForwardToWithLocalCx>::LocalCx;

    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        let (limit, inner) = (self.limit(), self.get_mut().forward());
        WithLocalCx {
            local_cx: inner.local_cx,
            inner: inner.inner.take(limit),
        }
    }
}
