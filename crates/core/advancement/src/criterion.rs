//! Criterion is what you need to do to obtain advancements.

use remap::remap;

#[remap(yarn = "Criterion")]
pub trait Criterion {
    type Tracker;
    type Condition;

    fn track_one(tracker: Self::Tracker, condition: Self::Condition);
    fn untrack_one(tracker: Self::Tracker, condition: Self::Condition);
    fn untrack_all(tracker: Self::Tracker);
}
