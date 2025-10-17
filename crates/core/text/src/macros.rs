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
/// A simple example from [`crate::ordered_text::empty`]:
///
/// ```
/// ordered_text! {
///     _visitor => {
///         VisitResult::Continue
///     }
/// }
/// ```
///
/// ...will expand to something like:
///
/// ```
/// struct Impl; // Hygienic name
///
/// impl crate::ordered_text::OrderedText for Impl {
///     fn accept<V>(&self, _visitor: &mut V) -> crate::visitor::VisitResult
///     where
///         V: crate::visitor::CharVisitor
///     {
///         VisitResult::Continue
///     }
/// }
/// ```
///
/// You can also define fields and generics like in [`crate::ordered_text::styled`]:
///
/// ```
/// ordered_text! {
///     <Ext> {
///         c: char,
///         style: Style<Ext>,
///     } where Ext: Clone;
///     visitor => {
///         visitor.visit(0, style.clone(), c)
///     }
/// }
/// ```
///
/// ...will expand to something like:
///
/// ```
/// struct Impl<Ext> {
///     c: char,
///     style: Style<Ext>,
/// }
///
/// impl<Ext> crate::ordered_text::OrderedText for Impl<Ext>
/// where
///     Ext: Clone,
/// {
///     fn accept<V>(&self, visitor: &mut V) -> crate::visitor::VisitResult
///     where
///         V: crate::visitor::CharVisitor
///     {
///         let c = &self.c;
///         let style = &self.style;
///         visitor.visit(0, style.clone(), c)
///     }
/// }
/// ```
#[macro_export]
macro_rules! ordered_text {
    ($($(<$($generic:ident),*>)? {
        $($name:ident: $type:ty),* $(,)?
    } $(where $($bound_id:ident: $bound:tt),*)? ;)?

    $visitor:ident => {
        $($body:expr)*
    }) => {
        {
            struct Impl<$($($($generic),*)?)?> {
                $($($name: $type),*)?
            }

            impl<$($($($generic),*)?)?> $crate::ordered_text::OrderedText for Impl<$($($($generic),*)?)?>
            where
                    $($($($bound_id: $bound),*)?)?
            {
                fn accept<V>(&self, $visitor: &mut V) -> $crate::visitor::VisitResult
                where
                    V: $crate::visitor::CharVisitor
                {
                    $(
                        $(let $name = &self.$name; )*
                    )?
                    $($body)*
                }
            }

            Impl {
                $($($name),*)?
            }
        }
    };
}
