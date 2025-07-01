use crate::*;

use self::vanilla::{MINECRAFT, Namespace, Path};

#[test]
fn create_identifiers() {
    let path = Path::new("path");
    let identifier = Identifier::new(MINECRAFT, path);
    assert_eq!("minecraft:path", identifier.to_string());

    let path = Path::try_new_formatted(vec![vec!["a", "b", ""], vec![], vec!["42"]]).unwrap();
    let identifier = Identifier::new(Namespace::new("n"), path);
    assert_eq!("n:a_b/42", identifier.to_string());

    let identifier = format_identifier!("namespace".parse().unwrap() => "a", "b"; "c"; "42");
    assert_eq!("namespace:a_b/c/42", identifier.to_string());
}
