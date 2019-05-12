use std::collections::HashMap;

#[macro_use]
use nanbox::*;

make_nanbox!{
    pub unsafe enum Value, Variant {
        Float(f64),
        Bool(u8),
        Int(i32),
        Char(char),
        Pointer(*mut ())
    }
}

pub enum HeapValue {
    Str(String),
    Array(Vec<Value>),
    Hash(HashMap<Self, Value>),
}


pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub heap: Vec<HeapValue>,

    pub ip: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            stack: Vec::new(),
            heap:  Vec::new(),

            ip: 0,
        }
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v)
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
}