pub mod collections;
pub mod math;

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub fn new(namespace: &str, path: &str) -> anyhow::Result<Self> {
        if Self::is_namespace_valid(namespace) && Self::is_path_valid(path) {
            Ok(Self {
                namespace: namespace.to_string(),
                path: path.to_string(),
            })
        } else {
            Err(anyhow::anyhow!(
                "Non [a-z0-9/._-] character in identifier: {namespace}:{path}"
            ))
        }
    }

    pub fn parse(id: &str) -> Self {
        Self::try_parse(id).unwrap()
    }

    pub fn try_parse(id: &str) -> anyhow::Result<Self> {
        Self::split_on(id, ':')
    }

    pub fn split_on(id: &str, delimiter: char) -> anyhow::Result<Self> {
        match id.split_once(delimiter) {
            Some(arr) => Self::new(arr.0, arr.1),
            None => Self::new("rimecraft", id),
        }
    }

    fn is_namespace_valid(namespace: &str) -> bool {
        for c in namespace.chars() {
            if !(c == '_' || c == '-' || c >= 'a' || c <= 'z' || c >= '0' || c <= '9' || c == '.') {
                return false;
            }
        }
        true
    }

    fn is_path_valid(path: &str) -> bool {
        for c in path.chars() {
            if !(c == '_'
                || c == '-'
                || c >= 'a'
                || c <= 'z'
                || c >= '0'
                || c <= '9'
                || c == '.'
                || c == '/')
            {
                return false;
            }
        }
        true
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_str(":")?;
        f.write_str(&self.path)?;
        std::fmt::Result::Ok(())
    }
}

impl serde::Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let str = String::deserialize(deserializer)?;
        Self::try_parse(str.as_str()).map_err(|_| {
            D::Error::invalid_value(
                serde::de::Unexpected::Str(str.as_str()),
                &"string with a ':' separated and which chars are in [a-z0-9/._-]",
            )
        })
    }
}

/// Describes a var int.
pub struct VarI32(pub i32);

/// Represents types of enum that can be itered with values, like Java.
pub trait EnumValues<const N: usize>: Sized + Clone + Copy + PartialEq + Eq {
    fn values() -> [Self; N];
}
