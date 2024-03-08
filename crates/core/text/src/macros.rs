//! Macro rules.

/// Creates a new localization key with literals.
///
/// # Examples
///
/// ```
/// use rimecraft_identifier::format_localization_key;
///
/// let key = format_localization_key!(
/// 	"category", "id", "path"
/// );
/// assert_eq!("category.id.path", key);
/// ```
#[macro_export]
macro_rules! format_localization_key {
	($($word:expr),*) => {
		{
			let mut words: Vec<String> = Vec::new();

			$(
				words.push($word.to_string());
			)*

			words.into_iter().filter(|s| s.len() > 0).collect::<Vec<String>>().join(".")
		}
	};
}