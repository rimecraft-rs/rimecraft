//! Minecraft client narration components.

use std::fmt::{Debug, Display};

use rimecraft_text::{ProvideTextTy, Text};

/// A component that can provide narrations for accessibility purposes.
///
/// See: [`NarrationMessageBuilder`]
pub trait Narratable {
    /// Appends narrations to the given [`NarrationMessageBuilder`].
    fn append_narrations<B>(&self, builder: &mut B)
    where
        B: NarrationMessageBuilder;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn string(value: String) -> Self {
        Self {
            value,
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
    pub fn text<Cx>(value: Text<Cx>) -> Narration<Text<Cx>>
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
    pub fn texts<Cx>(value: Vec<Text<Cx>>) -> Narration<Vec<Text<Cx>>>
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
    fn put_text<Cx>(&mut self, part: NarrationPart, text: Text<Cx>)
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        self.put(part, Narration::text::<Cx>(text));
    }

    /// Adds an array of [`Text`] narrations to this builder, replacing any existing narration for that [`NarrationPart`].
    fn put_texts<Cx>(&mut self, part: NarrationPart, texts: Vec<Text<Cx>>)
    where
        Cx: ProvideTextTy,
        <Cx as ProvideTextTy>::Content: Display,
    {
        self.put(part, Narration::texts::<Cx>(texts));
    }

    /// Creates a [`NarrationMessageBuilder`] for a submessage.
    ///
    /// Submessages can have their own set of [`Narration`]s for the [`NarrationPart`]s, which are merged with the "parent" message's narrations as described above.
    ///
    /// # API Notes
    ///
    /// All returned builder instances are equivalent and refer to the same submessage. If you want to add yet another set of narrations, call this method again on the first submessage builder to obtain a "nested" submessage builder.
    fn next_message(&self) -> Self;
}
