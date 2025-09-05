use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{IntStreamCodec, StringCodec, VanillaCodec};

fn from_ints(ints: [u32; 4]) -> Uuid {
    Uuid::from_u64_pair(
        ((ints[0] as u64) << u32::BITS) | ints[1] as u64,
        ((ints[2] as u64) << u32::BITS) | ints[3] as u64,
    )
}

impl Serialize for IntStreamCodec<&Uuid> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (m, l) = self.0.as_u64_pair();
        [
            (m >> u32::BITS) as u32,
            m as u32,
            (l >> u32::BITS) as u32,
            l as u32,
        ]
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IntStreamCodec<Uuid> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <[u32; 4]>::deserialize(deserializer)
            .map(from_ints)
            .map(Self)
    }
}

impl Serialize for IntStreamCodec<Uuid> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        IntStreamCodec(&self.0).serialize(serializer)
    }
}

impl Serialize for StringCodec<&Uuid> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Uuid::encode_buffer();
        self.0.as_hyphenated().encode_lower(&mut buf);
        serializer.serialize_str(str::from_utf8(&buf).map_err(serde::ser::Error::custom)?)
    }
}

impl<'de> Deserialize<'de> for StringCodec<Uuid> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s)
            .map(StringCodec)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for StringCodec<Uuid> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        StringCodec(&self.0).serialize(serializer)
    }
}

impl Serialize for VanillaCodec<&Uuid> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = Uuid::encode_buffer();
            self.0.as_simple().encode_lower(&mut buf);
            serializer.serialize_str(str::from_utf8(&buf).map_err(serde::ser::Error::custom)?)
        } else {
            IntStreamCodec(self.0).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for VanillaCodec<Uuid> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "an uuid literal or a corresponding int stream")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Uuid::parse_str(v).map_err(E::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut ints = [0u32; 4];
                for (len, i) in ints.iter_mut().enumerate() {
                    *i = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(len, &"4"))?;
                }
                Ok(from_ints(ints))
            }
        }

        deserializer.deserialize_any(Visitor).map(Self)
    }
}

impl Serialize for VanillaCodec<Uuid> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        VanillaCodec(&self.0).serialize(serializer)
    }
}
