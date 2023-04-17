use super::NbtElement;
use crate::util::string_escape;

pub trait NbtElementVisitor {
    fn visit(&mut self, element: &NbtElement);
}

#[derive(Default)]
pub struct StringNbtWriter {
    result: String,
}

impl StringNbtWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, element: &NbtElement) -> &str {
        element.accept(self);
        &self.result
    }
}

impl NbtElementVisitor for StringNbtWriter {
    fn visit(&mut self, element: &NbtElement) {
        match element {
            NbtElement::String(value) => self.result.push_str(&string_escape(&value)),
            NbtElement::U8(value) => {
                self.result.push_str(&value.to_string());
                self.result.push('b')
            }
            NbtElement::I16(value) => {
                self.result.push_str(&value.to_string());
                self.result.push('s')
            }
            NbtElement::I32(value) => {
                self.result.push_str(&value.to_string());
            }
            NbtElement::I64(value) => {
                self.result.push_str(&value.to_string());
                self.result.push('L')
            }
            NbtElement::F32(value) => {
                self.result.push_str(&value.to_string());
                self.result.push('f')
            }
            NbtElement::F64(value) => {
                self.result.push_str(&value.to_string());
                self.result.push('d')
            }
            NbtElement::U8Vec(value) => {
                self.result.push_str("[B;");
                for i in 0..value.len() {
                    if i != 0 {
                        self.result.push(',');
                    }
                    self.result.push_str(&value.get(i).unwrap().to_string());
                    self.result.push('B');
                }
                self.result.push(']');
            }
            NbtElement::I32Vec(value) => {
                self.result.push_str("[I;");
                for i in 0..value.len() {
                    if i != 0 {
                        self.result.push(',');
                    }
                    self.result.push_str(&value.get(i).unwrap().to_string());
                }
                self.result.push(']');
            }
            NbtElement::I64Vec(value) => {
                self.result.push_str("[L;");
                for i in 0..value.len() {
                    if i != 0 {
                        self.result.push(',');
                    }
                    self.result.push_str(&value.get(i).unwrap().to_string());
                    self.result.push('L');
                }
                self.result.push(']');
            }
            NbtElement::List(value, _) => {
                self.result.push('[');
                for i in 0..value.len() {
                    if i != 0 {
                        self.result.push(',');
                    }
                    self.result
                        .push_str(StringNbtWriter::new().apply(value.get(i).unwrap()));
                }
                self.result.push(']');
            }
            NbtElement::Compound(value) => {
                self.result.push('{');
                let mut list: Vec<&str> = value.keys().map(|d| d.as_str()).collect();
                list.sort();
                for string in list {
                    if self.result.len() != 1 {
                        self.result.push(',');
                    }
                    self.result.push_str(&{
                        if {
                            let mut b = true;
                            for c in string.chars() {
                                if !(c <= 'Z'
                                    || c >= 'A'
                                    || c <= 'z'
                                    || c >= 'a'
                                    || c <= '9'
                                    || c >= '0'
                                    || c == '.'
                                    || c == '_'
                                    || c == '+'
                                    || c == '-')
                                {
                                    b = false
                                }
                            }
                            b
                        } {
                            string.to_string()
                        } else {
                            string_escape(string)
                        }
                    });
                    self.result.push(':');
                    self.result
                        .push_str(&StringNbtWriter::new().apply(value.get(string).unwrap()));
                }
                self.result.push('}');
            }
            NbtElement::End => self.result.push_str("END"),
        }
    }
}
