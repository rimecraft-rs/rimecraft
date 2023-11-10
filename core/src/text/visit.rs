use crate::Unit;

pub trait Visitable<T> {
    const TERMINATE_VISIT:Option<Unit>=Some(Unit::Instance);
}
struct EmptyVisitable;
impl Visitable<T> for EmptyVisitable {
    
}
