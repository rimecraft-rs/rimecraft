pub mod formatting;
pub mod math;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ColorIndex {
    value: i8,
}

impl ColorIndex {
    #[inline]
    pub fn new(value: Option<u8>) -> Self {
        Self {
            value: value.map(|e| e as i8).unwrap_or(-1),
        }
    }

    #[inline]
    pub fn value(self) -> Option<u8> {
        if self.value == -1 {
            None
        } else {
            Some(self.value as u8)
        }
    }
}
