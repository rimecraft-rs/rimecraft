use super::Identifier;

pub struct Event<I, O> {
    phases: Vec<(Identifier, Vec<Box<(dyn Fn(I) -> O + Send + Sync)>>)>,
    invoker: Box<(dyn Fn(Vec<&(dyn Fn(I) -> O + Send + Sync)>, I) -> O + Send + Sync)>,
    default_impl: Box<(dyn Fn(I) -> O + Send + Sync)>,
}

impl<I, O: Default> Event<I, O> {
    pub fn new_default(
        invoker: Box<dyn Fn(Vec<&(dyn Fn(I) -> O + Send + Sync)>, I) -> O + Send + Sync>,
    ) -> Self {
        Self {
            phases: vec![(default_phase(), Vec::new())],
            invoker,
            default_impl: Box::new(|_i| O::default()),
        }
    }
}

impl<I, O> Event<I, O> {
    pub fn new(
        invoker: Box<dyn Fn(Vec<&(dyn Fn(I) -> O + Send + Sync)>, I) -> O + Send + Sync>,
        empty_impl: Box<dyn Fn(I) -> O + Send + Sync>,
        mut phases: Vec<Identifier>,
    ) -> Self {
        if phases.is_empty() {
            phases.push(default_phase());
        }

        Self {
            phases: phases.iter().map(|id| (id.clone(), Vec::new())).collect(),
            invoker,
            default_impl: empty_impl,
        }
    }

    pub fn invoke(&self, input: I) -> O {
        if self.phases.is_empty() {
            self.default_impl.as_ref()(input)
        } else {
            self.invoker.as_ref()(
                {
                    let mut vec = Vec::new();
                    for phase in &self.phases {
                        vec.append(&mut phase.1.iter().map(|b| b.as_ref()).collect());
                    }
                    vec
                },
                input,
            )
        }
    }

    pub fn register(
        &mut self,
        callback: Box<dyn Fn(I) -> O + Send + Sync>,
        phase: &Identifier,
    ) -> bool {
        match self.phases.iter_mut().find(|p| p.0.eq(phase)) {
            Some(phase) => {
                phase.1.push(callback);
                true
            }
            None => false,
        }
    }

    pub fn register_default(&mut self, callback: Box<dyn Fn(I) -> O + Send + Sync>) -> bool {
        self.register(callback, &default_phase())
    }

    pub fn get_phases(&self) -> Vec<Identifier> {
        self.phases.iter().map(|p| p.0.to_owned()).collect()
    }
}

pub fn default_phase() -> Identifier {
    Identifier::parse("default_phase".to_string()).unwrap()
}
