pub type Terminate = ();

pub trait Visitable<T> {}

impl<T> Visitable<T> for () {}
