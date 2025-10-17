use crate::Style;

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
