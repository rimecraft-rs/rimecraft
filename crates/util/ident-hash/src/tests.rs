use std::{
    collections::HashSet,
    hash::{Hash, Hasher as _},
};

use crate::{IdentityBuildHasher, IdentityHasher};

#[test]
fn signed_hash() {
    fn hash<T: Hash>(val: T) -> u64 {
        let mut hasher = IdentityHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    macro_rules! traverse {
        ($($t:ident),*$(,)?) => {
            {$(assert_ne!(hash(0 - $t::MAX / 2), hash($t::MAX / 2));)*}
        };
    }
    traverse![i8, i16, i32, i64, isize]
}

#[test]
#[allow(trivial_numeric_casts)]
fn unsigned_hash_set() {
    macro_rules! traverse {
        ($($t:ty),*$(,)?) => {
            {$(
                let mut set: HashSet<$t, _> = HashSet::with_hasher(IdentityBuildHasher::new());
                set.insert(u8::MAX as $t / 2);
                assert!(set.contains(&(u8::MAX as $t / 2)));
            )*}
        };
    }
    traverse![u8, u16, u32, u64, usize]
}
