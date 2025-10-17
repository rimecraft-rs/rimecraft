use crate::Style;

/// The Unicode Replacement Character '�' (U+FFFD).
pub const REPLACEMENT_CHAR: char = '\u{FFFD}';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VisitResult {
    Continue,
    Break,
}

pub trait CharVisitor {
    fn visit<Ext>(&mut self, i: i32, style: Style<Ext>, c: char) -> VisitResult;

    fn map<F>(self, f: F) -> impl CharVisitor
    where
        Self: Sized,
        F: Fn(char) -> char + 'static,
    {
        struct Mapped<V, F> {
            visitor: V,
            func: F,
        }

        impl<V, F> CharVisitor for Mapped<V, F>
        where
            V: CharVisitor,
            F: Fn(char) -> char,
        {
            fn visit<Ext>(&mut self, i: i32, style: Style<Ext>, c: char) -> VisitResult {
                self.visitor.visit(i, style, (self.func)(c))
            }
        }

        Mapped {
            visitor: self,
            func: f,
        }
    }
}

pub mod factory {
    use crate::Style;
    use rimecraft_fmt::Formatting;

    // The `chars()` iterator automatically handles UTF-8 decoding
    // and yields valid Unicode scalar values (char).

    /// Visits characters in text from start to end.
    ///
    /// Each character is passed to the visitor with its index and the provided style.
    pub fn visit_forwards<V, Ext>(
        text: &str,
        style: Style<Ext>,
        visitor: &mut V,
    ) -> super::VisitResult
    where
        V: super::CharVisitor,
        Ext: Clone,
    {
        for (i, c) in text.chars().enumerate() {
            match visitor.visit(i as i32, style.clone(), c) {
                super::VisitResult::Continue => {}
                super::VisitResult::Break => return super::VisitResult::Break,
            }
        }
        super::VisitResult::Continue
    }

    /// Visits characters in text from end to start.
    ///
    /// Each character is passed to the visitor with its index and the provided style.
    pub fn visit_backwards<V, Ext>(
        text: &str,
        style: Style<Ext>,
        visitor: &mut V,
    ) -> super::VisitResult
    where
        V: super::CharVisitor,
        Ext: Clone,
    {
        for (i, c) in text.chars().rev().enumerate() {
            match visitor.visit(i as i32, style.clone(), c) {
                super::VisitResult::Continue => {}
                super::VisitResult::Break => return super::VisitResult::Break,
            }
        }
        super::VisitResult::Continue
    }

    /// Visits the code points of a string, applying the formatting codes within.
    /// The visit is in forward direction.
    pub fn visit_formatted<V, Ext>(
        text: &str,
        start_index: usize,
        starting_style: Style<Ext>,
        reset_style: Style<Ext>,
        visitor: &mut V,
    ) -> super::VisitResult
    where
        V: super::CharVisitor,
        Ext: Clone + Default,
    {
        let mut style = starting_style;
        let mut chars = text.chars().enumerate().skip(start_index).peekable();

        while let Some((i, c)) = chars.next() {
            // Check for formatting code prefix (§ = U+00A7)
            if c == Formatting::CODE_PREFIX {
                // Peek at the next character to see if it's a valid formatting code
                if let Some(&(_, code_char)) = chars.peek()
                    && let Ok(formatting) = char::try_into(code_char)
                {
                    // Valid formatting code found
                    style = if formatting == Formatting::Reset {
                        reset_style.clone()
                    } else {
                        style.with_exclusive_formatting(formatting)
                    };
                    // Skip the code character
                    chars.next();
                    continue;
                }
                // If we reach here, '§' was at the end or followed by invalid code
                // Break out of the loop
                break;
            }

            // Regular character - visit it
            match visitor.visit(i as i32, style.clone(), c) {
                super::VisitResult::Continue => {}
                super::VisitResult::Break => return super::VisitResult::Break,
            }
        }

        super::VisitResult::Continue
    }
}
