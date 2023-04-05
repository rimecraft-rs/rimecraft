use crate::util::Identifier;

pub fn root_key() -> Identifier {
    Identifier::parse(String::from("root")).unwrap()
}
