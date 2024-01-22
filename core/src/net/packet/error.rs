#[derive(Debug)]
pub struct UnknownConnectionIntentError(pub i32);

impl std::fmt::Display for UnknownConnectionIntentError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown connection intent: {}", self.0)
    }
}

impl std::error::Error for UnknownConnectionIntentError {}

#[derive(Debug)]
pub struct PayloadTooLargeError {
    pub max: usize,
}

impl std::fmt::Display for PayloadTooLargeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "payload may not be larger than {} bytes", self.max)
    }
}

impl std::error::Error for PayloadTooLargeError {}
