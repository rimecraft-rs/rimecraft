pub mod scanner;
pub mod visitor;

use self::{
    scanner::{NbtScanner, ScannerResult},
    visitor::NbtElementVisitor,
};
use crate::util;
use log::error;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Read, Write},
};

const END_TYPE: u8 = 0;
const U8_TYPE: u8 = 1;
const I16_TYPE: u8 = 2;
const I32_TYPE: u8 = 3;
const I64_TYPE: u8 = 4;
const F32_TYPE: u8 = 5;
const F64_TYPE: u8 = 6;
const U8_VEC_TYPE: u8 = 7;
const STRING_TYPE: u8 = 8;
const LIST_TYPE: u8 = 9;
const COMPOUND_TYPE: u8 = 10;
const I32_VEC_TYPE: u8 = 11;
const I64_VEC_TYPE: u8 = 12;

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

    pub fn get_keys(&self) -> Vec<&str> {
        self.entries.keys().map(|f| f.as_str()).collect()
    }

    pub fn get_size(&self) -> usize {
        self.entries.len()
    }

    pub fn put(&mut self, key: String, element: NbtElement) -> Option<NbtElement> {
        self.entries.insert(key, element)
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
    I64Vec(Vec<i64>),
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
    I64Vec,
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
            }
            NbtElement::U8(byte) => {
                let _ = output.write(&mut [*byte]);
            }
            NbtElement::I16(_) => todo!(),
            NbtElement::I32(_) => todo!(),
            NbtElement::I64(_) => todo!(),
            NbtElement::F32(_) => todo!(),
            NbtElement::F64(_) => todo!(),
            NbtElement::U8Vec(_) => todo!(),
            NbtElement::I32Vec(_) => todo!(),
            NbtElement::I64Vec(_) => todo!(),
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
    }

    pub fn get_type(&self) -> u8 {
        match &self {
            NbtElement::String(_) => STRING_TYPE,
            NbtElement::U8(_) => U8_TYPE,
            NbtElement::I16(_) => I16_TYPE,
            NbtElement::I32(_) => I32_TYPE,
            NbtElement::I64(_) => I64_TYPE,
            NbtElement::F32(_) => F32_TYPE,
            NbtElement::F64(_) => F64_TYPE,
            NbtElement::U8Vec(_) => U8_VEC_TYPE,
            NbtElement::I32Vec(_) => I32_VEC_TYPE,
            NbtElement::I64Vec(_) => I64_VEC_TYPE,
            NbtElement::List(_, _) => LIST_TYPE,
            NbtElement::Compound(_) => COMPOUND_TYPE,
            NbtElement::End => END_TYPE,
        }
    }

    pub fn get_size_in_bytes(&self) -> usize {
        match self {
            NbtElement::String(value) => 36 + 2 * value.len(),
            NbtElement::U8(_) => 9,
            NbtElement::I16(_) => todo!(),
            NbtElement::I32(_) => todo!(),
            NbtElement::I64(_) => todo!(),
            NbtElement::F32(_) => todo!(),
            NbtElement::F64(_) => todo!(),
            NbtElement::U8Vec(_) => todo!(),
            NbtElement::I32Vec(_) => todo!(),
            NbtElement::I64Vec(_) => todo!(),
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
    }

    pub fn accept(&self, visitor: &mut impl NbtElementVisitor) {
        visitor.visit(self)
    }

    pub fn do_accept(&self, visitor: &mut impl NbtScanner) -> ScannerResult {
        match self {
            NbtElement::String(value) => visitor.visit_string(value),
            NbtElement::U8(_) => todo!(),
            NbtElement::I16(_) => todo!(),
            NbtElement::I32(_) => todo!(),
            NbtElement::I64(_) => todo!(),
            NbtElement::F32(_) => todo!(),
            NbtElement::F64(_) => todo!(),
            NbtElement::U8Vec(_) => todo!(),
            NbtElement::I32Vec(_) => todo!(),
            NbtElement::I64Vec(_) => todo!(),
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
    }

    pub fn accept_scanner(&self, visitor: &mut impl NbtScanner) {
        let result = visitor.start(self.get_nbt_type());
        if result == ScannerResult::Continue {
            self.do_accept(visitor);
        }
    }

    pub fn get_nbt_type(&self) -> NbtType {
        match self {
            NbtElement::String(_) => NbtType::String,
            NbtElement::U8(_) => NbtType::U8,
            NbtElement::I16(_) => NbtType::I16,
            NbtElement::I32(_) => NbtType::I32,
            NbtElement::I64(_) => NbtType::I64,
            NbtElement::F32(_) => NbtType::F32,
            NbtElement::F64(_) => NbtType::F64,
            NbtElement::U8Vec(_) => NbtType::U8Vec,
            NbtElement::I32Vec(_) => NbtType::I32Vec,
            NbtElement::I64Vec(_) => NbtType::I64Vec,
            NbtElement::List(_, _) => NbtType::List,
            NbtElement::Compound(_) => NbtType::Compound,
            NbtElement::End => NbtType::End,
        }
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
            NbtType::String => {
                tracker.add(36);
                let string = {
                    let mut s = String::new();
                    input.read_to_string(&mut s)?;
                    s
                };
                tracker.add(2 * string.len());
                Ok(NbtElement::String(string))
            }
            NbtType::U8 => {
                tracker.add(9);
                Ok(NbtElement::U8({
                    let mut arr = [0; 1];
                    input.read(&mut arr)?;
                    *arr.get(0).unwrap()
                }))
            }
            NbtType::I16 => todo!(),
            NbtType::I32 => todo!(),
            NbtType::I64 => todo!(),
            NbtType::F32 => todo!(),
            NbtType::F64 => todo!(),
            NbtType::U8Vec => todo!(),
            NbtType::I32Vec => todo!(),
            NbtType::I64Vec => todo!(),
            NbtType::List => todo!(),
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn do_accept(
        &self,
        input: &mut impl Read,
        scanner: &mut impl NbtScanner,
    ) -> io::Result<ScannerResult> {
        match self {
            NbtType::String => Ok(scanner.visit_string(&{
                let mut s = String::new();
                input.read_to_string(&mut s)?;
                s
            })),
            NbtType::U8 => Ok(scanner.visit_u8({
                let mut arr = [0; 1];
                input.read(&mut arr)?;
                *arr.get(0).unwrap()
            })),
            NbtType::I16 => todo!(),
            NbtType::I32 => todo!(),
            NbtType::I64 => todo!(),
            NbtType::F32 => todo!(),
            NbtType::F64 => todo!(),
            NbtType::U8Vec => todo!(),
            NbtType::I32Vec => todo!(),
            NbtType::I64Vec => todo!(),
            NbtType::List => todo!(),
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn accept(&self, input: &mut impl Read, visitor: &mut impl NbtScanner) {
        match visitor.start(*self) {
            ScannerResult::Continue => self.accept(input, visitor),
            ScannerResult::Break => (),
            ScannerResult::Halt => self.skip(input),
        }
    }

    pub fn is_immutable(&self) -> bool {
        match self {
            NbtType::String | NbtType::U8 => true,
            _ => false,
        }
    }
    pub fn get_crash_report_name(&self) -> &str {
        match self {
            NbtType::String => "STRING",
            NbtType::U8 => "BYTE",
            NbtType::I16 => todo!(),
            NbtType::I32 => todo!(),
            NbtType::I64 => todo!(),
            NbtType::F32 => todo!(),
            NbtType::F64 => todo!(),
            NbtType::U8Vec => todo!(),
            NbtType::I32Vec => todo!(),
            NbtType::I64Vec => todo!(),
            NbtType::List => todo!(),
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }
    pub fn get_command_feedback_name(&self) -> &str {
        match self {
            NbtType::String => "STAG_String",
            NbtType::U8 => "TAG_Byte",
            NbtType::I16 => todo!(),
            NbtType::I32 => todo!(),
            NbtType::I64 => todo!(),
            NbtType::F32 => todo!(),
            NbtType::F64 => todo!(),
            NbtType::U8Vec => todo!(),
            NbtType::I32Vec => todo!(),
            NbtType::I64Vec => todo!(),
            NbtType::List => todo!(),
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn skip(&self, input: &mut impl Read) {
        if let Some(size) = self.get_size_in_bytes() {
            for _ in 0..size {
                let mut arr = [0; 1];
                if input.read(&mut arr).is_err() {
                    return;
                }
            }
            return;
        }

        match self {
            NbtType::String => {
                let _r = util::read_unsigned_short(input);
            }
            NbtType::I16 => todo!(),
            NbtType::I32 => todo!(),
            NbtType::I64 => todo!(),
            NbtType::F32 => todo!(),
            NbtType::F64 => todo!(),
            NbtType::U8Vec => todo!(),
            NbtType::I32Vec => todo!(),
            NbtType::I64Vec => todo!(),
            NbtType::List => todo!(),
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
            _ => (),
        }
    }

    pub fn skip_counted(&self, input: &mut impl Read, count: usize) {
        if let Some(size) = self.get_size_in_bytes() {
            for _ in 0..(size * count) {
                let mut arr = [0; 1];
                if input.read(&mut arr).is_err() {
                    return;
                }
            }
            return;
        }

        match self {
            NbtType::String => {
                for _ in 0..count {
                    self.skip(input);
                }
            }
            _ => (),
        }
    }

    pub fn get_size_in_bytes(&self) -> Option<usize> {
        match self {
            NbtType::U8 => Some(1),
            _ => None,
        }
    }
}

impl Display for NbtElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtElement::String(value) => (),
            NbtElement::U8(_) => todo!(),
            NbtElement::I16(_) => todo!(),
            NbtElement::I32(_) => todo!(),
            NbtElement::I64(_) => todo!(),
            NbtElement::F32(_) => todo!(),
            NbtElement::F64(_) => todo!(),
            NbtElement::U8Vec(_) => todo!(),
            NbtElement::I32Vec(_) => todo!(),
            NbtElement::I64Vec(_) => todo!(),
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
        return Ok(());
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
