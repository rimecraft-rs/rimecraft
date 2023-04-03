pub struct RenderCall {
    executor: Box<dyn Fn()>,
}

impl RenderCall {
    pub fn new(executor: Box<impl Fn() + 'static>) -> Self {
        Self { executor }
    }

    pub fn execute(&self) {
        (self.executor)()
    }
}
