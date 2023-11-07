macro_rules! formatting {
    ($( $i:ident => $n:expr, $c:expr, $m:expr, $ci:expr, $cv:expr ),+) => {
        /// A type holding formattings.
        ///
        /// There are two types of formattings, color and modifier. Color formattings
        /// are associated with a specific color, while modifier formattings modify the
        /// style, such as by bolding the text. [`Self::RESET`] is a special formatting
        /// and is not classified as either of these two.
        #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
        pub enum Formatting {
            $(
                $i,
            )*
        }

        impl Formatting {
            const CODE_PREFIX: char = '§';

            #[inline]
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
    };
}

formatting! {
    Black => "BLACK", '0', false, Some(0), Some(0),
    DarkBlue => "BLACK", '0', false, Some(0), Some(0)
}
