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
    io::{self, ErrorKind, Read, Write},
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

#[derive(Clone, PartialEq, Default)]
pub struct NbtCompound {
    pub(self) entries: HashMap<String, NbtElement>,
}

impl NbtCompound {
    pub fn new() -> Self {
        Self::default()
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
    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        match &self {
            NbtElement::String(string) => {
                if let Err(err) = output.write(string.as_bytes()) {
                    error!("{err}");
                    output.write("".as_bytes())?;
                };
            }
            NbtElement::U8(byte) => {
                output.write(&[*byte])?;
            }
            NbtElement::I16(value) => {
                output.write(&value.to_be_bytes())?;
            }
            NbtElement::I32(value) => {
                output.write(&value.to_be_bytes())?;
            }
            NbtElement::I64(value) => {
                output.write(&value.to_be_bytes())?;
            }
            NbtElement::F32(value) => {
                output.write(&value.to_be_bytes())?;
            }
            NbtElement::F64(value) => {
                output.write(&value.to_be_bytes())?;
            }
            NbtElement::U8Vec(value) => {
                output.write(&(value.len() as i32).to_be_bytes())?;
                output.write(&value)?;
            }
            NbtElement::I32Vec(value) => {
                output.write(&(value.len() as i32).to_be_bytes())?;
                for i in value {
                    output.write(&i.to_be_bytes())?;
                }
            }
            NbtElement::I64Vec(value) => {
                output.write(&(value.len() as i32).to_be_bytes())?;
                for i in value {
                    output.write(&i.to_be_bytes())?;
                }
            }
            NbtElement::List(_, _) => todo!(),
            NbtElement::Compound(_) => todo!(),
            NbtElement::End => todo!(),
        }
        Ok(())
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
            NbtElement::I16(_) => 10,
            NbtElement::I32(_) => 12,
            NbtElement::I64(_) => 16,
            NbtElement::F32(_) => 12,
            NbtElement::F64(_) => 16,
            NbtElement::U8Vec(value) => 24 + value.len(),
            NbtElement::I32Vec(value) => 24 + 4 * value.len(),
            NbtElement::I64Vec(value) => 24 + 8 * value.len(),
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
            NbtElement::U8(value) => visitor.visit_u8(*value),
            NbtElement::I16(value) => visitor.visit_i16(*value),
            NbtElement::I32(value) => visitor.visit_i32(*value),
            NbtElement::I64(value) => visitor.visit_i64(*value),
            NbtElement::F32(value) => visitor.visit_f32(*value),
            NbtElement::F64(value) => visitor.visit_f64(*value),
            NbtElement::U8Vec(value) => visitor.visit_u8_arr(value.to_vec()),
            NbtElement::I32Vec(value) => visitor.visit_i32_arr(value.to_vec()),
            NbtElement::I64Vec(value) => visitor.visit_i64_arr(value.to_vec()),
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
    pub fn from_id(id: u8) -> Option<NbtType> {
        match id {
            0 => Some(Self::End),
            1 => Some(Self::U8),
            2 => Some(Self::I16),
            3 => Some(Self::I32),
            4 => Some(Self::I64),
            5 => Some(Self::F32),
            6 => Some(Self::F64),
            7 => Some(Self::U8Vec),
            8 => Some(Self::String),
            9 => Some(Self::List),
            10 => Some(Self::Compound),
            11 => Some(Self::I32Vec),
            12 => Some(Self::I64Vec),
            _ => None,
        }
    }

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
                    input.read_exact(&mut arr)?;
                    match arr.first() {
                        Some(e) => *e,
                        None => return Err(io::Error::new(ErrorKind::Other, "Can't read u8")),
                    }
                }))
            }
            NbtType::I16 => {
                tracker.add(10);
                Ok(NbtElement::I16({
                    let mut arr = [0; 2];
                    input.read_exact(&mut arr)?;
                    i16::from_be_bytes(arr)
                }))
            }
            NbtType::I32 => {
                tracker.add(12);
                Ok(NbtElement::I32({
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                }))
            }
            NbtType::I64 => {
                tracker.add(16);
                Ok(NbtElement::I64({
                    let mut arr = [0; 8];
                    input.read_exact(&mut arr)?;
                    i64::from_be_bytes(arr)
                }))
            }
            NbtType::F32 => {
                tracker.add(12);
                Ok(NbtElement::F32({
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    f32::from_be_bytes(arr)
                }))
            }
            NbtType::F64 => {
                tracker.add(16);
                Ok(NbtElement::F64({
                    let mut arr = [0; 8];
                    input.read_exact(&mut arr)?;
                    f64::from_be_bytes(arr)
                }))
            }
            NbtType::U8Vec => {
                tracker.add(24);
                if let Ok(j) = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                }
                .try_into()
                {
                    tracker.add(j);
                    let mut bs: Vec<u8> = Vec::with_capacity(j);
                    for _ in 0..j {
                        let mut arr = [0; 1];
                        input.read_exact(&mut arr)?;
                        bs.push(match arr.first() {
                            Some(e) => *e,
                            None => {
                                return Err(io::Error::new(ErrorKind::Other, "Can't read u8 vec"))
                            }
                        })
                    }
                    Ok(NbtElement::U8Vec(bs))
                } else {
                    Err(io::Error::new(ErrorKind::Other, "Can't read u8 vec"))
                }
            }
            NbtType::I32Vec => {
                tracker.add(24);
                if let Ok(j) = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                }
                .try_into()
                {
                    tracker.add(4 * j);
                    let mut is: Vec<i32> = Vec::with_capacity(j);
                    for _ in 0..j {
                        let mut arr = [0; 4];
                        input.read_exact(&mut arr)?;
                        is.push(i32::from_be_bytes(arr));
                    }
                    Ok(NbtElement::I32Vec(is))
                } else {
                    Err(io::Error::new(ErrorKind::Other, "Can't read i32 vec"))
                }
            }
            NbtType::I64Vec => {
                tracker.add(24);
                if let Ok(j) = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                }
                .try_into()
                {
                    tracker.add(8 * j);
                    let mut ls: Vec<i64> = Vec::with_capacity(j);
                    for _ in 0..j {
                        let mut arr = [0; 8];
                        input.read_exact(&mut arr)?;
                        ls.push(i64::from_be_bytes(arr));
                    }
                    Ok(NbtElement::I64Vec(ls))
                } else {
                    Err(io::Error::new(ErrorKind::Other, "Can't read i64 vec"))
                }
            }
            NbtType::List => {
                tracker.add(37);
                if i > 512 {
                    return Err(io::Error::new(
                        ErrorKind::Other,
                        "Tried to read NBT tag with too high complexity, depth > 512",
                    ));
                }
                let b = {
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                    *arr.get(0).unwrap()
                };
                let j = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                };
                if b == 0 && j > 0 {
                    return Err(io::Error::new(ErrorKind::Other, "Missing type on ListTag"));
                }
                tracker.add((4 * j) as usize);
                let nbt_type = NbtType::from_id(b).unwrap();
                let mut list: Vec<NbtElement> = Vec::with_capacity(j as usize);
                for _ in 0..j {
                    list.push(nbt_type.read(input, i + 1, tracker)?);
                }
                Ok(NbtElement::List(list, b))
            }
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
                input.read_exact(&mut arr)?;
                *arr.first().unwrap()
            })),
            NbtType::I16 => Ok(scanner.visit_i16({
                let mut arr = [0; 2];
                input.read_exact(&mut arr)?;
                i16::from_be_bytes(arr)
            })),
            NbtType::I32 => Ok(scanner.visit_i32({
                let mut arr = [0; 4];
                input.read_exact(&mut arr)?;
                i32::from_be_bytes(arr)
            })),
            NbtType::I64 => Ok(scanner.visit_i64({
                let mut arr = [0; 8];
                input.read_exact(&mut arr)?;
                i64::from_be_bytes(arr)
            })),
            NbtType::F32 => Ok(scanner.visit_f32({
                let mut arr = [0; 4];
                input.read_exact(&mut arr)?;
                f32::from_be_bytes(arr)
            })),
            NbtType::F64 => Ok(scanner.visit_f64({
                let mut arr = [0; 8];
                input.read_exact(&mut arr)?;
                f64::from_be_bytes(arr)
            })),
            NbtType::U8Vec => Ok(scanner.visit_u8_arr({
                let mut bs: Vec<u8> = Vec::with_capacity({
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr) as usize
                });
                input.read_exact(&mut bs)?;
                bs
            })),
            NbtType::I32Vec => Ok(scanner.visit_i32_arr({
                let mut bs: Vec<i32> = Vec::with_capacity({
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr) as usize
                });
                for _ in 0..bs.capacity() {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    bs.push(i32::from_be_bytes(arr));
                }
                bs
            })),
            NbtType::I64Vec => Ok(scanner.visit_i64_arr({
                let mut bs: Vec<i64> = Vec::with_capacity({
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr) as usize
                });
                for _ in 0..bs.capacity() {
                    let mut arr = [0; 8];
                    input.read_exact(&mut arr)?;
                    bs.push(i64::from_be_bytes(arr));
                }
                bs
            })),
            NbtType::List => {
                let nbt_type = NbtType::from_id({
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                    *arr.get(0).unwrap()
                })
                .unwrap();
                let i = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr) as usize
                };
                match scanner.visit_list_meta(nbt_type, i) {
                    ScannerResult::Break => {
                        nbt_type.skip_counted(input, i)?;
                        Ok(scanner.end_nested())
                    }
                    ScannerResult::Halt => Ok(ScannerResult::Halt),
                    _ => {
                        let mut j = 0;
                        loop {
                            if j < i {
                                match scanner.start_list_item(nbt_type, j) {
                                    scanner::ScannerNestedResult::Skip => {
                                        nbt_type.skip(input)?;
                                        j += 1;
                                        continue;
                                    }
                                    scanner::ScannerNestedResult::Break => {
                                        nbt_type.skip(input)?;
                                    }
                                    scanner::ScannerNestedResult::Halt => {
                                        return Ok(ScannerResult::Halt)
                                    }
                                    _ => match nbt_type.do_accept(input, scanner)? {
                                        ScannerResult::Break => (),
                                        ScannerResult::Halt => return Ok(ScannerResult::Halt),
                                        _ => {
                                            j += 1;
                                            continue;
                                        }
                                    },
                                }
                            }

                            let k = i - 1 - j;
                            if k > 0 {
                                nbt_type.skip_counted(input, k)?;
                            }
                            return Ok(scanner.end_nested());
                        }
                    }
                }
            }
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn accept(&self, input: &mut impl Read, visitor: &mut impl NbtScanner) -> io::Result<()> {
        match visitor.start(*self) {
            ScannerResult::Continue => self.accept(input, visitor),
            ScannerResult::Break => Ok(()),
            ScannerResult::Halt => self.skip(input),
        }
    }

    pub fn is_immutable(&self) -> bool {
        matches!(
            self,
            NbtType::String
                | NbtType::U8
                | NbtType::I16
                | NbtType::I32
                | NbtType::I64
                | NbtType::F32
                | NbtType::F64
        )
    }

    pub fn get_crash_report_name(&self) -> &str {
        match self {
            NbtType::String => "STRING",
            NbtType::U8 => "BYTE",
            NbtType::I16 => "SHORT",
            NbtType::I32 => "INT",
            NbtType::I64 => "LONG",
            NbtType::F32 => "FLOAT",
            NbtType::F64 => "DOUBLE",
            NbtType::U8Vec => "BYTE[]",
            NbtType::I32Vec => "INT[]",
            NbtType::I64Vec => "LONG[]",
            NbtType::List => "LIST",
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn get_command_feedback_name(&self) -> &str {
        match self {
            NbtType::String => "STAG_String",
            NbtType::U8 => "TAG_Byte",
            NbtType::I16 => "TAG_Short",
            NbtType::I32 => "TAG_Int",
            NbtType::I64 => "TAG_Long",
            NbtType::F32 => "TAG_Float",
            NbtType::F64 => "TAG_Double",
            NbtType::U8Vec => "TAG_Byte_Array",
            NbtType::I32Vec => "TAG_Int_Array",
            NbtType::I64Vec => "TAG_Long_Array",
            NbtType::List => "TAG_List",
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
        }
    }

    pub fn skip(&self, input: &mut impl Read) -> io::Result<()> {
        if let Some(size) = self.get_size_in_bytes() {
            for _ in 0..size {
                let mut arr = [0; 1];
                input.read_exact(&mut arr)?;
            }
            return Ok(());
        }

        match self {
            NbtType::String => {
                util::read_unsigned_short(input)?;
                Ok(())
            }
            NbtType::U8Vec => {
                let mut arr = [0; 4];
                input.read_exact(&mut arr)?;
                for _ in 0..i32::from_be_bytes(arr) {
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                }
                Ok(())
            }
            NbtType::I32Vec => {
                let mut arr = [0; 4];
                input.read_exact(&mut arr)?;
                for _ in 0..i32::from_be_bytes(arr) * 4 {
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                }
                Ok(())
            }
            NbtType::I64Vec => {
                let mut arr = [0; 4];
                input.read_exact(&mut arr)?;
                for _ in 0..i32::from_be_bytes(arr) * 8 {
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                }
                Ok(())
            }
            NbtType::List => {
                let nbt_type = NbtType::from_id({
                    let mut arr = [0; 1];
                    input.read_exact(&mut arr)?;
                    *arr.get(0).unwrap()
                })
                .unwrap();
                let i = {
                    let mut arr = [0; 4];
                    input.read_exact(&mut arr)?;
                    i32::from_be_bytes(arr)
                } as usize;
                nbt_type.skip_counted(input, i)
            }
            NbtType::Compound => todo!(),
            NbtType::End => todo!(),
            _ => Ok(()),
        }
    }

    pub fn skip_counted(&self, input: &mut impl Read, count: usize) -> io::Result<()> {
        if let Some(size) = self.get_size_in_bytes() {
            for _ in 0..(size * count) {
                let mut arr = [0; 1];
                input.read_exact(&mut arr)?;
            }
            return Ok(());
        }

        match self {
            NbtType::String | NbtType::I32Vec | NbtType::I64Vec | NbtType::List => {
                for _ in 0..count {
                    self.skip(input)?;
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn get_size_in_bytes(&self) -> Option<usize> {
        match self {
            NbtType::U8 => Some(1),
            NbtType::I16 => Some(2),
            NbtType::I32 => Some(4),
            NbtType::I64 => Some(8),
            NbtType::F32 => Some(4),
            NbtType::F64 => Some(8),
            _ => None,
        }
    }
}

impl Display for NbtElement {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtElement::String(_value) => (),
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
        Ok(())
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
