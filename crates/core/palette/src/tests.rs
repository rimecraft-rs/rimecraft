use std::marker::PhantomData;

use crate::{IndexFromRaw, IndexToRaw, Palette, Strategy};
use rimecraft_maybe::Maybe;

struct List<const N: usize>;

// Just blanket impl.
impl<const N: usize> IndexToRaw<&u8> for List<N> {
    fn raw_id(&self, entry: &u8) -> Option<usize> {
        (entry == &36).then_some(N)
    }
}
impl<'a, 'b, const N: usize> IndexFromRaw<'a, Maybe<'b, u8>> for List<N> {
    fn of_raw(&'a self, _id: usize) -> Option<Maybe<'b, u8>> {
        None
    }
}
impl<const N: usize> IndexFromRaw<'_, u8> for List<N> {
    fn of_raw(&self, id: usize) -> Option<u8> {
        (id == N).then_some(36)
    }
}

struct Iter<'a>(PhantomData<&'a ()>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        0
    }
}
impl<'a, const N: usize> IntoIterator for &'a List<N> {
    type Item = &'a u8;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(PhantomData)
    }
}

#[test]
fn singular() {
    let mut palette = Palette::new(Strategy::Singular, 0, List::<0>, Vec::<u8>::new());

    assert_eq!(palette.len(), 0);
    assert_eq!(palette.index_or_insert(36), Ok(0));
    assert!(matches!(palette.get(0), Some(Maybe::Borrowed(&36))));
    let mut iter = palette.iter();
    assert!(matches!(iter.next(), Some(36)));
    assert!(iter.next().is_none());
    assert_eq!(palette.len(), 1);
    assert_eq!(palette.config(), (Strategy::Singular, 0));
}

macro_rules! helper {
    ($pascal:ident, $snake:ident) => {
        #[test]
        fn $snake() {
            let mut palette = Palette::new(Strategy::$pascal, 2, List::<0>, vec![36u8, 39u8]);

            // For easier diagnosis.
            assert_eq!(palette.len(), 2, "initial length failed");
            assert_eq!(
                palette.index_or_insert(140),
                Ok(2),
                "index_or_insert #1 failed"
            );
            assert_eq!(
                palette.index_or_insert(4),
                Ok(3),
                "index_or_insert #2 failed"
            );
            assert_eq!(
                palette.index_or_insert(114),
                Err((3, 114)),
                "index_or_insert #3 failed"
            );
            assert!(
                matches!(palette.get(0), Some(Maybe::Borrowed(&36))),
                "get failed"
            );
            assert_eq!(
                palette.iter().collect::<Vec<&u8>>(),
                vec![&36, &39, &140, &4],
                "iteration failed"
            );
            assert_eq!(palette.len(), 4, "final length failed");
            assert_eq!(palette.config(), (Strategy::$pascal, 2), "config failed");
        }
    };
}

helper!(Array, array);
helper!(BiMap, bi_map);

#[test]
fn direct() {
    struct List(Vec<u8>);

    impl IndexToRaw<&u8> for List {
        fn raw_id(&self, entry: &u8) -> Option<usize> {
            self.0.iter().position(|val| val == entry)
        }
    }
    impl<'a: 'b, 'b> IndexFromRaw<'a, Maybe<'b, u8>> for List {
        fn of_raw(&'a self, id: usize) -> Option<Maybe<'b, u8>> {
            self.0.get(id).map(Maybe::Borrowed)
        }
    }

    struct Iter<'a>(std::slice::Iter<'a, u8>);

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a u8;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }
    impl ExactSizeIterator for Iter<'_> {
        fn len(&self) -> usize {
            self.0.len()
        }
    }
    impl<'a> IntoIterator for &'a List {
        type Item = &'a u8;

        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Self::IntoIter {
            Iter(self.0.iter())
        }
    }

    let palette = Palette::new(
        Strategy::Direct,
        36,
        List(vec![36, 39, 140, 4]),
        Vec::<u8>::new(),
    );

    assert_eq!(palette.len(), 4);
    assert_eq!(palette.index(&36), Some(0));
    assert_eq!(palette.index(&39), Some(1));
    assert_eq!(palette.index(&140), Some(2));
    assert_eq!(palette.index(&4), Some(3));
    assert!(matches!(palette.get(0), Some(Maybe::Borrowed(&36))));
    assert_eq!(
        palette.iter().collect::<Vec<&u8>>(),
        vec![&36, &39, &140, &4]
    );
    assert_eq!(palette.config(), (Strategy::Direct, 0));
}

#[cfg(feature = "edcode")]
mod edcode {
    use super::List;
    use crate::{Palette, Strategy};
    use edcode2::{Decode, Encode};
    use rimecraft_maybe::Maybe;

    macro_rules! helper {
        ($pascal:ident,$snake:ident,$num:literal) => {
            #[test]
            fn $snake() {
                let src = Palette::new(Strategy::$pascal, 0, List::<$num>, vec![36u8]);
                let mut buf = Vec::<u8>::new();
                src.encode(&mut buf).expect("encode failed");
                let mut dest = Palette::new(Strategy::Singular, 0, List::<$num>, vec![114u8]);
                dest.decode_in_place(buf.as_ref()).expect("decode failed");
                assert!(
                    matches!(dest.get(0), Some(Maybe::Borrowed(&36))),
                    "edcode consistency check failed"
                );
            }
        };
    }

    helper!(Singular, singular, 0);
    helper!(Array, array, 1);
    helper!(BiMap, bi_map, 1);
}
