//! Criterion is what you need to do to obtain advancements.

use remap::{remap, remap_method};

#[remap(yarn = "Criterion")]
pub trait Criterion {
    type Tracker;
    type Condition;

    #[remap_method(yarn = "beginTrackingCondition")]
    fn track_one(&mut self, tracker: Self::Tracker, condition: Self::Condition);
    #[remap_method(yarn = "endTrackingCondition")]
    fn untrack_one(&mut self, tracker: Self::Tracker, condition: Self::Condition);
    #[remap_method(yarn = "endTracking")]
    fn untrack_all(&mut self, tracker: Self::Tracker);
}
