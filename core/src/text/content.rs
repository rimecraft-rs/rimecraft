use super::visit::{StyledVisit, Visit};

pub trait Content: Visit + StyledVisit {}
