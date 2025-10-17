use crate::{
    Style, ordered_text,
    visitor::{CharVisitor, VisitResult},
};

pub trait OrderedText {
    fn accept<V>(&self, visitor: &mut V) -> VisitResult
    where
        V: CharVisitor;
}

pub fn empty() -> impl OrderedText {
    ordered_text! {
        _visitor => {
            VisitResult::Continue
        }
    }
}

pub fn styled<Ext>(c: char, style: Style<Ext>) -> impl OrderedText
where
    Ext: Clone,
{
    ordered_text! {
        <Ext> {
            c: char,
            style: Style<Ext>,
        } where Ext: Clone;

        visitor => {
            visitor.visit(0, style.clone(), *c)
        }
    }
}

pub fn styled_forwards_visited_string<Ext>(s: &str, style: Style<Ext>) -> impl OrderedText
where
    Ext: Clone,
{
    let s = s.to_owned();

    ordered_text! {
        <Ext> {
            s: String,
            style: Style<Ext>,
        } where Ext: Clone;

        visitor => {
            for (i, c) in s.chars().enumerate() {
                match visitor.visit(i as i32, style.clone(), c) {
                    VisitResult::Continue => {},
                    VisitResult::Break => return VisitResult::Break,
                }
            }
            VisitResult::Continue
        }
    }
}
