//! Minecraft client narration components.

use std::fmt::{Debug, Display};

use rimecraft_text::{ProvideTextTy, Text};

/// A component that can provide narrations for accessibility purposes.
///
/// See: [`NarrationMessageBuilder`]
pub trait Narratable {
    /// Gets the narrations as [`ErasedNarration`]s.
    fn narrations(&self) -> Vec<(NarrationPart, ErasedNarration)>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
/// A component of a [`NarrationMessageBuilder`]. This enum is mostly used for grouping and ordering narrations in a narration message.
pub enum NarrationPart {
    /// The main narration for a narrated element.
    Title,
    /// The position of a narrated element in a container such as a list.
    Position,
    /// A hint for a narrated element, e.g. a button tooltip.
    Hint,
    /// Usage instructions for a narrated element.
    Usage,
}

type Transformer<T> = dyn Fn(Box<dyn Fn(&str)>, T);

/// A narration is a message consisting of a list of string "sentences". The sentences can be iterated using [`Narration::accept`].
///
/// Narrations are attached to [`NarrationPart`]s using [`NarrationMessageBuilder::put`].
pub struct Narration<T> {
    value: T,
    transformer: Box<Transformer<T>>,
}

/// An owned, type-erased representation of a narration.
///
/// This materializes all sentences as owned [`String`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErasedNarration {
    sentences: Vec<String>,
}

impl ErasedNarration {
    /// Creates an empty erased narration.
    pub fn empty() -> Self {
        Self {
            sentences: Vec::new(),
        }
    }

    /// Iterates over all sentences.
    pub fn accept<F>(&self, mut f: F)
    where
        F: FnMut(&str),
    {
        for s in &self.sentences {
            f(s);
        }
    }

    /// Read-only access to the underlying sentences.
    pub fn sentences(&self) -> &[String] {
        &self.sentences
    }
}

impl<T> From<Narration<T>> for ErasedNarration {
    fn from(n: Narration<T>) -> Self {
        // Call the stored transformer directly to avoid requiring the pushed-closure
        // to be 'static (which `Narration::accept` requires).
        let sentences_arc = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sentences_clone = sentences_arc.clone();
        let transformer = n.transformer;
        let value = n.value;
        (transformer)(
            Box::new(move |s: &str| {
                let mut guard = sentences_clone.lock().unwrap();
                guard.push(s.to_owned());
            }),
            value,
        );

        // At this point the transformer should have been invoked and the
        // closure dropped, so we can unwrap the Arc and take the Vec out.
        let sentences = std::sync::Arc::try_unwrap(sentences_arc)
            .expect("transformer kept an Arc alive")
            .into_inner()
            .expect("mutex poisoned");

        Self { sentences }
    }
}

impl<T> Debug for Narration<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Narration")
            .field("value", &self.value)
            .field("transformer", &"<function>")
            .finish()
    }
}

impl Narration<String> {
    /// Creates a [`Narration`] from a single string sentence.
    pub fn string<T>(value: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            value: value.into(),
            transformer: Box::new(|f, v| f(&v)),
        }
    }

    /// Creates a [`Narration`] from a string slice.
    pub fn str(value: &str) -> Self {
        Self {
            value: value.to_owned(),
            transformer: Box::new(|f, v| f(&v)),
        }
    }
}

impl Narration<()> {
    /// Creates a [`Narration`] from a single [`Text`] sentence.
    pub fn text<Cx>(value: &Text<Cx>) -> Narration<&Text<Cx>>
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        Narration {
            value,
            transformer: Box::new(|f, v| f(&v.content().to_string())),
        }
    }

    /// Creates a [`Narration`] from a list of [`Text`] sentences.
    pub fn texts<Cx>(value: Vec<&Text<Cx>>) -> Narration<Vec<&Text<Cx>>>
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        Narration {
            value,
            transformer: Box::new(|f, v| {
                v.into_iter()
                    .map(|t| t.content().to_string())
                    .for_each(|v| f(&v));
            }),
        }
    }
}

impl<T> Narration<T> {
    /// An empty narration that contains no sentences.
    pub fn empty() -> Narration<()> {
        Narration {
            value: (),
            transformer: Box::new(|_, _| {}),
        }
    }

    /// Iterates all sentences in this [`Narration`] by calling the given function `f` for each sentence.
    pub fn accept<F>(&self, f: F)
    where
        F: Fn(&str) + 'static,
        T: Clone,
    {
        (self.transformer)(Box::new(f), self.value.clone());
    }
}

impl<T> Narration<T> {
    /// Materialize this generic [`Narration<T>`] into an [`ErasedNarration`].
    pub fn erase(self) -> ErasedNarration {
        ErasedNarration::from(self)
    }
}

/// A builder for narration messages.
///
/// Narration messages consist of multiple sections known as [`NarrationPart`]s.
/// Each narration message can contain only one narration per part.
///
/// You can create a *submessage* by calling [`NarrationMessageBuilder::next_message`].
/// Each submessage can have its own set of narrations for the different narration parts.
///
/// # Ordering
///
/// The narrations added to a message will be ordered by their part first, in [`NarrationPart`]'s natural ordering.
/// If there are multiple narrations for a part added through submessages, they will be ordered earliest submessage first.
pub trait NarrationMessageBuilder {
    /// Adds a narration to this builder, replacing any existing narration for that [`NarrationPart`].
    fn put<T>(&mut self, part: NarrationPart, narration: Narration<T>);

    /// Adds a string narration to this builder, replacing any existing narration for that [`NarrationPart`].
    fn put_string(&mut self, part: NarrationPart, text: impl Into<String>) {
        self.put(part, Narration::string(text.into()));
    }

    /// Adds a [`Text`] narration to this builder, replacing any existing narration for that [`NarrationPart`].
    fn put_text<Cx>(&mut self, part: NarrationPart, text: &Text<Cx>)
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        self.put(part, Narration::text::<Cx>(text));
    }

    /// Adds an array of [`Text`] narrations to this builder, replacing any existing narration for that [`NarrationPart`].
    fn put_texts<Cx>(&mut self, part: NarrationPart, texts: Vec<&Text<Cx>>)
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        self.put(part, Narration::texts::<Cx>(texts));
    }

    /// Adds an erased (non-generic) narration to this builder.
    ///
    /// Default implementation converts the `ErasedNarration` into a
    /// `Narration<Vec<String>>` and forwards to the generic `put`.
    fn put_erased(&mut self, part: NarrationPart, narration: ErasedNarration) {
        self.put(
            part,
            Narration {
                value: narration.sentences,
                transformer: Box::new(|f, v: Vec<String>| {
                    v.into_iter().for_each(|s| f(&s));
                }),
            },
        );
    }

    /// Creates a [`NarrationMessageBuilder`] for a submessage, which merges into this message after used.
    ///
    /// Submessages can have their own set of [`Narration`]s for the [`NarrationPart`]s, which are merged with the "parent" message's narrations as described above.
    fn next_message(&mut self) -> impl NarrationMessageBuilder;
}

#[cfg(test)]
mod tests;
