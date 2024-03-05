//! Macro rules.

#[cfg(feature = "vanilla")]
#[macro_export]
macro_rules! format_identifier {
	($namespace:expr; $($($word:expr),*);*) => {
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

			Identifier::new($namespace, Path::new_formatted(locations))
		}
	};
}