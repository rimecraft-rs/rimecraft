/// Optional state result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum StateOption<T> {
    /// A state.
    Some(T),
    /// The `void_air` variant of the state.
    Void,
    /// No state available.
    None,
}

macro_rules! fill_match {
    ($s:expr, $e:expr) => {
        match $s {
            StateOption::Some(val) => StateOption::Some($e(val)),
            StateOption::Void => StateOption::Void,
            StateOption::None => StateOption::None,
        }
    };
}

impl<T> StateOption<T> {
    /// Maps this optional state to another type.
    pub fn map<O, F>(self, mapper: F) -> StateOption<O>
    where
        F: FnOnce(T) -> O,
    {
        fill_match!(self, mapper)
    }
}

impl<T> From<Option<T>> for StateOption<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => StateOption::Some(val),
            None => StateOption::None,
        }
    }
}
