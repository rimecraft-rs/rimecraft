#[derive(Debug)]
pub struct VarI32TooBigError;

impl std::fmt::Display for VarI32TooBigError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "variable i32 too big")
    }
}

impl std::error::Error for VarI32TooBigError {}

#[derive(Debug)]
pub enum ErrorWithVarI32Len<T> {
    Target(T),
    Len(VarI32TooBigError),
}

impl<T: std::fmt::Display> std::fmt::Display for ErrorWithVarI32Len<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorWithVarI32Len::Target(e) => write!(f, "{}", e),
            ErrorWithVarI32Len::Len(e) => write!(f, "variable length error: {}", e),
        }
    }
}

impl<T: std::error::Error + 'static> std::error::Error for ErrorWithVarI32Len<T> {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorWithVarI32Len::Target(e) => Some(e),
            ErrorWithVarI32Len::Len(e) => Some(e),
        }
    }
}

impl<T> From<VarI32TooBigError> for ErrorWithVarI32Len<T> {
    #[inline]
    fn from(value: VarI32TooBigError) -> Self {
        Self::Len(value)
    }
}

#[derive(Debug)]
pub enum KvError<K, V> {
    Key(K),
    Value(V),
}

impl<K: std::fmt::Display, V: std::fmt::Display> std::fmt::Display for KvError<K, V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvError::Key(e) => write!(f, "key error: {}", e),
            KvError::Value(e) => write!(f, "value error: {}", e),
        }
    }
}

impl<K: std::error::Error + 'static, V: std::error::Error + 'static> std::error::Error
    for KvError<K, V>
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KvError::Key(e) => Some(e),
            KvError::Value(e) => Some(e),
        }
    }
}
