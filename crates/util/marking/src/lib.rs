//! Smol marker types for identifying objects.

/// A leaked byte pointer that is guaranteed to be unique.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct LeakedPtrMarker(*mut u8);

impl LeakedPtrMarker {
    /// Creates a new leaked pointer marker.
    #[inline]
    pub fn new() -> Self {
        #[cfg(not(miri))]
        {
            Self(Box::into_raw(Box::new(0u8)).cast())
        }

        #[cfg(miri)]
        {
            static ATOMIC_COUNTER: std::sync::atomic::AtomicPtr<u8> =
                std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

            Self(ATOMIC_COUNTER.fetch_byte_add(1, std::sync::atomic::Ordering::Relaxed))
        }
    }

    /// Gets a non-leaked pointer marker reference from this.
    #[inline]
    pub fn as_non_leaked(&self) -> &PtrMarker {
        //SAFETY: same ABI.
        unsafe { *std::ptr::from_ref(self).cast() }
    }
}

impl Default for LeakedPtrMarker {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A non-leaked byte pointer that is guaranteed to be unique.
///
/// This type doesn't implement `Copy` and `Clone` as they break the uniqueness.
/// For allowing multiple markers to exist, see [`LeakedPtrMarker`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PtrMarker(*mut u8);

impl PtrMarker {
    /// Creates a new byte pointer.
    #[inline]
    pub fn new() -> Self {
        Self(Box::into_raw(Box::new(0u8)))
    }
}

impl Default for PtrMarker {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PtrMarker {
    #[inline]
    fn drop(&mut self) {
        let _dropped = unsafe { Box::from_raw(self.0) };
    }
}

unsafe impl Send for LeakedPtrMarker {}
unsafe impl Sync for LeakedPtrMarker {}

unsafe impl Send for PtrMarker {}
unsafe impl Sync for PtrMarker {}
