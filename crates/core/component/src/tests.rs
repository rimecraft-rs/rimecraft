use std::sync::{Arc, LazyLock};

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use fastnbt::DeOpts;
use local_cx::{
    BaseLocalContext, LocalContext, LocalContextExt,
    dyn_codecs::Any,
    dyn_cx::{AsDynamicContext, ContextTable, DynamicContext},
    edcode_codec,
    serde::DeserializeWithCx,
    serde_codec,
};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{Registry, RegistryKey, RegistryMut};
use serde::{Deserialize, Serialize};

use crate::{
    ComponentType, PacketCodec, RawErasedComponentType, SerdeCodec, changes::ComponentChanges,
    map::ComponentMap,
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

const fn packet_codec_edcode<'a>() -> PacketCodec<'a, Foo> {
    edcode_codec!(Foo: Any + 'a)
}

const fn packet_codec_nbt<'a>() -> PacketCodec<'a, Foo> {
    edcode_codec!(Nbt<Context> Foo: Any + 'a)
}

const fn serde_codec<'a>() -> SerdeCodec<'a, Foo> {
    serde_codec!(Foo: Any + 'a)
}

#[test]
#[should_panic]
fn type_builder_no_edcode() {
    let _ty = ComponentType::<'static, Foo>::builder::<Context>().build();
}

#[test]
fn type_transient_check() {
    let ty = ComponentType::<'_, Foo>::builder::<Context>()
        .packet_codec(packet_codec_edcode())
        .build();
    assert!(ty.is_transient());

    let ty = ComponentType::<'_, Foo>::builder::<Context>()
        .packet_codec(packet_codec_edcode())
        .serde_codec(serde_codec())
        .build();
    assert!(!ty.is_transient());
}

const REGISTRY_ID: Id =
    unsafe { test_global::integration::registry::id_unchecked("data_component_types") };

const fn type_transient_edcode<'a>() -> ComponentType<'a, Foo> {
    ComponentType::<'_, Foo>::builder::<Context>()
        .packet_codec(packet_codec_edcode())
        .build()
}
const fn type_transient_edcode_key<'a>() -> RegistryKey<Id, RawErasedComponentType<'a, Context>> {
    registry_key("foo_transient_edcode")
}

const fn type_persistent<'a>() -> ComponentType<'a, Foo> {
    ComponentType::<'_, Foo>::builder::<Context>()
        .packet_codec(packet_codec_nbt())
        .serde_codec(serde_codec())
        .build()
}
const fn type_persistent_key<'a>() -> RegistryKey<Id, RawErasedComponentType<'a, Context>> {
    registry_key("foo_persistent")
}

const fn registry_key<'a>(
    name: &'static str,
) -> RegistryKey<Id, RawErasedComponentType<'a, Context>> {
    RegistryKey::new(REGISTRY_ID, unsafe { Id::const_new("test", name) })
}

fn init_registry<'a>() -> Registry<Id, RawErasedComponentType<'a, Context>> {
    let mut registry = RegistryMut::new(RegistryKey::with_root(REGISTRY_ID));
    registry
        .register(
            type_transient_edcode_key(),
            (&type_transient_edcode()).into(),
        )
        .expect("register failed");
    registry
        .register(type_persistent_key(), (&type_persistent()).into())
        .expect("register failed");

    registry.into()
}

struct LocalCx<'a> {
    component_ty_registry: Registry<Id, RawErasedComponentType<'a, Context>>,
}

impl BaseLocalContext for &LocalCx<'_> {}

impl<'a, 'c> LocalContext<&'a Registry<Id, RawErasedComponentType<'c, Context>>>
    for &'a LocalCx<'c>
{
    fn acquire(self) -> &'a Registry<Id, RawErasedComponentType<'c, Context>> {
        &self.component_ty_registry
    }
}

impl AsDynamicContext for &LocalCx<'_> {
    type InnerContext = Self;

    #[allow(clippy::unnecessary_cast)]
    fn as_dynamic_context(&self) -> DynamicContext<'_, Self::InnerContext> {
        static TABLE: LazyLock<ContextTable<&'static LocalCx<'static>>> = LazyLock::new(|| {
            let mut table = ContextTable::new();
            table.enable::<&Registry<Id, RawErasedComponentType<'static, Context>>>();
            table
        });

        DynamicContext::from_borrowed_table(*self, unsafe {
            &*(std::ptr::from_ref(&*TABLE) as *const ContextTable<&'_ LocalCx<'_>>)
        })
    }
}

fn init_context<'a>() -> LocalCx<'a> {
    let component_ty_registry = init_registry();
    LocalCx {
        component_ty_registry,
    }
}

#[test]
fn built_map() {
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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
        unsafe { map.get(&type_transient_edcode()) }
            .expect("missing edcode_ty")
            .value,
        514,
        "edcode_ty value mismatch"
    );
    assert_eq!(
        unsafe { map.get(&type_persistent()) }
            .expect("missing persistent_ty")
            .value,
        1919,
        "persistent_ty value mismatch"
    );

    unsafe { map.get_mut(&type_transient_edcode()) }
        .expect("missing edcode_ty")
        .value = 114;
    assert_eq!(
        unsafe { map.get(&type_transient_edcode()) }
            .expect("missing edcode_ty")
            .value,
        114,
        "edcode_ty value mismatch after modification"
    );
}

#[test]
fn iter_map() {
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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
        let obj: &(dyn Any) = obj;
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
            .remove(&type_persistent())
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
            let obj: &(dyn Any) = obj;
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
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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
            .remove(&type_transient_edcode())
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
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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

    let buf = fastnbt::to_bytes(&cx.with(&map)).expect("serialize failed");
    let new_map = ComponentMap::deserialize_with_cx(cx.with(
        &mut fastnbt::de::Deserializer::from_bytes(&buf, DeOpts::new()),
    ))
    .expect("deserialize failed");
    assert_eq!(new_map.len(), 1, "map length not intended");
    let obj = unsafe { new_map.get(&type_persistent()) }.expect("missing persistent_ty");
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
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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
                .remove(&type_transient_edcode())
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

        let buf = fastnbt::to_bytes(&cx.with(&changes)).expect("serialize failed");
        let new_changes = ComponentChanges::deserialize_with_cx(cx.with(
            &mut fastnbt::de::Deserializer::from_bytes(&buf, DeOpts::new()),
        ))
        .expect("deserialize failed");
        assert_eq!(new_changes.len(), 1, "changes length not intended");
        assert_eq!(
            unsafe { new_changes.get(&type_persistent()) }
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
                .remove(&type_persistent())
                .expect("remove persistent component failed");
        }

        let changes = patched.changes().expect("no changes");

        let buf = fastnbt::to_bytes(&cx.with(&changes)).expect("serialize failed");
        let new_changes = ComponentChanges::deserialize_with_cx(cx.with(
            &mut fastnbt::de::Deserializer::from_bytes(&buf, DeOpts::new()),
        ))
        .expect("deserialize failed");
        assert_eq!(new_changes.len(), 1, "changes length not intended");
        assert!(
            unsafe { new_changes.get(&type_persistent()) }
                .expect("missing persistent_ty")
                .is_none(),
            "persistent_ty should be removed"
        );
    }
}

#[test]
fn changes_edcode() {
    let cx = init_context();
    let reg = &cx.component_ty_registry;

    let edcode_ty = reg
        .get(&type_transient_edcode_key())
        .expect("invalid registry");
    let persistent_ty = reg.get(&type_persistent_key()).expect("invalid registry");

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
            .remove(&type_transient_edcode())
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
    changes.encode(cx.with(&mut buf)).expect("serialize failed");
    let new_changes =
        ComponentChanges::<'_, '_, Context>::decode(cx.with(&buf[..])).expect("deserialize failed");
    assert_eq!(new_changes.len(), 2, "changes length not intended");
    assert!(
        unsafe { new_changes.get(&type_transient_edcode()) }
            .expect("missing edcode_ty")
            .is_none(),
        "edcode_ty is removed"
    );
    assert_eq!(
        unsafe { new_changes.get(&type_persistent()) }
            .expect("missing persistent_ty")
            .expect("persistent_ty is not removed")
            .value,
        1919
    );
}
