use std::marker::PhantomData;

use crate::{IndexFromRaw, IndexToRaw, Palette, Strategy};
use rimecraft_maybe::Maybe;

#[test]
fn test_singular() {
    struct ListImpl;
    // Just blanket impl.
    impl IndexToRaw<&u8> for ListImpl {
        fn raw_id(&self, _entry: &u8) -> Option<usize> {
            None
        }
    }
    impl<'a, 'b> IndexFromRaw<'a, Maybe<'b, u8>> for ListImpl {
        fn of_raw(&'a self, _id: usize) -> Option<Maybe<'b, u8>> {
            None
        }
    }
    struct IterImpl<'a>(PhantomData<&'a ()>);
    impl<'a> Iterator for IterImpl<'a> {
        type Item = &'a u8;

        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }
    impl ExactSizeIterator for IterImpl<'_> {
        fn len(&self) -> usize {
            0
        }
    }
    impl<'a> IntoIterator for &'a ListImpl {
        type Item = &'a u8;
        type IntoIter = IterImpl<'a>;

        fn into_iter(self) -> Self::IntoIter {
            IterImpl(PhantomData)
        }
    }

    let mut palette = Palette::new(Strategy::Singular, 0, ListImpl, Vec::<u8>::new());
    assert_eq!(palette.len(), 0);
    assert_eq!(palette.index_or_insert(36), Ok(0));
    assert!(matches!(palette.get(0), Some(Maybe::Borrowed(&36))));
    let mut iter = palette.iter();
    assert!(matches!(iter.next(), Some(36)));
    assert!(matches!(iter.next(), None));
    assert_eq!(palette.len(), 1);
    assert_eq!(palette.config(), (Strategy::Singular, 0));
}
