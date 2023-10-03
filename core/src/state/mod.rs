pub mod property;

use std::{hash::Hash, ops::Deref};

pub use property::Property;

pub struct State {
    id: usize,
    entries: std::collections::HashMap<property::Property, u8>,
    table: once_cell::sync::OnceCell<std::collections::HashMap<property::Property, Vec<usize>>>,
}

impl State {
    pub fn cycle(&self, property: &property::Property) -> Result<usize, StateError> {
        self.with(property, {
            let range = property.range();
            let mut value = *self
                .entries
                .get(property)
                .ok_or_else(|| StateError::InvalidProperty(property.clone()))?;

            if value >= range.1 {
                value = range.0;
            } else {
                value += 1;
            }

            value
        })
    }

    pub fn with<T: Into<u8>>(
        &self,
        property: &property::Property,
        value: T,
    ) -> Result<usize, StateError> {
        let i = value.into();
        let peq = *self
            .entries
            .get(property)
            .ok_or_else(|| StateError::InvalidProperty(property.clone()))?;

        if peq == i {
            Ok(self.id)
        } else {
            self.table
                .get()
                .unwrap()
                .get(property)
                .unwrap()
                .get((i - property.range().0) as usize)
                .ok_or_else(|| StateError::InvalidValue {
                    property: property.clone(),
                    value: i,
                })
                .copied()
        }
    }

    pub fn with_or_default<T: Into<u8>>(
        &self,
        property: &property::Property,
        value: T,
    ) -> Result<usize, StateError> {
        let i = value.into();
        let peq = self.entries.get(property).copied();

        if match peq {
            Some(e) => e == i,
            None => true,
        } {
            Ok(self.id)
        } else {
            self.table
                .get()
                .unwrap()
                .get(property)
                .unwrap()
                .get((i - property.range().0) as usize)
                .ok_or_else(|| StateError::InvalidValue {
                    property: property.clone(),
                    value: i,
                })
                .copied()
        }
    }

    fn init_table<T: Deref<Target = Self>>(&self, states: &[T]) {
        assert!(self.table.get().is_none(), "table already initialized");
        let mut table = std::collections::HashMap::new();

        for p in self.entries.keys() {
            let range = p.range();
            let mut vec = Vec::new();

            for i in range.0..=range.1 {
                let iindex = i - range.0;

                vec.push(
                    states
                        .iter()
                        .find(|state| {
                            self.entries.keys().all(|k| {
                                state.entries.get(k).map_or(false, |value| {
                                    if k == p {
                                        iindex == *value
                                    } else {
                                        self.entries.get(k).map_or(false, |v| value == v)
                                    }
                                })
                            })
                        })
                        .unwrap()
                        .id,
                );
            }

            table.insert(p.clone(), vec);
        }

        self.table.get_or_init(|| table);
    }

    pub fn properties(&self) -> Vec<&property::Property> {
        self.entries.keys().collect::<Vec<_>>()
    }

    pub fn contains(&self, property: &property::Property) -> bool {
        self.entries.contains_key(property)
    }

    pub fn get<T: From<u8>>(&self, property: &property::Property) -> Option<T> {
        self.entries.get(property).copied().map(T::from)
    }

    pub fn entries(&self) -> &std::collections::HashMap<property::Property, u8> {
        &self.entries
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("property {0:?} doesn't exist")]
    InvalidProperty(property::Property),
    #[error("{value} it's not an allowed value of {property:?}")]
    InvalidValue { property: Property, value: u8 },
}

pub struct States<T: Deref<Target = State> + 'static> {
    def: usize,
    properties: std::collections::HashMap<String, property::Property>,
    states: Vec<rimecraft_primitives::Ref<'static, T>>,
}

impl<T: Deref<Target = State> + 'static> States<T> {
    pub fn states(&self) -> &[rimecraft_primitives::Ref<'static, T>] {
        &self.states
    }

    pub fn from_id(&self, id: usize) -> Option<rimecraft_primitives::Ref<T>> {
        self.states.get(id).copied()
    }

    pub fn default_state(&self) -> &T {
        self.states.get(self.def).unwrap()
    }

    pub fn properties(&self) -> Vec<&property::Property> {
        self.properties.values().collect()
    }

    pub fn get_property(&self, name: &str) -> Option<&property::Property> {
        self.properties.get(name)
    }

    pub fn get_shared(shared: &'static crate::state::States<T>, id: usize) -> Shared<T> {
        Shared {
            entries: rimecraft_primitives::Ref(shared),
            value: shared.states[id],
        }
    }
}

/// A shared state with states reference count and the index.
pub struct Shared<T: Deref<Target = State> + 'static> {
    pub entries: rimecraft_primitives::Ref<'static, crate::state::States<T>>,
    pub value: rimecraft_primitives::Ref<'static, T>,
}

impl<T: Deref<Target = State>> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.0
    }
}

impl<T: Deref<Target = State>> Copy for Shared<T> {}

impl<T: Deref<Target = State>> Clone for Shared<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Deref<Target = State>> Eq for Shared<T> {}

impl<T: Deref<Target = State>> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Deref<Target = State>> Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

fn new_states<E: Clone, T: Deref<Target = State> + From<(E, State)>>(
    data: E,
    def_state: std::collections::HashMap<property::Property, u8>,
    properties: std::collections::HashMap<String, property::Property>,
) -> States<T> {
    let mut states_raw: Vec<State> = Vec::new();
    let mut temp: Vec<Vec<(property::Property, u8)>> = Vec::new();

    for property in properties.values() {
        temp = temp
            .iter()
            .flat_map(|list| {
                property.values::<u8>().into_iter().map(|i| {
                    let mut list2 = list.clone();
                    list2.push((property.clone(), i));
                    list2
                })
            })
            .collect()
    }

    for list2 in temp {
        let mut entries: std::collections::HashMap<property::Property, u8> =
            std::collections::HashMap::new();

        for e in list2 {
            entries.insert(e.0, e.1);
        }

        states_raw.push(State {
            id: 0,
            entries,
            table: once_cell::sync::OnceCell::new(),
        });
    }

    states_raw.iter_mut().enumerate().for_each(|e| e.1.id = e.0);
    let states: Vec<T> = states_raw
        .into_iter()
        .map(|e| T::from((data.clone(), e)))
        .collect();
    states.iter().for_each(|e| e.init_table(&states));

    States {
        def: states
            .iter()
            .enumerate()
            .find(|e| {
                def_state
                    .iter()
                    .all(|ee| ee.1 == e.1.deref().entries.get(ee.0).unwrap())
            })
            .map_or(0, |e| e.0),
        properties,
        states: states
            .into_iter()
            .map(rimecraft_primitives::Ref::from)
            .collect(),
    }
}

#[derive(Default)]
pub struct StatesBuilder {
    properties: std::collections::HashMap<String, property::Property>,
}

impl StatesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, property: property::Property) -> Result<(), StatesBuilderError> {
        let name = property.name();

        {
            if lazy_regex::regex_is_match!("^[a-z0-9_]+$", name) {
                return Err(StatesBuilderError::InvalidPropertyName {
                    name: name.to_string(),
                });
            }

            {
                let len = property.values::<u8>().len();

                if len <= 1 {
                    return Err(StatesBuilderError::PropertyLeOnePossibleValues {
                        name: name.to_string(),
                        len,
                    });
                }
            }

            if self.properties.contains_key(name) {
                return Err(StatesBuilderError::PropertyDuplicated {
                    name: name.to_string(),
                });
            }
        }

        self.properties.insert(name.to_string(), property);

        Ok(())
    }

    pub fn build<E: Clone, S: Deref<Target = State> + From<(E, State)>>(
        self,
        data: E,
        def_state: std::collections::HashMap<property::Property, u8>,
    ) -> States<S> {
        new_states(data, def_state, self.properties)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StatesBuilderError {
    #[error("invalidly named property: {name}")]
    InvalidPropertyName { name: String },
    #[error("property {name} has {len} possible values which is lesser than or equal to 1")]
    PropertyLeOnePossibleValues { name: String, len: usize },
    #[error("property duplicated: {name}")]
    PropertyDuplicated { name: String },
}
