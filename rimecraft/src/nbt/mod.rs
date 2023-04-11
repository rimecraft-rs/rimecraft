use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Read, Write},
};

use log::error;

use crate::util;

use self::{scanner::NbtScanner, visitor::NbtElementVisitor};

pub mod scanner;
pub mod visitor;

const END_TYPE: u8 = 0;

#[derive(Clone, PartialEq)]
pub struct NbtCompound {
    pub(self) entries: HashMap<String, NbtElement>,
}

impl NbtCompound {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum NbtElement {
    String(String),
    U8(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    U8Vec(Vec<u8>),
    I32Vec(Vec<i32>),
    List(Vec<NbtElement>, u8),
    Compound(NbtCompound),
    End,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NbtType {
    String,
    U8,
    I16,
    I32,
    I64,
    F32,
    F64,
    U8Vec,
    I32Vec,
    List,
    Compound,
    End,
}

impl NbtElement {
    pub fn write(&self, output: &mut impl Write) {
        match &self {
            NbtElement::String(string) => {
                if let Err(err) = output.write(string.as_bytes()) {
                    error!("{err}");
                    output.write("".as_bytes()).unwrap();
                }
            },
            NbtElement::U8(_) => todo!(),
            NbtElement::I16(_) => todo!(),
            NbtElement::I32(_) => todo!(),
            NbtElement::I64(_) => todo!(),
            NbtElement::F32(_) => todo!(),
            NbtElement::F64(_) => todo!(),
            NbtElement::U8Vec(_) => todo!(),
            NbtElement::I32Vec(_) => todo!(),
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
    }

    pub fn get_type(&self) -> u8 {
        todo!()
    }

    pub fn get_size_in_bytes(&self) -> usize {
        todo!()
    }

    pub fn accept(&self, visitor: &mut impl NbtElementVisitor) {
        visitor.visit(self)
    }

    pub fn do_accept(&self, visitor: &mut impl NbtScanner) {
        todo!()
    }

    pub fn accept_scanner(&self, visitor: &mut impl NbtScanner) {
        let result = visitor.start(self.get_nbt_type());
        if result == scanner::ScannerResult::Continue {
            self.do_accept(visitor)
        }
    } 

    pub fn get_nbt_type(&self) -> NbtType {
        todo!()
    }
}

impl NbtType {
    pub fn read(
        &self,
        input: &mut impl Read,
        i: usize,
        tracker: &mut NbtTagSizeTracker,
    ) -> io::Result<NbtElement> {
        match self {
            _ => todo!(),
        }
    }

    pub fn is_immutable(&self) -> bool {
        match self {
            NbtType::String => true,
            _ => false,
        }
    }
    pub fn get_crash_report_name(&self) -> &str {
        match self {
            NbtType::String => "STRING",
            _ => todo!(),
        }
    }
    pub fn get_command_feedback_name(&self) -> &str {
        match self {
            NbtType::String => "STAG_STRING",
            _ => todo!(),
        }
    }

    pub fn skip(&self, input: &mut impl Read) {
        todo!()
    }

    pub fn skip_counted(&self, input: &mut impl Read, count: usize) {
        match self {
            NbtType::String => {
                for _ in 0..count {
                    self.skip(input);
                }
            }
            _ => todo!(),
        }
    }
}

impl Display for NbtElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Default)]
pub struct NbtTagSizeTracker {
    max_bytes: usize,
    allocated_bytes: usize,
}

impl NbtTagSizeTracker {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            allocated_bytes: usize::default(),
        }
    }

    pub fn add(&mut self, bytes: usize) {
        if self.max_bytes == 0 {
            return;
        }
        self.allocated_bytes += bytes;
        if self.allocated_bytes > self.max_bytes {
            self.allocated_bytes = self.max_bytes
        }
    }

    pub fn get_allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
}

pub mod string {
    use crate::util;

    use super::*;

    pub struct Type;

    pub fn skip(input: &mut impl Read) {
        if let Ok(u) = util::read_unsigned_short(input) {
            for _ in 0..u {
                if input.read(&mut [0; 1]).is_err() {
                    return;
                }
            }
        }
    }
}
