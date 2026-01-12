//! Macro rules for generating vanilla identifiers.

/// Creates a new [`Identifier<Namespace, Path>`] with formatted literals.
///
/// # Examples
///
/// ```
/// # use rimecraft_identifier::{*, vanilla::*};
/// let identifier = format_identifier!("namespace".parse().unwrap() =>
///     "a", "b"; "c"; "42"
/// );
/// assert_eq!("namespace:a_b/c/42", identifier.to_string());
///
/// let identifier = format_identifier!(MINECRAFT =>
///     "tags"; "piglin", "repellents"
/// );
/// assert_eq!("minecraft:tags/piglin_repellents", identifier.to_string());
/// ```
#[cfg(feature = "vanilla")]
#[macro_export]
macro_rules! format_identifier {
	($namespace:expr => $($($word:expr),*);*) => {
		{
			$crate::Identifier::<$crate::vanilla::Namespace, $crate::vanilla::Path>::new(
				$namespace,
				$crate::vanilla::Path::new_formatted(
					::std::vec![
						$(::std::vec![$($word),*]),*
					])
			)
		}
	};
}
