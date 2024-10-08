use std::sync::Arc;

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::RegistryKey;
use serde::{Deserialize, Serialize};

use crate::{
    changes::ComponentChanges, map::ComponentMap, ComponentType, PacketCodec,
    RawErasedComponentType, SerdeCodec,
};

use test_global::TestContext as Context;

type Id = <Context as ProvideIdTy>::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Foo {
    value: i32,
    info: String,
}

impl<B> Encode<B> for Foo
where
    B: BufMut,
{
    fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
        self.value.encode(&mut buf)?;
        self.info.encode(&mut buf)?;
        Ok(())
    }
}

impl<'de, B> Decode<'de, B> for Foo
where
    B: Buf,
{
    fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        let value = i32::decode(&mut buf)?;
        let info = String::decode(&mut buf)?;
        Ok(Self { value, info })
    }
}

const PACKET_CODEC_EDCODE: PacketCodec<'static, Foo> = crate::packet_codec_edcode();
const PACKET_CODEC_NBT: PacketCodec<'static, Foo> = crate::packet_codec_nbt::<'_, _, Context>();
const SERDE_CODEC: SerdeCodec<'static, Foo> = crate::serde_codec();

#[test]
#[should_panic]
fn type_builder_no_edcode() {
    let _ty = ComponentType::<'static, Foo>::builder::<Context>().build();
}

#[test]
fn type_transient_check() {
    let ty = ComponentType::<'static, Foo>::builder::<Context>()
        .packet_codec(&PACKET_CODEC_EDCODE)
        .build();
    assert!(ty.is_transient());

    let ty = ComponentType::<'static, Foo>::builder::<Context>()
        .packet_codec(&PACKET_CODEC_EDCODE)
        .serde_codec(&SERDE_CODEC)
        .build();
    assert!(!ty.is_transient());
}

const TYPE_TRANSIENT_EDCODE: ComponentType<'static, Foo> =
    ComponentType::<'static, Foo>::builder::<Context>()
        .packet_codec(&PACKET_CODEC_EDCODE)
        .build();
const TYPE_TRANSIENT_EDCODE_KEY: RegistryKey<Id, RawErasedComponentType<'static, Context>> =
    registry_key("foo_transient_edcode");

const TYPE_PERSISTENT: ComponentType<'static, Foo> =
    ComponentType::<'static, Foo>::builder::<Context>()
        .packet_codec(&PACKET_CODEC_NBT)
        .serde_codec(&SERDE_CODEC)
        .build();
const TYPE_PERSISTENT_KEY: RegistryKey<Id, RawErasedComponentType<'static, Context>> =
    registry_key("foo_persistent");

const fn registry_key(
    name: &'static str,
) -> RegistryKey<Id, RawErasedComponentType<'static, Context>> {
    RegistryKey::new(crate::test_global_integration::REGISTRY_ID, unsafe {
        Id::const_new("test", name)
    })
}

fn init_registry() {
    crate::test_global_integration::peek_registry_mut(|registry| {
        registry
            .register(TYPE_TRANSIENT_EDCODE_KEY, (&TYPE_TRANSIENT_EDCODE).into())
            .expect("register failed");
        registry
            .register(TYPE_PERSISTENT_KEY, (&TYPE_PERSISTENT).into())
            .expect("register failed");
    });
    crate::test_global_integration::init_registry();
}

#[test]
fn built_map() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        edcode_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 1919,
            info: "wlg".to_owned(),
        },
    );
    let mut map = builder.build();

    assert_eq!(map.len(), 2);
    assert!(map.changes().is_none());

    assert_eq!(
        unsafe { map.get(&TYPE_TRANSIENT_EDCODE) }
            .expect("missing edcode_ty")
            .value,
        514,
        "edcode_ty value mismatch"
    );
    assert_eq!(
        unsafe { map.get(&TYPE_PERSISTENT) }
            .expect("missing persistent_ty")
            .value,
        1919,
        "persistent_ty value mismatch"
    );

    unsafe { map.get_mut(&TYPE_TRANSIENT_EDCODE) }
        .expect("missing edcode_ty")
        .value = 114;
    assert_eq!(
        unsafe { map.get(&TYPE_TRANSIENT_EDCODE) }
            .expect("missing edcode_ty")
            .value,
        114,
        "edcode_ty value mismatch after modification"
    );
}

#[test]
fn iter_map() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    let map = builder.build();

    let mut count = 0;
    let iter = map.iter();
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.len(), 2);
    for (ty, obj) in iter {
        let obj = unsafe { obj.downcast_ref::<Foo>() }.expect("downcast failed");
        if ty == edcode_ty {
            assert_eq!(obj.value, 114);
        } else if ty == persistent_ty {
            assert_eq!(obj.value, 514);
        } else {
            panic!("unexpected type for simple map");
        }
        count += 1;
    }
    assert_eq!(count, 2);

    let arc = Arc::new(map);
    let mut patched = ComponentMap::arc_new(arc);
    unsafe {
        patched
            .remove(&TYPE_PERSISTENT)
            .expect("remove persistent component failed");
        patched.insert(
            persistent_ty,
            Foo {
                value: 1919,
                info: "wlg".to_owned(),
            },
        );

        let mut count = 0;
        let iter = patched.iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.len(), 2);
        for (ty, obj) in iter {
            let obj = obj.downcast_ref::<Foo>().expect("downcast failed");
            if ty == edcode_ty {
                assert_eq!(obj.value, 114);
            } else if ty == persistent_ty {
                assert_eq!(obj.value, 1919);
            } else {
                panic!("unexpected type for patched map");
            }
            count += 1;
        }
        assert_eq!(count, 2);
    }
}

#[test]
fn patched_changes() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    let map = Arc::new(builder.build());

    let mut patched = ComponentMap::arc_new(map.clone());
    unsafe {
        patched
            .remove(&TYPE_TRANSIENT_EDCODE)
            .expect("remove transient component failed");
        patched.insert(
            persistent_ty,
            Foo {
                value: 1919,
                info: "wlg".to_owned(),
            },
        );
        assert_eq!(patched.len(), 1);
    }

    let changes = patched.changes().expect("no changes");
    assert_eq!(changes.len(), 2);
    let new_patched = ComponentMap::arc_with_changes(map.clone(), changes);
    assert_eq!(new_patched.len(), 1);
}

#[test]
fn map_serde() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    let map = builder.build();

    let buf = fastnbt::to_bytes(&map).expect("serialize failed");
    let new_map =
        fastnbt::from_bytes::<ComponentMap<'_, Context>>(&buf).expect("deserialize failed");
    assert_eq!(new_map.len(), 1, "map length not intended");
    let obj = unsafe { new_map.get(&TYPE_PERSISTENT) }.expect("missing persistent_ty");
    assert_eq!(
        obj,
        &Foo {
            value: 514,
            info: "world".to_owned()
        }
    );
}

#[test]
fn changes_serde() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    let map = Arc::new(builder.build());

    // Additions and modifications
    {
        let mut patched = ComponentMap::arc_new(map.clone());
        unsafe {
            patched
                .remove(&TYPE_TRANSIENT_EDCODE)
                .expect("remove transient component failed");
            patched.insert(
                persistent_ty,
                Foo {
                    value: 1919,
                    info: "wlg".to_owned(),
                },
            );
        }

        let changes = patched.changes().expect("no changes");

        let buf = fastnbt::to_bytes(&changes).expect("serialize failed");
        let new_changes = fastnbt::from_bytes::<ComponentChanges<'_, 'static, Context>>(&buf)
            .expect("deserialize failed");
        assert_eq!(new_changes.len(), 1, "changes length not intended");
        assert_eq!(
            unsafe { new_changes.get(&TYPE_PERSISTENT) }
                .expect("missing persistent_ty")
                .expect("persistent_ty is not removed")
                .value,
            1919
        );
    }

    // Removals
    {
        let mut patched = ComponentMap::arc_new(map.clone());
        unsafe {
            patched
                .remove(&TYPE_PERSISTENT)
                .expect("remove persistent component failed");
        }

        let changes = patched.changes().expect("no changes");

        let buf = fastnbt::to_bytes(&changes).expect("serialize failed");
        let new_changes = fastnbt::from_bytes::<ComponentChanges<'_, 'static, Context>>(&buf)
            .expect("deserialize failed");
        assert_eq!(new_changes.len(), 1, "changes length not intended");
        assert!(
            unsafe { new_changes.get(&TYPE_PERSISTENT) }
                .expect("missing persistent_ty")
                .is_none(),
            "persistent_ty should be removed"
        );
    }
}

#[test]
fn changes_edcode() {
    init_registry();
    let reg = crate::test_global_integration::registry();
    let edcode_ty = reg
        .get(&TYPE_TRANSIENT_EDCODE_KEY)
        .expect("invalid registry");
    let persistent_ty = reg.get(&TYPE_PERSISTENT_KEY).expect("invalid registry");

    let mut builder = ComponentMap::builder();
    builder.insert(
        edcode_ty,
        Foo {
            value: 114,
            info: "hello".to_owned(),
        },
    );
    builder.insert(
        persistent_ty,
        Foo {
            value: 514,
            info: "world".to_owned(),
        },
    );
    let map = Arc::new(builder.build());

    let mut patched = ComponentMap::arc_new(map.clone());
    unsafe {
        patched
            .remove(&TYPE_TRANSIENT_EDCODE)
            .expect("remove transient component failed");
        patched.insert(
            persistent_ty,
            Foo {
                value: 1919,
                info: "wlg".to_owned(),
            },
        );
    }

    let changes = patched.changes().expect("no changes");

    let mut buf = Vec::new();
    changes.encode(&mut buf).expect("serialize failed");
    let new_changes =
        ComponentChanges::<'_, 'static, Context>::decode(&buf[..]).expect("deserialize failed");
    assert_eq!(new_changes.len(), 2, "changes length not intended");
    assert!(
        unsafe { new_changes.get(&TYPE_TRANSIENT_EDCODE) }
            .expect("missing edcode_ty")
            .is_none(),
        "edcode_ty is removed"
    );
    assert_eq!(
        unsafe { new_changes.get(&TYPE_PERSISTENT) }
            .expect("missing persistent_ty")
            .expect("persistent_ty is not removed")
            .value,
        1919
    );
}
