//! Macro rules.

#[allow(unused_imports)]
use crate::*; // Used in docs

/// Creates a new [`Identifier<Namespace, Path>`] with formatted literals.
///
/// # Examples
///
/// ```
/// use rimecraft_identifier::{format_identifier, Identifier, vanilla::{Namespace, Path, MINECRAFT}};
///
/// let identifier = format_identifier!("namespace".parse().unwrap() =>
/// 	"a", "b"; "c"; "42"
/// );
/// assert_eq!("namespace:a_b/c/42", identifier.to_string());
///
/// let identifier = format_identifier!(MINECRAFT =>
/// 	"tags"; "piglin", "repellents"
/// );
/// assert_eq!("minecraft:tags/piglin_repellents", identifier.to_string());
/// ```
#[cfg(feature = "vanilla")]
#[macro_export]
macro_rules! format_identifier {
	($namespace:expr => $($($word:expr),*);*) => {
		{
			let mut locations = Vec::new();

			$(
				{
					let mut words = Vec::new();

					$(
						words.push($word);
					)*

					locations.push(words);
				}
			)*

			Identifier::<Namespace, Path>::new($namespace, Path::new_formatted(locations))
		}
	};
}
