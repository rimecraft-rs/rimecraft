use super::{tag::TagKey, RegistryKey};
use crate::util::Identifier;
use datafixerupper::datafixers::util::Either;
use std::fmt::Display;

pub enum RegistryEntry<T> {
    Direct(T),
    Reference(ReferenceEntry<T>),
}

impl<T> RegistryEntry<T> {
    pub fn as_ref_entry(&self) -> Option<&ReferenceEntry<T>> {
        match self {
            RegistryEntry::Direct(_) => None,
            RegistryEntry::Reference(r) => Some(r),
        }
    }

    pub fn as_ref_entry_mut(&mut self) -> Option<&mut ReferenceEntry<T>> {
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

    pub fn value_mut(&mut self) -> Option<&mut T> {
        match self {
            RegistryEntry::Direct(value) => Some(value),
            RegistryEntry::Reference(r) => match &mut r.value {
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

    pub fn is_in(&self, tag: &TagKey<T>) -> bool {
        match self {
            RegistryEntry::Direct(_) => false,
            RegistryEntry::Reference(r) => r.tags.contains(tag),
        }
    }

    pub fn get_tags(&self) -> Vec<&TagKey<T>> {
        match self {
            RegistryEntry::Direct(_) => Vec::new(),
            RegistryEntry::Reference(r) => r.tags.iter().collect(),
        }
    }

    pub fn get_key_or_value(&self) -> Option<Either<&RegistryKey<T>, &T>> {
        match self {
            RegistryEntry::Direct(v) => Some(Either::Right(v)),
            RegistryEntry::Reference(r) => r.registry_key.as_ref().map(Either::Left),
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

impl<T: Display> Display for RegistryEntry<T> {
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

pub struct ReferenceEntry<T> {
    pub value: Option<T>,
    pub registry_key: Option<RegistryKey<T>>,
    pub reference_type: ReferenceType,
    pub tags: Vec<TagKey<T>>,
}

impl<T> ReferenceEntry<T> {
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

    pub fn intrusive(value: Option<T>) -> Self {
        Self::new(ReferenceType::Intrusive, None, value)
    }

    pub fn set_tags(&mut self, tags: Vec<TagKey<T>>) {
        self.tags = tags
    }

    pub fn set_registry_key(&mut self, registry_key: RegistryKey<T>) {
        if !(self.registry_key.is_some() && self.registry_key.as_ref().unwrap() != &registry_key) {
            self.registry_key = Some(registry_key)
        }
    }

    pub fn get_registry_key(&self) -> Option<&RegistryKey<T>> {
        match &self.registry_key {
            Some(s) => Some(s),
            None => None,
        }
    }
}

impl<T: PartialEq> ReferenceEntry<T> {
    pub fn set_value(&mut self, value: T) {
        if !(self.reference_type == ReferenceType::Intrusive
            && self.value.is_some()
            && self.value.as_ref().unwrap() == &value)
        {
            self.value = Some(value)
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ReferenceType {
    StandAlone,
    Intrusive,
}
