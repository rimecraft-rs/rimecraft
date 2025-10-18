//! Macro rules.

/// Creates a new localization key with literals.
///
/// # Examples
///
/// ```
/// # use rimecraft_text::format_localization_key;
/// let key = format_localization_key![
///     "category", "id", "path"
/// ];
/// assert_eq!("category.id.path", key);
/// ```
#[macro_export]
macro_rules! format_localization_key {
    ($($word:expr),* $(,)?) => {{
        [$($word.to_string()),*].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(".")
    }};
}
