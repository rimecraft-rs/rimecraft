use std::borrow::Cow;

use super::Style;

/// Types that can supply strings to a [`Visitor`].
pub trait Visit {
    /// Supplies this visitable's literal content to the visitor.
    /// Returns `None` if the visit finished, or a terminating
    /// result from the visitor.
    fn visit<T, V: Visitor<T>>(&self, visitor: V) -> Option<T>;
}

macro_rules! erased_text_visit {
    ($($v:vis trait $n:ident, $t:ty => $vi:ty);+) => {
        $($v trait $n { fn visit(&self, visitor: $vi) -> Option<$t>; }
        impl<T: Visit> $n for T { #[inline] fn visit(&self, visitor: $vi) -> Option<$t> { Visit::visit(self, visitor) } })+
    };
}

erased_text_visit! {
    pub trait ErasedVisit, () => &mut dyn Visitor<()>
}

/// Creates a `Visit` from a plain string.
#[inline]
pub fn plain(s: &str) -> Plain<'_> {
    Plain(Cow::Borrowed(s))
}

/// Types that can supply strings to a [`StyleVisitor`] with a style context.
pub trait StyledVisit {
    /// Supplies this visitable's literal content and contextual style
    /// to the visitor.
    /// Returns `None` if the visit finished, or a terminating
    /// result from the visitor.
    fn visit<T, V: StyleVisitor<T>>(&self, visitor: V, style: &Style) -> Option<T>;
}

macro_rules! erased_text_styled_visit {
    ($($v:vis trait $n:ident, $t:ty => $vi:ty);+) => {
        $($v trait $n { fn visit(&self, visitor: $vi, style: &Style) -> Option<$t>; }
        impl<T: StyledVisit> $n for T { #[inline] fn visit(&self, visitor: $vi, style: &Style) -> Option<$t> { StyledVisit::visit(self, visitor, style) } })+
    };
}

erased_text_styled_visit! {
    pub trait ErasedVisitStyled, Style => &mut dyn StyleVisitor<Style>
}

/// Creates a `Visit` from a plain string and a root style.
#[inline]
pub fn styled(s: &str, style: Style) -> Styled<'_> {
    Styled(Cow::Borrowed(s), style)
}

impl Visit for () {
    fn visit<T, V: Visitor<T>>(&self, _: V) -> Option<T> {
        None
    }
}

impl StyledVisit for () {
    fn visit<T, V: StyleVisitor<T>>(&self, _: V, _: &Style) -> Option<T> {
        None
    }
}

/// A visit for string content.
pub trait Visitor<T> {
    /// Visits a literal string.
    ///
    /// When `Some` is returned, the visit is terminated before
    /// visiting all text. Can return [`Some`] for convenience.
    fn accept(&mut self, as_str: &str) -> Option<T>;
}

/// A visitor for string content and a contextual [`Style`].
pub trait StyleVisitor<T> {
    /// Visits a string's content with a contextual style.
    ///
    /// A contextual style is obtained by calling [`Style::with_parent`]
    /// on the current's text style, passing the previous contextual text style
    /// or the starting style if it is the beginning of a visit.
    ///
    /// When `Some` is returned, the visit is terminated before
    /// visiting all text. Can return [`Some`] for convenience.
    fn accept(&mut self, style: &Style, as_str: &str) -> Option<T>;
}

impl<'a, T, V> Visitor<T> for &'a mut V
where
    V: Visitor<T> + ?Sized,
{
    fn accept(&mut self, as_str: &str) -> Option<T> {
        (**self).accept(as_str)
    }
}

impl<'a, T, V> StyleVisitor<T> for &'a mut V
where
    V: StyleVisitor<T> + ?Sized,
{
    fn accept(&mut self, style: &Style, as_str: &str) -> Option<T> {
        (**self).accept(style, as_str)
    }
}

/// The `Visit` returned from [`plain`].
pub struct Plain<'a>(Cow<'a, str>);

impl<'a> Visit for Plain<'a> {
    fn visit<T, V: Visitor<T>>(&self, mut visitor: V) -> Option<T> {
        visitor.accept(&self.0)
    }
}

impl<'a> StyledVisit for Plain<'a> {
    fn visit<T, V: StyleVisitor<T>>(&self, mut visitor: V, style: &Style) -> Option<T> {
        visitor.accept(style, &self.0)
    }
}

/// The `Visit` returned from [`styled`].
pub struct Styled<'a>(Cow<'a, str>, Style);

impl<'a> Visit for Styled<'a> {
    fn visit<T, V: Visitor<T>>(&self, mut visitor: V) -> Option<T> {
        visitor.accept(&self.0)
    }
}

impl<'a> StyledVisit for Styled<'a> {
    fn visit<T, V: StyleVisitor<T>>(&self, mut visitor: V, style: &Style) -> Option<T> {
        visitor.accept(&self.1.clone().with_parent(style.clone()), &self.0)
    }
}

/// A visitor for single characters in a string.
pub trait CharVisitor {
    /// Visits a single character.
    ///
    /// Multiple surrogate characters are converted into one single `code_point`
    /// when passed into this method.
    ///
    /// Returns `true` to continue visiting other characters, or `false` to
    /// terminate the visit.
    fn accept(&mut self, index: usize, style: &Style, code_point: u32) -> bool;
}

impl<T> CharVisitor for T
where
    T: (FnMut(usize, &Style, u32) -> bool) + ?Sized,
{
    #[inline]
    fn accept(&mut self, index: usize, style: &Style, code_point: u32) -> bool {
        self(index, style, code_point)
    }
}
