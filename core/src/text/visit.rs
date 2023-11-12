use super::Style;

pub type Terminate = ();

pub trait Visitable {
    fn plain_visit<T, V: PlainVisitor<T>>(visitor: V) -> Option<T>;
    fn styled_visit<T, V: StyledVisitor<T>>(visitor: V) -> Option<T>;
}

/// Referring to a empty [`Visitable`]
impl Visitable for () {
    #[inline]
    fn plain_visit<T, V: PlainVisitor<T>>(_visitor: V) -> Option<T> {
        None
    }

    #[inline]
    fn styled_visit<T, V: StyledVisitor<T>>(_visitor: V) -> Option<T> {
        None
    }
}

pub trait StyledVisitor<T> {
    fn accept(style: &Style, as_string: &String) -> Option<T>;
}

pub trait PlainVisitor<T> {
    fn accept(as_string: &String) -> Option<T>;
}
