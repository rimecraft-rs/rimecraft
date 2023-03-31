pub mod kinds {
    pub trait App<K, A> {}

    pub trait K1 {
        fn new() -> Self;
    }
    pub trait Kind1<F: K1, Mu: Kind1Mu>: App<Mu, F> {}

    pub trait Kind1Mu: K1 {}
}

pub mod products {
    use super::kinds::{App, K1};

    pub struct P1<F: K1, T1> {
        pub t1: Box<dyn App<F, T1>>,
    }

    pub struct P2<F: K1, T1, T2> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
    }

    pub struct P3<F: K1, T1, T2, T3> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
    }

    pub struct P4<F: K1, T1, T2, T3, T4> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
    }

    pub struct P5<F: K1, T1, T2, T3, T4, T5> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
    }

    pub struct P6<F: K1, T1, T2, T3, T4, T5, T6> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
    }

    pub struct P7<F: K1, T1, T2, T3, T4, T5, T6, T7> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
    }

    pub struct P8<F: K1, T1, T2, T3, T4, T5, T6, T7, T8> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
    }

    pub struct P9<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
    }

    pub struct P10<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
    }

    pub struct P11<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
    }

    pub struct P12<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
        pub t12: Box<dyn App<F, T12>>,
    }

    pub struct P13<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
        pub t12: Box<dyn App<F, T12>>,
        pub t13: Box<dyn App<F, T13>>,
    }

    pub struct P14<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
        pub t12: Box<dyn App<F, T12>>,
        pub t13: Box<dyn App<F, T13>>,
        pub t14: Box<dyn App<F, T14>>,
    }

    pub struct P15<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
        pub t12: Box<dyn App<F, T12>>,
        pub t13: Box<dyn App<F, T13>>,
        pub t14: Box<dyn App<F, T14>>,
        pub t15: Box<dyn App<F, T15>>,
    }

    pub struct P16<F: K1, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16> {
        pub t1: Box<dyn App<F, T1>>,
        pub t2: Box<dyn App<F, T2>>,
        pub t3: Box<dyn App<F, T3>>,
        pub t4: Box<dyn App<F, T4>>,
        pub t5: Box<dyn App<F, T5>>,
        pub t6: Box<dyn App<F, T6>>,
        pub t7: Box<dyn App<F, T7>>,
        pub t8: Box<dyn App<F, T8>>,
        pub t9: Box<dyn App<F, T9>>,
        pub t10: Box<dyn App<F, T10>>,
        pub t11: Box<dyn App<F, T11>>,
        pub t12: Box<dyn App<F, T12>>,
        pub t13: Box<dyn App<F, T13>>,
        pub t14: Box<dyn App<F, T14>>,
        pub t15: Box<dyn App<F, T15>>,
        pub t16: Box<dyn App<F, T16>>,
    }
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
