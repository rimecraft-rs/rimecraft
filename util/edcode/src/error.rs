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
pub enum ErrorWithVarI32Err<T> {
    Target(T),
    Var(VarI32TooBigError),
}

impl<T: std::fmt::Display> std::fmt::Display for ErrorWithVarI32Err<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorWithVarI32Err::Target(e) => write!(f, "{}", e),
            ErrorWithVarI32Err::Var(e) => write!(f, "variable integer error: {}", e),
        }
    }
}

impl<T: std::error::Error + 'static> std::error::Error for ErrorWithVarI32Err<T> {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorWithVarI32Err::Target(e) => Some(e),
            ErrorWithVarI32Err::Var(e) => Some(e),
        }
    }
}

impl<T> From<VarI32TooBigError> for ErrorWithVarI32Err<T> {
    #[inline]
    fn from(value: VarI32TooBigError) -> Self {
        Self::Var(value)
    }
}

#[derive(Debug)]
pub enum EitherError<T1, T2> {
    A(T1),
    B(T2),
}

impl<T1: std::fmt::Display, T2: std::fmt::Display> std::fmt::Display for EitherError<T1, T2> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EitherError::A(e) => write!(f, "A error: {}", e),
            EitherError::B(e) => write!(f, "B error: {}", e),
        }
    }
}

impl<T1: std::error::Error + 'static, T2: std::error::Error + 'static> std::error::Error
    for EitherError<T1, T2>
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EitherError::A(e) => Some(e),
            EitherError::B(e) => Some(e),
        }
    }
}
