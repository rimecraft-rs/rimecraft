//! Macro rules.

/// Creates a new localization key with expressions.
///
/// # Examples
///
/// ```
/// # use rimecraft_text::format_localization_key;
/// const KEY: &str = format_localization_key!["category", "id", "path"];
/// assert_eq!("category.id.path", KEY);
///
/// let key = format_localization_key![
///     "category", "id", { let path = "path"; path }
/// ];
/// assert_eq!("category.id.path", key);
/// ```
#[macro_export]
macro_rules! format_localization_key {
    ($(,)?) => {
        ""
    };
    ($($word:literal),*$(,)?) => {
        $crate::__priv_macro_use::strip_dot_prefix($crate::__priv_macro_use::concat!($('.', $word),*))
    };
    ($($word:expr),*$(,)?) => {{
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
