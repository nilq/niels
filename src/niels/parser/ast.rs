use std::fmt;
use std::rc::Rc;
use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
    Expression(Expression),
    Assignment(Expression, Expression),
    Return(Option<Rc<Expression>>),
    Implement(Expression, Expression, Option<Expression>),
    Import(String, Vec<String>),
    Function(String, Vec<String>, Vec<Statement>),
    Public(Rc<Statement>),
    Skip,
    Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub node: StatementNode,
    pub pos: Pos,
}

impl Statement {
    pub fn new(node: StatementNode, pos: Pos) -> Self {
        Statement { node, pos }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    Int(u64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),

    Neg(Rc<Expression>),
    Not(Rc<Expression>),

    Identifier(String),
    Binary(Rc<Expression>, Operator, Rc<Expression>),
    Array(Vec<Expression>),
    Record(HashMap<String, Expression>),
    Index(Rc<Expression>, Rc<Expression>, bool), // whether_index_is_an_array_index: bool

    Call(Rc<Expression>, Vec<Expression>),

    Empty,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub pos: Pos,
}

impl Expression {
    pub fn new(node: ExpressionNode, pos: Pos) -> Self {
        Expression { node, pos }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Eq,
    Lt,
    Gt,
    NEq,
    LtEq,
    GtEq,
    Or,
    And,
}

impl Operator {
    pub fn from_str(operator: &str) -> Option<(Operator, u8)> {
        use self::Operator::*;

        let op_prec = match operator {
            "or" => (Or, 0),
            "and" => (And, 0),
            "==" => (Eq, 1),
            "<" => (Lt, 1),
            ">" => (Gt, 1),
            "!=" => (NEq, 1),
            "<=" => (LtEq, 1),
            ">=" => (GtEq, 1),
            "+" => (Add, 2),
            "-" => (Sub, 2),
            "++" => (Concat, 2),
            "*" => (Mul, 3),
            "/" => (Div, 3),
            "%" => (Mod, 3),
            "^" => (Pow, 4),
            _ => return None,
        };

        Some(op_prec)
    }

    pub fn as_str(&self) -> &str {
        use self::Operator::*;

        match *self {
            Add => "+",
            Sub => "-",
            Concat => "++",
            Pow => "^",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Eq => "==",
            Lt => "<",
            Gt => ">",
            NEq => "!=",
            LtEq => "<=",
            GtEq => ">=",
            Or => "or",
            And => "and",
        }
    }

    pub fn is_compoundable(operator: &str) -> bool {
        ["+", "-", "*", "/", "++", "%", "^", "not", "or", "and"].contains(&operator)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
