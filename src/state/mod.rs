pub mod property;

use std::{hash::Hash, ops::Deref};

pub use property::Property;

pub struct State {
    id: usize,
    entries: hashbrown::HashMap<property::Property, u8>,
    table: once_cell::sync::OnceCell<hashbrown::HashMap<property::Property, Vec<usize>>>,
}

impl State {
    pub fn cycle(&self, property: &property::Property) -> anyhow::Result<usize> {
        self.with(property, {
            let range = property.range();
            let mut value = *self.entries.get(property).ok_or_else(|| {
                anyhow::anyhow!("Cannot set property {property:?} as it does not exist")
            })?;
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
    ) -> anyhow::Result<usize> {
        let i = value.into();
        let peq = *self.entries.get(property).ok_or_else(|| {
            anyhow::anyhow!("Cannot set property {property:?} as it does not exist")
        })?;

        if peq == i {
            Ok(self.id)
        } else {
            self.table
                .get()
                .unwrap()
                .get(property)
                .unwrap()
                .get((i - property.range().0) as usize)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot set property {property:?} to {i}, it's not an allowed value"
                    )
                })
                .copied()
        }
    }

    pub fn with_or_default<T: Into<u8>>(
        &self,
        property: &property::Property,
        value: T,
    ) -> anyhow::Result<usize> {
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
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot set property {property:?} to {i}, it's not an allowed value"
                    )
                })
                .copied()
        }
    }

    fn init_table<T: Deref<Target = Self>>(&self, states: &[T]) {
        assert!(self.table.get().is_none());
        let mut table = hashbrown::HashMap::new();
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

    pub fn entries(&self) -> &hashbrown::HashMap<property::Property, u8> {
        &self.entries
    }
}

pub struct States<T: Deref<Target = State> + 'static> {
    def: usize,
    properties: hashbrown::HashMap<String, property::Property>,
    states: Vec<crate::util::Ref<'static, T>>,
}

impl<T: Deref<Target = State> + 'static> States<T> {
    pub fn states(&self) -> &[crate::util::Ref<'static, T>] {
        &self.states
    }

    pub fn from_id(&self, id: usize) -> Option<crate::util::Ref<T>> {
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
            entries: crate::Ref(shared),
            value: shared.states[id],
        }
    }
}

/// A shared state with states reference count and the index
/// which is cheap to clone.
pub struct Shared<T: Deref<Target = State> + 'static> {
    pub entries: crate::util::StaticRef<crate::state::States<T>>,
    value: crate::util::StaticRef<T>,
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
    def_state: hashbrown::HashMap<property::Property, u8>,
    properties: hashbrown::HashMap<String, property::Property>,
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
        let mut entries: hashbrown::HashMap<property::Property, u8> = hashbrown::HashMap::new();

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
        states: states.into_iter().map(crate::util::Ref::from).collect(),
    }
}

#[derive(Default)]
pub struct StatesBuilder {
    properties: hashbrown::HashMap<String, property::Property>,
}

impl StatesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, property: property::Property) -> anyhow::Result<()> {
        let name = property.name();

        {
            let matcher = todo!();
            if matcher.is_match(name) {
                return Err(anyhow::anyhow!("Invalidly named property: {name}"));
            }

            let c = property.values::<u8>();
            if c.len() <= 1 {
                return Err(anyhow::anyhow!(
                    "Attempted use property {name} with <= 1 possible values"
                ));
            }

            if self.properties.contains_key(name) {
                return Err(anyhow::anyhow!("Duplicated property: {name}"));
            }
        }

        self.properties.insert(name.to_string(), property);

        Ok(())
    }

    pub fn build<E: Clone, S: Deref<Target = State> + From<(E, State)>>(
        self,
        data: E,
        def_state: hashbrown::HashMap<property::Property, u8>,
    ) -> States<S> {
        new_states(data, def_state, self.properties)
    }
}
