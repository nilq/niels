#[derive(Clone, PartialEq, Debug)]
pub enum OpCode {
    LoadInt(i32),
    LoadFloat(f64),
    LoadChar(char),
    LoadString(String),
    LoadBool(bool),
    LoadLocal(u32),
    LoadArray(u32),
    LoadIndex(u32),

    Call(u32),

    SetLocal(u32),
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