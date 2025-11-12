use std::collections::BTreeMap;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PartIndex(NarrationPart, usize);

struct TestNarrator {
    narrations: BTreeMap<PartIndex, ErasedNarration>,
}

impl TestNarrator {
    fn new() -> Self {
        Self {
            narrations: BTreeMap::new(),
        }
    }
}

impl TestNarrator {
    fn builder(&mut self) -> TestNarrationMessageBuilder<'_> {
        TestNarrationMessageBuilder::new(self)
    }

    fn narrate(&self) -> String {
        let mut result = String::new();
        for narration in self.narrations.values() {
            narration.accept(|s| {
                if !result.is_empty() {
                    result.push('.');
                    result.push(' ');
                }
                result.push_str(s);
            });
        }
        result
    }
}

struct TestNarrationMessageBuilder<'a> {
    narrations: &'a mut BTreeMap<PartIndex, ErasedNarration>,
    depth: usize,
}

impl PartialOrd for PartIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PartIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).then(self.1.cmp(&other.1))
    }
}

impl<'a> TestNarrationMessageBuilder<'a> {
    fn new(narrator: &'a mut TestNarrator) -> Self {
        Self {
            narrations: &mut narrator.narrations,
            depth: 0,
        }
    }
}

impl NarrationMessageBuilder for TestNarrationMessageBuilder<'_> {
    fn put<T>(&mut self, part: NarrationPart, narration: Narration<T>) {
        let erased: ErasedNarration = narration.into();
        self.narrations.insert(PartIndex(part, self.depth), erased);
    }

    fn next_message(&mut self) -> impl NarrationMessageBuilder {
        self.depth += 1;
        TestNarrationMessageBuilder {
            narrations: self.narrations,
            depth: self.depth,
        }
    }
}

#[test]
fn narration() {
    let mut narrator = TestNarrator::new();
    let mut builder = narrator.builder();

    builder.put_erased(
        NarrationPart::Title,
        Narration::string("Main message title").erase(),
    );
    builder.put(NarrationPart::Hint, Narration::string("Main message hint"));

    {
        let mut sub_builder = builder.next_message();
        sub_builder.put_string(NarrationPart::Title, "Submessage title");
        sub_builder.put_string(NarrationPart::Position, "Submessage position");
        sub_builder.put_string(NarrationPart::Usage, "Submessage usage");
        sub_builder.put_string(NarrationPart::Title, "Submessage title (overridden)");
    }

    let narration = narrator.narrate();
    let collected = narration.split(". ").collect::<Vec<_>>();

    // Ordering: PART [TITLE > POSITION > HINT > USAGE] > DEPTH
    assert_eq!(
        collected,
        vec![
            "Main message title",
            "Submessage title (overridden)",
            "Submessage position",
            "Main message hint",
            "Submessage usage",
        ]
    );
}
