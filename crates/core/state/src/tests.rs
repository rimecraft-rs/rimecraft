use crate::{
    StatesMut,
    property::{
        BoolProperty, IntProperty,
        data::{BoolData, IntData},
    },
};

static INT_PROPERTY: IntProperty<'static> = IntProperty::new("int_property", IntData(1..=3));
static BOOL_PROPERTY: BoolProperty<'static> = BoolProperty::new("bool_property", BoolData);

#[test]
fn states_create() {
    let mut states = StatesMut::new(|_| ());
    states.add(&INT_PROPERTY).unwrap();
    states.add(&BOOL_PROPERTY).unwrap();
    let states = states.freeze();
    assert_eq!(states.len(), 6);
    let default_state = states.default_state();

    assert_eq!(default_state.get(&INT_PROPERTY), Some(1));
    assert_eq!(default_state.get(&BOOL_PROPERTY), Some(false));
}

#[test]
fn with_cycle() {
    let mut states = StatesMut::new(|_| ());
    states.add(&INT_PROPERTY).unwrap();
    states.add(&BOOL_PROPERTY).unwrap();
    let states = states.freeze();

    let state = states.default_state();
    let state = state.with(&INT_PROPERTY, 2).unwrap();
    let state = state.with(&BOOL_PROPERTY, true).unwrap();
    assert_eq!(state.get(&INT_PROPERTY), Some(2));
    assert_eq!(state.get(&BOOL_PROPERTY), Some(true));
    let state = state.cycle(&BOOL_PROPERTY).unwrap();
    assert_eq!(state.get(&BOOL_PROPERTY), Some(false));
}
