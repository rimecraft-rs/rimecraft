use super::{tag::TagKey, Registry, RegistryKey};
use crate::util::Identifier;
use datafixerupper::datafixers::util::Either;
use std::fmt::Display;

pub enum RegistryEntry<T, R: Registry<T>> {
    Direct(T),
    Reference(ReferenceEntry<T, R>),
}

impl<T, R: Registry<T>> RegistryEntry<T, R> {
    pub fn as_ref_entry(&self) -> Option<&ReferenceEntry<T, R>> {
        match self {
            RegistryEntry::Direct(_) => None,
            RegistryEntry::Reference(r) => Some(r),
        }
    }

    pub fn as_ref_entry_mut(&mut self) -> Option<&mut ReferenceEntry<T, R>> {
        match self {
            RegistryEntry::Direct(_) => None,
            RegistryEntry::Reference(r) => Some(r),
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            RegistryEntry::Direct(value) => Some(value),
            RegistryEntry::Reference(r) => match &r.value {
                Some(a) => Some(a),
                None => None,
            },
        }
    }

    pub fn has_key_and_value(&self) -> bool {
        match self {
            RegistryEntry::Direct(_) => true,
            RegistryEntry::Reference(r) => r.registry_key.is_some() && r.value.is_some(),
        }
    }

    pub fn matches_id(&self, id: &Identifier) -> bool {
        match self {
            RegistryEntry::Direct(_) => false,
            RegistryEntry::Reference(r) => match &r.registry_key {
                Some(k) => k.get_value().eq(id),
                None => false,
            },
        }
    }

    pub fn matches_key(&self, key: &RegistryKey<T>) -> bool {
        match self {
            RegistryEntry::Direct(_) => false,
            RegistryEntry::Reference(r) => match &r.registry_key {
                Some(k) => k == key,
                None => false,
            },
        }
    }

    pub fn matches(&self, predicate: impl Fn(&RegistryKey<T>) -> bool) -> bool {
        match self {
            RegistryEntry::Direct(_) => false,
            RegistryEntry::Reference(r) => match &r.registry_key {
                Some(k) => predicate(k),
                None => false,
            },
        }
    }

    pub fn is_in(&self, tag: &TagKey<T, R>) -> bool {
        match self {
            RegistryEntry::Direct(_) => false,
            RegistryEntry::Reference(r) => r.tags.contains(tag),
        }
    }

    pub fn get_tags(&self) -> Vec<&TagKey<T, R>> {
        match self {
            RegistryEntry::Direct(_) => Vec::new(),
            RegistryEntry::Reference(r) => r.tags.iter().collect(),
        }
    }

    pub fn get_key_or_value(&self) -> Option<Either<&RegistryKey<T>, &T>> {
        match self {
            RegistryEntry::Direct(v) => Some(Either::Right(v)),
            RegistryEntry::Reference(r) => match &r.registry_key {
                Some(key) => Some(Either::Left(key)),
                None => None,
            },
        }
    }

    pub fn get_key(&self) -> Option<&RegistryKey<T>> {
        match self {
            RegistryEntry::Direct(_) => None,
            RegistryEntry::Reference(r) => match &r.registry_key {
                Some(key) => Some(key),
                None => None,
            },
        }
    }
}

impl<T: Display, R: Registry<T>> Display for RegistryEntry<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryEntry::Direct(value) => {
                f.write_str("Direct{")?;
                value.fmt(f)?;
                f.write_str("}")?;
            }
            RegistryEntry::Reference(r) => {
                f.write_str("Reference{")?;
                match &r.registry_key {
                    Some(s) => s.fmt(f),
                    None => f.write_str("nil"),
                }?;
                f.write_str("=")?;
                match &r.value {
                    Some(s) => s.fmt(f),
                    None => f.write_str("nil"),
                }?;
                f.write_str("}")?;
            }
        }

        std::fmt::Result::Ok(())
    }
}

pub struct ReferenceEntry<T, R: Registry<T>> {
    pub value: Option<T>,
    pub registry_key: Option<RegistryKey<T>>,
    pub reference_type: ReferenceType,
    pub tags: Vec<TagKey<T, R>>,
}

impl<T, R: Registry<T>> ReferenceEntry<T, R> {
    fn new(
        reference_type: ReferenceType,
        registry_key: Option<RegistryKey<T>>,
        value: Option<T>,
    ) -> Self {
        Self {
            value,
            registry_key,
            reference_type,
            tags: Vec::new(),
        }
    }

    pub fn stand_alone(value: Option<T>) -> Self {
        Self::new(ReferenceType::StandAlone, None, value)
    }
}

pub enum ReferenceType {
    StandAlone,
    Intrusive,
}

pub trait RegistryEntryOwner<T> {}
