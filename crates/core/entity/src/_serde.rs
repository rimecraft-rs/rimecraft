use std::{borrow::Cow, marker::PhantomData};

use glam::{DVec3, Vec2};
use serde::{
    Deserialize, Serialize,
    ser::{SerializeMap, SerializeSeq},
};
use serde_private::de::ContentVisitor;
use serde_update::Update as _;
use uuid::Uuid;

use crate::{EntityCell, EntityCx, ErasedData, RawEntity};

const KEY_POS: &str = "Pos";
const KEY_VELOCITY: &str = "Motion";
const KEY_ROTATION: &str = "Rotation";
const KEY_UUID: &str = "UUID";
const KEY_CUSTOM_COMPONENT: &str = "data";
const KEY_PASSENGERS: &str = "Passengers";

#[derive(Debug)]
enum Field<'a> {
    Pos,
    Velocity,
    Rotation,
    Uuid,
    CustomComponent,
    Passengers,
    Other(Cow<'a, str>),
}

impl Serialize for Field<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let literal = match self {
            Field::Pos => KEY_POS,
            Field::Velocity => KEY_VELOCITY,
            Field::Rotation => KEY_ROTATION,
            Field::Uuid => KEY_UUID,
            Field::CustomComponent => KEY_CUSTOM_COMPONENT,
            Field::Passengers => KEY_PASSENGERS,
            Field::Other(name) => name,
        };
        serializer.serialize_str(literal)
    }
}

impl<'de> Deserialize<'de> for Field<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn match_name(name: &str) -> Option<Field<'static>> {
            match name {
                KEY_POS => Some(Field::Pos),
                KEY_VELOCITY => Some(Field::Velocity),
                KEY_ROTATION => Some(Field::Rotation),
                KEY_UUID => Some(Field::Uuid),
                KEY_CUSTOM_COMPONENT => Some(Field::CustomComponent),
                KEY_PASSENGERS => Some(Field::Passengers),
                _ => None,
            }
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Field<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a field name")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match_name(v).unwrap_or(Field::Other(Cow::Borrowed(v))))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match_name(v).unwrap_or(Field::Other(Cow::Owned(v.to_owned()))))
            }
        }

        deserializer.deserialize_identifier(Visitor)
    }
}

struct SerWrapper<'borrow, T: ?Sized> {
    id: Option<&'borrow str>,
    inner: &'borrow T,
}

fn do_ser_as_passenger<'a, T: ?Sized, Cx>(this: &RawEntity<'a, T, Cx>) -> bool
where
    Cx: EntityCx<'a>,
{
    this.removal.is_none_or(|r| r.should_save) && this.ty.erased_is_saveable()
}

struct SerPassengers<'borrow, T: ?Sized>(&'borrow T);

impl<'a, T: ?Sized, Cx> Serialize for RawEntity<'a, T, Cx>
where
    T: Serialize,
    Cx: EntityCx<'a, Compound: Serialize>,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerWrapper {
            id: None,
            inner: self,
        }
        .serialize(serializer)
    }
}

impl<'a, T: ?Sized, Cx> Serialize for SerWrapper<'_, RawEntity<'a, T, Cx>>
where
    T: Serialize,
    Cx: EntityCx<'a, Compound: Serialize>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let this = self.inner;
        let mut map = serializer.serialize_map(None)?;

        if let Some(id) = self.id {
            map.serialize_entry("id", id)?;
        }
        // glam and vanilla use same serde pattern for vectors
        map.serialize_entry(
            KEY_POS,
            &this.vehicle.as_ref().map_or(this.pos, |v| v.lock().pos),
        )?;
        map.serialize_entry(KEY_VELOCITY, &this.velocity)?;
        map.serialize_entry(KEY_ROTATION, &Vec2::new(this.yaw, this.pitch))?;
        map.serialize_entry(KEY_UUID, &serde_compat::IntStreamCodec(this.uuid))?;
        map.serialize_entry(KEY_CUSTOM_COMPONENT, &this.custom_compound)?;
        this.data
            .serialize(serde_private::ser::FlatMapSerializer(&mut map))?;
        if this.has_passengers() {
            map.serialize_entry(KEY_PASSENGERS, &SerPassengers(&*this.passengers))?;
        }

        map.end()
    }
}

impl<'a, Cx> Serialize for SerPassengers<'_, [EntityCell<'a, Cx>]>
where
    Cx: EntityCx<'a, Compound: Serialize>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for p in self.0 {
            let g = p.lock();
            if do_ser_as_passenger(&g) {
                let id = registry::Reg::to_id(g.ty).to_string();
                seq.serialize_element(&SerWrapper {
                    id: Some(&id),
                    inner: &**g,
                })?;
            }
        }
        seq.end()
    }
}

impl<'a, 'de, T: ?Sized, Cx> serde_update::Update<'de> for RawEntity<'a, T, Cx>
where
    Cx: EntityCx<'a, Compound: Deserialize<'de>>,
    T: serde_update::Update<'de> + ErasedData<'a, Cx>,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'borrow, T: ?Sized>(&'borrow mut T);

        impl<'a, T: ?Sized, Cx> Visitor<'_, RawEntity<'a, T, Cx>>
        where
            Cx: EntityCx<'a>,
        {
            const POS_UPPER_BOUND: DVec3 =
                DVec3::new(Cx::POS_XZ_BOUND, Cx::POS_Y_BOUND, Cx::POS_XZ_BOUND);

            const VELOCITY_UPPER_BOUND: DVec3 =
                DVec3::new(Cx::VELOCITY_BOUND, Cx::VELOCITY_BOUND, Cx::VELOCITY_BOUND);
        }

        impl<'a, 'de, T: ?Sized, Cx> serde::de::Visitor<'de> for Visitor<'_, RawEntity<'a, T, Cx>>
        where
            Cx: EntityCx<'a, Compound: Deserialize<'de>>,
            T: serde_update::Update<'de> + ErasedData<'a, Cx>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "an entity")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde_private::de::Content;
                let mut collect: Vec<Option<(Content<'de>, Content<'de>)>> =
                    Vec::with_capacity(map.size_hint().map_or(0, |i| i - 1));
                let this = self.0;

                while let Some(field) = map.next_key::<Field<'de>>()? {
                    match field {
                        Field::Pos => {
                            let pos = map.next_value::<DVec3>()?;
                            this.set_pos(pos.clamp(-Self::POS_UPPER_BOUND, Self::POS_UPPER_BOUND));
                            this.update_last_pos();
                        }
                        Field::Velocity => {
                            let velocity = map.next_value::<DVec3>()?;

                            this.set_velocity(DVec3::select(
                                velocity.cmpgt(Self::VELOCITY_UPPER_BOUND)
                                    | velocity.cmplt(-Self::VELOCITY_UPPER_BOUND),
                                DVec3::ZERO,
                                velocity,
                            ));
                        }
                        Field::Rotation => {
                            let rot = map.next_value::<Vec2>()?;
                            this.set_yaw(rot.x);
                            this.set_pitch(rot.y);
                            this.update_last_rotation();
                            this.data.erased_set_yaws(this.yaw);
                        }
                        Field::Uuid => {
                            this.uuid = map.next_value::<serde_compat::IntStreamCodec<Uuid>>()?.0
                        }
                        Field::CustomComponent => this.custom_compound = map.next_value()?,
                        Field::Passengers => {}
                        Field::Other(cow) => collect.push(Some((
                            match cow {
                                Cow::Borrowed(a) => Content::Str(a),
                                Cow::Owned(a) => Content::String(a),
                            },
                            map.next_value_seed(ContentVisitor::new())?,
                        ))),
                    }
                }

                this.ext.update(serde_private::de::FlatMapDeserializer(
                    &mut collect,
                    PhantomData,
                ))?;

                this.data.update(serde_private::de::FlatMapDeserializer(
                    &mut collect,
                    PhantomData,
                ))?;

                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}
