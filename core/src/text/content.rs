use super::{
    visit::{StyledVisit, Visit},
    Style,
};

pub trait Content: Visit<()> + StyledVisit<Style> {}
