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
        let mut parts = $crate::__priv_macro_use::Vec::<$crate::__priv_macro_use::String>::new();
        $({
            let s = $crate::__priv_macro_use::ToString::to_string(&$word);
            if !str::is_empty(&s) {
                $crate::__priv_macro_use::Vec::push(&mut parts, s);
            }
        })*
        <[$crate::__priv_macro_use::String]>::join::<&str>(&parts, ".")
    }};
}
