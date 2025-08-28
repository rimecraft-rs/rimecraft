use std::borrow::Cow;

use glam::Vec2;
use serde::{
    Deserialize, Serialize,
    ser::{SerializeMap, SerializeSeq},
};

use crate::{EntityCell, EntityCx, RawEntity};

const KEY_POS: &str = "Pos";
const KEY_MOTION: &str = "Motion";
const KEY_ROTATION: &str = "Rotation";
const KEY_UUID: &str = "UUID";
const KEY_CUSTOM_COMPONENT: &str = "data";
const KEY_PASSENGERS: &str = "Passengers";

#[derive(Debug)]
enum Field<'a> {
    Pos,
    Motion,
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
            Field::Motion => KEY_MOTION,
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
                KEY_MOTION => Some(Field::Motion),
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
        map.serialize_entry(KEY_MOTION, &this.velocity)?;
        map.serialize_entry(KEY_ROTATION, &Vec2::new(this.yaw, this.pitch))?;
        map.serialize_entry(KEY_UUID, &serde_compat::IntStreamCodec(this.uuid))?;
        map.serialize_entry(KEY_CUSTOM_COMPONENT, &this.custom_compound)?;
        this.data
            .serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        if let Some(ref passengers) = this.passengers {
            map.serialize_entry(KEY_PASSENGERS, &SerPassengers(&**passengers))?;
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
