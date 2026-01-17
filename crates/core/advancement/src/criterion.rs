//! Criterion is what you need to do to obtain advancements.

use remap::{remap, remap_method};

#[remap(yarn = "Criterion", mojmaps = "Criterion")]
pub trait Criterion {
    type Tracker;
    type Condition;

    #[remap_method(yarn = "beginTrackingCondition", mojmaps = "addPlayerListener")]
    fn track_one(&mut self, tracker: Self::Tracker, condition: Self::Condition);
    #[remap_method(yarn = "endTrackingCondition", mojmaps = "removePlayerListener")]
    fn untrack_one(&mut self, tracker: Self::Tracker, condition: Self::Condition);
    #[remap_method(yarn = "endTracking", mojmaps = "removePlayerListeners")]
    fn untrack_all(&mut self, tracker: Self::Tracker);
}
