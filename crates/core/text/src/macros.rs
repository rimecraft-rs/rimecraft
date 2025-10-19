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
        let mut parts = ::std::vec::Vec::new();
        $(
            let s = ::std::string::ToString::to_string(&$word);
            if !s.is_empty() {
                parts.push(s);
            }
        )*
        parts.join(".")
    }};
}
