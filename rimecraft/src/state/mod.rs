use std::ops::Deref;

pub mod property;

// id: index inside a state manager.

pub struct RawState {
    id: usize,
    entries: std::collections::HashMap<property::Property, u8>,
    table: once_cell::sync::OnceCell<std::collections::HashMap<property::Property, Vec<usize>>>,
}

impl RawState {
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

    pub fn init_table<T: Deref<Target = Self>>(&self, states: &[T]) {
        assert!(self.table.get().is_none());
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

pub struct States<T: Deref<Target = RawState>> {
    def: usize,
    properties: std::collections::HashMap<String, property::Property>,
    states: Vec<T>,
}

impl<T: Deref<Target = RawState> + From<RawState>> States<T> {
    fn new(
        def_state: std::collections::HashMap<property::Property, u8>,
        properties: std::collections::HashMap<String, property::Property>,
    ) -> Self {
        let mut states_raw: Vec<RawState> = Vec::new();
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
            let mut m: std::collections::HashMap<property::Property, u8> =
                std::collections::HashMap::new();
            for e in list2 {
                m.insert(e.0, e.1);
            }
            let state = RawState {
                id: 0,
                entries: todo!(),
                table: todo!(),
            };
        }

        states_raw.iter_mut().enumerate().for_each(|e| e.1.id = e.0);
        let states: Vec<T> = states_raw.into_iter().map(T::from).collect();
        Self {
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
            states,
        }
    }
}

impl<T: Deref<Target = RawState>> States<T> {
    pub fn states(&self) -> &[T] {
        &self.states
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
}
