use crate::Style;

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
        // Checks for formatting code prefix (§ = U+00A7)
        if c == Formatting::CODE_PREFIX {
            // Peeks at the next character to see if it's a valid formatting code
            if let Some(&(_, code_char)) = chars.peek()
                && let Ok(formatting) = char::try_into(code_char)
            {
                // Valid formatting code found
                style = if formatting == Formatting::Reset {
                    reset_style.clone()
                } else {
                    style.with_exclusive_formatting(formatting)
                };
                // Skips the code character
                chars.next();
                continue;
            }
            // If we reach here, '§' was at the end or followed by invalid code
            // Breaks out of the loop
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

pub fn visit_formatted_with_style<V, Ext>(
    text: &str,
    start_index: usize,
    style: Style<Ext>,
    visitor: &mut V,
) -> super::VisitResult
where
    V: super::CharVisitor,
    Ext: Clone + Default,
{
    visit_formatted(text, start_index, style.clone(), style.clone(), visitor)
}

pub fn visit_formatted_with_style_from_start<V, Ext>(
    text: &str,
    style: Style<Ext>,
    visitor: &mut V,
) -> super::VisitResult
where
    V: super::CharVisitor,
    Ext: Clone + Default,
{
    visit_formatted_with_style(text, 0, style, visitor)
}
