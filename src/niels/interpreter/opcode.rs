use super::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Opcode {
    LoadInt(i32),
    LoadFloat(f64),
    LoadChar(char),
    LoadString(String),
    LoadBool(bool),
    LoadLocal(u32)
    LoadArray,
    LoadIndex(u32),

    Call(u32),

    SetLocal(u32),
    SetArray,
    SetIndex(u32),

    Jmp(u32),
    JmpIf(u32),
    JmpUnless(u32),

    Ret,

    MakeArray(u32),
    
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Or,
    And,
    Lt,
    Gt,
    Eq,
    NEq,
    LtEq,
    GtEq,
}