use std::collections::HashMap;
use std::fmt;

use super::{ OpCode };


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    Float(f64),
    Bool(bool),
    Int(i32),
    Char(char),
    Pointer(u32),
    Nil,
}

impl Value {
    pub fn truthy(&self) -> bool {
        use self::Value::*;

        match &self {
            &Float(_) |
            &Int(_)   |
            &Char(_)  |
            &Pointer(_) => true,
            Bool(true)  => true,
            _           => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HeapValue {
    Str(String),
    Array(Vec<Value>),
}

#[derive(Clone)]
pub struct VirtualMachine {
    pub heap: Vec<HeapValue>,

    pub stack: Vec<Value>,
    pub call_stack: Vec<usize>,
    pub var_stack: [Value; 10000],

    pub var_top: usize,
    
    pub frames: Vec<usize>,
    pub ip: usize,
}


unsafe impl Sync for VirtualMachine {}
unsafe impl Send for VirtualMachine {}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            heap:  Vec::with_capacity(10000),

            stack: Vec::with_capacity(10000),
            call_stack: Vec::with_capacity(10000),
            var_stack: [Value::Nil; 10000],

            frames: vec!(0),

            var_top: 0,

            ip: 0,
        }
    }

    pub fn execute(&mut self, program: &[OpCode]) {
        while self.ip < program.len() {
            self.execute_op(&program[self.ip]);

            self.ip += 1
        }
    }

    pub fn execute_op(&mut self, op: &OpCode) {
        use self::OpCode::*;

        macro_rules! binop {
            ($($pat:pat => $block:block)+) => {{
                let _b = self.pop();
                let _a = self.pop();

                let _result = match (_b, _a) {
                    $($pat => $block)+,
                    _ => panic!("Invalid operands"),
                };
                self.push(_result);
            }}
        }

        match op {
            LoadInt(ref a) => self.push(Value::Int(*a)),
            LoadFloat(ref a) => self.push(Value::Float(*a)),
            LoadBool(ref a) => self.push(Value::Bool(*a)),
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
            LoadLocal(n) => {
                let value = self.var_stack[self.current_frame() + *n as usize];

                self.push(value)
            },
            SetLocal(n) => {
                let value = self.pop();

                self.var_stack[self.current_frame() + *n as usize] = value
            },
            SetIndex(i) => {
                let value   = self.pop();
                let pointer = self.pop();

                if let Value::Pointer(ref heap_ref) = pointer {
                    if let HeapValue::Array(ref mut content) = self.heap[*heap_ref as usize] {
                        content[*i as usize] = value
                    }
                }
            },
            Jmp(n) => {
                self.ip = *n as usize
            },
            JmpIf(n) => {
                let condition = self.pop();

                if condition.truthy() {
                    self.ip = *n as usize
                }
            },
            Call(ret) => {
                self.call_stack.push(self.ip);
                self.ip = *ret as usize
            },
            Ret => {
                self.ip = self.call_stack.pop().unwrap()
            },
            PushFrame => {
                self.push_frame()
            },
            PopFrame => {
                self.var_top = self.pop_frame()
            }


            // TODO: less ugly
            Add => {
                binop! {
                    (Value::Int(a), Value::Int(b))     => { Value::Int(a + b) }
                    (Value::Float(a), Value::Float(b)) => { Value::Float(a + b) }
                    (Value::Int(a), Value::Float(b))   => { Value::Float(a as f64 + b) }
                    (Value::Float(a), Value::Int(b))   => { Value::Float(a + b as f64) }
                }
            },
            Sub => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Int(a - b) }
                (Value::Float(a), Value::Float(b)) => { Value::Float(a - b) }
                (Value::Int(a), Value::Float(b))   => { Value::Float(a as f64 - b) }
                (Value::Float(a), Value::Int(b))   => { Value::Float(a - b as f64) }
            },
            Mul => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Int(a * b) }
                (Value::Float(a), Value::Float(b)) => { Value::Float(a * b) }
                (Value::Int(a), Value::Float(b))   => { Value::Float(a as f64 * b) }
                (Value::Float(a), Value::Int(b))   => { Value::Float(a * b as f64) }
            },
            Div => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Int(a / b) }
                (Value::Float(a), Value::Float(b)) => { Value::Float(a / b) }
                (Value::Int(a), Value::Float(b))   => { Value::Float(a as f64 / b) }
                (Value::Float(a), Value::Int(b))   => { Value::Float(a / b as f64) }
            },
            Mod => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Int(a % b) }
                (Value::Float(a), Value::Float(b)) => { Value::Float(a % b) }
                (Value::Int(a), Value::Float(b))   => { Value::Float(a as f64 % b) }
                (Value::Float(a), Value::Int(b))   => { Value::Float(a % b as f64) }
            },
            Eq => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a == b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a == b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool(a as f64 == b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a == b as f64) }
            },
            NEq => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a != b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a != b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool(a as f64 != b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a != b as f64) }
            },
            Lt => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a < b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a < b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool((a as f64) < b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a < b as f64) }
            },
            Gt => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a > b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a > b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool(a as f64 > b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a > b as f64) }
            },
            LtEq => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a <= b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a <= b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool(a as f64 <= b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a <= b as f64) }
            },
            GtEq => binop! {
                (Value::Int(a), Value::Int(b))     => { Value::Bool(a >= b) }
                (Value::Float(a), Value::Float(b)) => { Value::Bool(a >= b) }
                (Value::Int(a), Value::Float(b))   => { Value::Bool(a as f64 >= b) }
                (Value::Float(a), Value::Int(b))   => { Value::Bool(a >= b as f64) }
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
        self.frames.push(self.var_top)
    }

    fn pop_frame(&mut self) -> usize {
        self.frames.pop().unwrap()
    }
}