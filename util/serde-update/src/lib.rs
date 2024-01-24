#[cfg(feature = "erased")]
pub mod erased;

/// Represent types that are able to be updated
/// through serializing and deserializing.
pub trait Update<'de>: serde::Serialize {
    /// Update this type from a deserializer.
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<'de, T> Update<'de> for T
where
    T: serde::Serialize + serde::Deserialize<'de>,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self = Self::deserialize(deserializer)?;
        Ok(())
    }
}
