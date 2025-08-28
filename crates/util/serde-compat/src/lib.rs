//! serde implementation that matches Minecraft codecs.

#[cfg(feature = "uuid")]
mod uuid;

/// Codec following vanilla Minecraft.
#[derive(Debug)]
#[repr(transparent)]
pub struct VanillaCodec<T>(pub T);

/// `IntStream` codec.
#[derive(Debug)]
#[repr(transparent)]
pub struct IntStreamCodec<T>(pub T);

/// `String` codec.
#[derive(Debug)]
#[repr(transparent)]
pub struct StringCodec<T>(pub T);
