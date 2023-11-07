use std::{borrow::Cow, collections::HashMap, fmt::Display, sync::OnceLock};

macro_rules! formatting {
    ($( ($i:ty, $n:expr, $c:expr, $m:expr, $ci:expr, $cv:expr) ),*) => {
        /// A type holding formattings.
        ///
        /// There are two types of formattings, color and modifier. Color formattings
        /// are associated with a specific color, while modifier formattings modify the
        /// style, such as by bolding the text. [`Self::RESET`] is a special formatting
        /// and is not classified as either of these two.
        #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
        pub enum Formatting {
            $(
                $i
            )*
        }

        impl Formatting {
            const CODE_PREFIX: char = '§';

            fn name_raw(&self) -> &'static str {
                match self {
                    $(
                        Formatting::$i => $n,
                    )*
                }
            }

            /// Returns the code to be placed after the [`Self::CODE_PREFIX`]
            /// when this format is converted to a string.
            #[inline]
            pub fn code(&self) -> char {
                match self {
                    $(
                        Formatting::$i => $c,
                    )*
                }
            }

            /// Returns the color index for the formatting.06
            #[inline]
            pub fn color_index(&self) -> Option<u8> {
                match self {
                    $(
                        Formatting::$i => $ci,
                    )*
                }
            }

            /// Returns `true` if the formatting is a modifier.
            #[inline]
            pub fn is_modifier(&self) -> bool {
                match self {
                    $(
                        Formatting::$i => $m,
                    )*
                }
            }

            /// Returns the color of the formatted text, or
            /// `None` if the formatting has no associated color.
            #[inline]
            pub fn color_value(&self) -> Option<u32> {
                match self {
                    $(
                        Formatting::$i => $cv,
                    )*
                }
            }
        }

        impl std::fmt::Display for Formatting {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    $(
                        Formatting::$i => {
                            f.write_char(Self::CODE_PREFIX)?;
                            f.write_char(self.code)?;
                        },
                    )*
                }

                Ok(())
            }
        }
    };
}

formatting!{
    Black -> ("BLACK", '0', false, 0, Some(0)),
}
