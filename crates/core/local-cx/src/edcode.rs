#![cfg(feature = "edcode")]

use edcode2::{Buf, BufMut};

use crate::WithLocalCx;

impl<T, Cx> Buf for WithLocalCx<T, Cx>
where
    T: Buf,
{
    #[inline(always)]
    fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    #[inline(always)]
    fn chunk(&self) -> &[u8] {
        self.inner.chunk()
    }

    #[inline(always)]
    fn advance(&mut self, cnt: usize) {
        self.inner.advance(cnt)
    }

    #[inline(always)]
    fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize {
        self.inner.chunks_vectored(dst)
    }
}

unsafe impl<T, Cx> BufMut for WithLocalCx<T, Cx>
where
    T: BufMut,
{
    #[inline(always)]
    fn remaining_mut(&self) -> usize {
        self.inner.remaining_mut()
    }

    #[inline(always)]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.inner.advance_mut(cnt)
    }

    #[inline(always)]
    fn chunk_mut(&mut self) -> &mut edcode2::UninitSlice {
        self.inner.chunk_mut()
    }

    #[inline(always)]
    fn put<T1: Buf>(&mut self, src: T1)
    where
        Self: Sized,
    {
        self.inner.put(src)
    }

    #[inline(always)]
    fn put_slice(&mut self, src: &[u8]) {
        self.inner.put_slice(src)
    }

    #[inline(always)]
    fn put_bytes(&mut self, val: u8, cnt: usize) {
        self.inner.put_bytes(val, cnt)
    }
}
