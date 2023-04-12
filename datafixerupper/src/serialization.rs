use std::{
    fmt::Debug,
    hash::Hash,
    ops::Add,
    option::{Iter, IterMut},
};

use crate::datafixers::{
    kinds::{App, K1},
    util::Either,
};

pub trait Keyable {
    fn keys<T>(&self, ops: &impl DynamicOps<T>) -> Iter<T>;
    fn keys_mut<T>(&mut self, ops: &impl DynamicOps<T>) -> IterMut<T>;
}

pub trait DynamicOps<T> {
    fn empty(&self) -> T;
    // TODO: more
}

pub struct DataResult<R> {
    result: Either<R, PartialResult<R>>,
    lifecycle: Lifecycle,
}

impl<R> DataResult<R> {
    fn new(result: Either<R, PartialResult<R>>, lifecycle: Lifecycle) -> Self {
        Self { result, lifecycle }
    }

    pub fn success(result: R, lifecycle: Lifecycle) -> Self {
        Self::new(Either::Left(result), lifecycle)
    }

    pub fn success_default(result: R) -> Self {
        Self::success(result, Lifecycle::Experimental)
    }

    pub fn error(message: String, partial_result: R, lifecycle: Lifecycle) -> Self {
        Self {
            result: Either::Right(PartialResult::new(message, Some(partial_result))),
            lifecycle,
        }
    }

    pub fn error_no_result(message: String, lifecycle: Lifecycle) -> Self {
        Self {
            result: Either::Right(PartialResult::new(message, None)),
            lifecycle,
        }
    }

    pub fn error_default(message: String, partial_result: R) -> Self {
        Self::error(message, partial_result, Lifecycle::Experimental)
    }

    pub fn error_default_no_result(message: String) -> Self {
        Self::error_no_result(message, Lifecycle::Experimental)
    }

    pub fn get(&self) -> &Either<R, PartialResult<R>> {
        &self.result
    }

    pub fn result(&self) -> Option<&R> {
        self.result.left()
    }

    pub fn lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }
}

pub struct DataResultMu;
impl K1 for DataResultMu {
    fn new() -> Self {
        Self
    }
}

impl<R> App<DataResultMu, R> for DataResult<R> {}

#[derive(PartialEq, Eq, Hash)]
pub struct PartialResult<R> {
    message: String,
    partial_result: Option<R>,
}

impl<R> PartialResult<R> {
    pub fn new(message: String, partial_result: Option<R>) -> PartialResult<R> {
        Self {
            message,
            partial_result,
        }
    }

    pub fn map<R2>(&self, function: Box<impl Fn(&R) -> R2>) -> PartialResult<R2> {
        PartialResult::new(
            self.message.to_owned(),
            self.partial_result.as_ref().map(function),
        )
    }

    pub fn flat_map<R2>(
        &self,
        function: Box<impl Fn(&R) -> PartialResult<R2>>,
    ) -> PartialResult<R2> {
        if let Some(rs) = &self.partial_result {
            let result = function(rs);
            PartialResult::new(
                format!("{}; {}", self.message(), result.message()),
                result.partial_result,
            )
        } else {
            PartialResult::new(self.message.clone(), None)
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl<R: ToString> ToString for PartialResult<R> {
    fn to_string(&self) -> String {
        format!(
            "DynamicException[{} {}]",
            &self.message,
            match &self.partial_result {
                Some(r) => r.to_string(),
                None => "empty".to_string(),
            }
        )
    }
}

pub enum Instance {
    Instance,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
    Deprecated(i32),
}

impl Lifecycle {
    pub fn since(&self) -> i32 {
        match self {
            Lifecycle::Deprecated(s) => *s,
            _ => unreachable!(),
        }
    }
}

impl Add for Lifecycle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Experimental, Self::Experimental) => Self::Experimental,
            (Self::Deprecated(s), Self::Deprecated(o)) => {
                if o < s {
                    Self::Deprecated(o)
                } else {
                    Self::Deprecated(s)
                }
            }
            (_, Self::Deprecated(o)) => Self::Deprecated(o),
            _ => Self::Stable,
        }
    }
}

impl ToString for Lifecycle {
    fn to_string(&self) -> String {
        match &self {
            Lifecycle::Stable => String::from("Stable"),
            Lifecycle::Experimental => String::from("Experimental"),
            Lifecycle::Deprecated(_) => "Deprecated".to_string(),
        }
    }
}

pub trait Encoder<A> {
    fn encode<T>(&self, input: A);
}
