use std::collections::VecDeque;

use crate::nbt::END_TYPE;

use super::{NbtCompound, NbtElement, NbtType};

pub trait NbtScanner {
    fn visit_end(&mut self) -> ScannerResult;
    fn visit_string(&mut self, value: &str) -> ScannerResult;
    fn visit_u8(&mut self, value: u8) -> ScannerResult;
    fn visit_i16(&mut self, value: i16) -> ScannerResult;
    fn visit_i32(&mut self, value: i32) -> ScannerResult;
    fn visit_i64(&mut self, value: i64) -> ScannerResult;
    fn visit_f32(&mut self, value: f32) -> ScannerResult;
    fn visit_f64(&mut self, value: f64) -> ScannerResult;
    fn visit_u8_arr(&mut self, value: Vec<u8>) -> ScannerResult;
    fn visit_i32_arr(&mut self, value: Vec<i32>) -> ScannerResult;
    fn visit_i64_arr(&mut self, value: Vec<i64>) -> ScannerResult;
    fn visit_list_meta(&mut self, nbt_type: NbtType, i: usize) -> ScannerResult;
    fn visit_sub_nbt_type(&mut self, nbt_type: NbtType) -> ScannerNestedResult;

    fn start_sub_nbt(&mut self, nbt_type: NbtType, nbt: &str) -> ScannerNestedResult;
    fn start_list_item(&mut self, nbt_type: NbtType, i: usize) -> ScannerNestedResult;
    fn end_nested(&mut self) -> ScannerResult;
    fn start(&mut self, nbt_type: NbtType) -> ScannerResult;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScannerNestedResult {
    Enter,
    Skip,
    Break,
    Halt,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScannerResult {
    Continue,
    Break,
    Halt,
}

#[derive(Default)]
pub struct NbtCollector {
    current_key: String,
    root: Option<NbtElement>,
    stack: VecDeque<Box<dyn Fn(&mut NbtElement, &mut Option<NbtElement>, &str)>>,
}

impl NbtCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_root(&self) -> Option<&NbtElement> {
        match &self.root {
            Some(r) => Some(r),
            None => None,
        }
    }

    pub fn get_root_mut(&mut self) -> Option<&mut NbtElement> {
        match &mut self.root {
            Some(r) => Some(r),
            None => None,
        }
    }

    pub fn get_depth(&self) -> usize {
        self.stack.len()
    }

    fn append(&mut self, nbt: &mut NbtElement) {
        if let Some(a) = self.stack.back() {
            let mut v = self.root.clone();
            a(nbt, &mut v, &self.current_key);
            self.root = v;
        }
    }

    fn push_stack(&mut self, nbt_type: &NbtType) {
        match nbt_type {
            NbtType::List => {
                self.root = Some(NbtElement::List(Vec::new(), super::END_TYPE));
                self.stack.push_back(Box::new(|e, s, _| match s {
                    Some(NbtElement::List(ls, _)) => ls.push(e.to_owned()),
                    _ => (),
                }));
            }
            _ => (),
        }
    }
}

impl NbtScanner for NbtCollector {
    fn visit_end(&mut self) -> ScannerResult {
        self.append(&mut NbtElement::End);
        ScannerResult::Continue
    }

    fn visit_string(&mut self, value: &str) -> ScannerResult {
        self.append(&mut NbtElement::String(value.to_string()));
        ScannerResult::Continue
    }

    fn visit_u8(&mut self, value: u8) -> ScannerResult {
        self.append(&mut NbtElement::U8(value));
        ScannerResult::Continue
    }

    fn visit_i16(&mut self, value: i16) -> ScannerResult {
        self.append(&mut NbtElement::I16(value));
        ScannerResult::Continue
    }

    fn visit_i32(&mut self, value: i32) -> ScannerResult {
        self.append(&mut NbtElement::I32(value));
        ScannerResult::Continue
    }

    fn visit_i64(&mut self, value: i64) -> ScannerResult {
        self.append(&mut NbtElement::I64(value));
        ScannerResult::Continue
    }

    fn visit_f32(&mut self, value: f32) -> ScannerResult {
        self.append(&mut NbtElement::F32(value));
        ScannerResult::Continue
    }

    fn visit_f64(&mut self, value: f64) -> ScannerResult {
        self.append(&mut NbtElement::F64(value));
        ScannerResult::Continue
    }

    fn visit_u8_arr(&mut self, value: Vec<u8>) -> ScannerResult {
        self.append(&mut NbtElement::U8Vec(value));
        ScannerResult::Continue
    }

    fn visit_i32_arr(&mut self, value: Vec<i32>) -> ScannerResult {
        self.append(&mut NbtElement::I32Vec(value));
        ScannerResult::Continue
    }

    fn visit_i64_arr(&mut self, value: Vec<i64>) -> ScannerResult {
        self.append(&mut NbtElement::I64Vec(value));
        ScannerResult::Continue
    }

    fn visit_list_meta(&mut self, _nbt_type: NbtType, _i: usize) -> ScannerResult {
        ScannerResult::Continue
    }

    fn visit_sub_nbt_type(&mut self, _nbt_type: NbtType) -> ScannerNestedResult {
        ScannerNestedResult::Enter
    }

    fn start_sub_nbt(&mut self, nbt_type: NbtType, nbt: &str) -> ScannerNestedResult {
        self.current_key = nbt.to_string();
        self.push_stack(&nbt_type);
        ScannerNestedResult::Enter
    }

    fn start_list_item(&mut self, nbt_type: NbtType, _i: usize) -> ScannerNestedResult {
        self.push_stack(&nbt_type);
        ScannerNestedResult::Enter
    }

    fn end_nested(&mut self) -> ScannerResult {
        self.stack.pop_back();
        ScannerResult::Continue
    }

    fn start(&mut self, nbt_type: NbtType) -> ScannerResult {
        match nbt_type {
            NbtType::List => {
                self.root = Some(NbtElement::List(Vec::new(), END_TYPE));
                self.stack.push_back(Box::new(|a, b, _| match b {
                    Some(NbtElement::List(ls, _)) => ls.push(a.to_owned()),
                    _ => (),
                }));
            }
            NbtType::Compound => {
                self.root = Some(NbtElement::Compound(NbtCompound::new()));
                self.stack.push_back(Box::new(|a, b, r| match b {
                    Some(NbtElement::Compound(compound)) => {
                        compound.put(r.to_owned(), a.clone());
                    }
                    _ => (),
                }));
            }
            _ => self.stack.push_back(Box::new(|a, b, _c| {
                *b = Some(a.clone());
            })),
        }
        ScannerResult::Continue
    }
}
