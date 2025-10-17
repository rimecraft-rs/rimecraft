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
	($($word:expr),*) => {
		{
			[$($word),*].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(".")
		}
	};
}

/// Implements an [`crate::ordered_text::OrderedText`].
///
/// You can define generics, named fields, and where clauses for the implementation struct.
/// Then, you should name the visitor variable and provide the body of the `accept` method.
/// Any named fields will be automatically available in the body as references.
///
/// # Examples
///
/// A simple example from [`crate::iter_text::styled`]:
///
/// ```
/// iter_text! {
///     <StyleExt> where StyleExt: Clone;
///     (c: char, style: Style<StyleExt>) -> StyleExt;
///     std::iter::once(IterTextItem {
///         index: 0,
///         c: c.to_owned(),
///         style: style.to_owned(),
///     })
/// }
/// ```
///
/// ...will expand to something like:
///
/// ```
/// struct Impl<StyleExt> {
///     c: char,
///     style: crate::Style<StyleExt>,
/// }
///
/// impl<StyleExt> crate::iter_text::IterText<StyleExt> for Impl<StyleExt>
/// where
///     StyleExt: Clone,
/// {
///     fn iter_text(&self) -> impl Iterator<Item = crate::iter_text::IterTextItem<StyleExt>> + '_ {
///         let c = &self.c;
///         let style = &self.style;
///         std::iter::once(IterTextItem {
///             index: 0,
///             c: c.to_owned(),
///             style: style.to_owned(),
///         })
///     }
/// }
/// ```
#[macro_export]
macro_rules! iter_text {
    (
        $(<$($generic:ident),*>)? $(where $($bound_id:ident: $bound:tt),*)? ;
        ($($name:ident: $type:ty),*) -> $res_ty:ty;
        $body:expr
    ) => {
        {
            struct Impl<$($($generic),*)?> {
                _phantom: std::marker::PhantomData<$res_ty>,
                $($name: $type),*
            }

            impl<$($($generic),*)?> $crate::iter_text::IterText<$res_ty> for Impl<$($($generic),*)?>
            where
                    $($($bound_id: $bound),*)?
            {
                fn iter_text(&self) -> impl Iterator<Item = $crate::iter_text::IterTextItem<$res_ty>> + '_ {
                    $(let $name = &self.$name; )*
                    $body
                }
            }

            Impl {
                _phantom: std::marker::PhantomData,
                $($name),*
            }
        }
    };
}
