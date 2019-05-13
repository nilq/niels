use std::collections::HashMap;
use std::fmt;

use super::{ OpCode };


#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Float(f64),
    Bool(u8),
    Int(i32),
    Char(char),
    Pointer(u32)
}

#[derive(Clone, Debug, PartialEq)]
pub enum HeapValue {
    Str(String),
    Array(Vec<Value>),
}

#[derive(Clone)]
pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub heap: Vec<HeapValue>,

    pub var_stack: Vec<Value>,
    pub frames: Vec<usize>,

    pub ip: usize,
}


unsafe impl Sync for VirtualMachine {}
unsafe impl Send for VirtualMachine {}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            stack: Vec::new(),
            heap:  Vec::new(),

            var_stack: Vec::new(),
            frames: vec!(0),

            ip: 0,
        }
    }

    pub fn execute_op(&mut self, op: &OpCode) {
        use self::OpCode::*;

        match op {
            LoadInt(ref a) => self.push(Value::Int(*a)),
            LoadFloat(ref a) => self.push(Value::Float(*a)),
            LoadChar(ref a) => self.push(Value::Char(*a)),
            LoadString(ref a) => {
                self.heap.push(HeapValue::Str(a.to_owned()));
                self.push(Value::Pointer(self.heap.len() as u32))
            },
            LoadArray(ref len) => {
                let mut content = Vec::new();

                for _ in 0 .. *len {
                    content.push(self.pop())
                }

                self.heap.push(HeapValue::Array(content));
                self.push(Value::Pointer((self.heap.len() - 1) as u32))
            },
            LoadIndex(i) => {
                let pointer = self.pop();

                if let Value::Pointer(ref heap_ref) = pointer {
                    if let HeapValue::Array(ref content) = self.heap[*heap_ref as usize] {
                        self.push(content[*i as usize].clone());
                    }
                }
            },
            _ => (),
        }
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v)
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn current_frame(&self) -> &usize {
        self.frames.last().unwrap()
    }

    fn push_frame(&mut self) {
        self.frames.push(self.var_stack.len())
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
    }
}