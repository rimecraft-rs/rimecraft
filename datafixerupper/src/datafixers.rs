pub mod kinds {
    pub trait App<K, A> {}

    pub trait K1 {
        fn new() -> Self;
    }
    pub trait Kind1<F: K1, Mu: Kind1Mu>: App<Mu, F> {}

    pub trait Kind1Mu: K1 {}
}

pub mod util {
    use super::kinds::{App, K1};

    #[derive(PartialEq)]
    pub enum Either<L, R> {
        Left(L),
        Right(R),
    }

    pub struct EitherMu<R> {
        _n: Option<R>,
    }
    impl<R> K1 for EitherMu<R> {
        fn new() -> EitherMu<R> {
            Self { _n: None }
        }
    }

    impl<L, R> Either<L, R> {
        pub fn map_both<C, D>(
            &self,
            f1: Box<impl Fn(&L) -> C>,
            f2: Box<impl Fn(&R) -> D>,
        ) -> Either<C, D> {
            match self {
                Either::Left(value) => {
                    let left = f1(value);
                    Either::Left(left)
                }
                Either::Right(value) => {
                    let right = f2(value);
                    Either::Right(right)
                }
            }
        }

        pub fn map<T>(&self, l: Box<impl Fn(&L) -> T>, r: Box<impl Fn(&R) -> T>) -> T {
            match self {
                Either::Left(value) => l(value),
                Either::Right(value) => r(value),
            }
        }

        pub fn if_left(&self, consumer: Box<impl Fn(&L)>) {
            match self {
                Either::Left(value) => consumer(value),
                _ => (),
            }
        }

        pub fn if_right(&self, consumer: Box<impl Fn(&R)>) {
            match self {
                Either::Right(value) => consumer(value),
                _ => (),
            }
        }

        pub fn if_left_mut(&mut self, consumer: Box<impl Fn(&mut L)>) {
            match self {
                Either::Left(value) => consumer(value),
                _ => (),
            }
        }

        pub fn if_right_mut(&mut self, consumer: Box<impl Fn(&mut R)>) {
            match self {
                Either::Right(value) => consumer(value),
                _ => (),
            }
        }

        pub fn left(&self) -> Option<&L> {
            match self {
                Either::Left(value) => Some(value),
                _ => None,
            }
        }

        pub fn right(&self) -> Option<&R> {
            match self {
                Either::Right(value) => Some(value),
                _ => None,
            }
        }

        pub fn left_mut(&mut self) -> Option<&mut L> {
            match self {
                Either::Left(value) => Some(value),
                _ => None,
            }
        }

        pub fn right_mut(&mut self) -> Option<&mut R> {
            match self {
                Either::Right(value) => Some(value),
                _ => None,
            }
        }
    }

    impl<L: ToString, R: ToString> ToString for Either<L, R> {
        fn to_string(&self) -> String {
            format!(
                "{}[{}]",
                match self {
                    Either::Left(_) => "Left",
                    Either::Right(_) => "Right",
                },
                match self {
                    Either::Left(l) => l.to_string(),
                    Either::Right(r) => r.to_string(),
                }
            )
        }
    }

    impl<L, R> App<EitherMu<R>, L> for Either<L, R> {}
}
