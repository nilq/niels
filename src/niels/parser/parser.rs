use super::*;
use Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
    index: usize,
    tokens: Vec<Token>,
    source: &'p Source,

    indent_standard: usize,
    indent:          usize,
}

impl<'p> Parser<'p> {
    pub fn new(tokens: Vec<Token>, source: &'p Source) -> Self {
        Parser {
            tokens,
            source,
            index: 0,

            indent_standard: 0,
            indent: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
        let mut ast = Vec::new();

        while self.remaining() > 0 {
            ast.push(self.parse_statement()?)
        }

        Ok(ast)
    }

    fn parse_statement(&mut self) -> Result<Statement, ()> {
        use self::TokenType::*;

        while self.current_type() == EOL && self.remaining() != 0 {
            self.next()?
        }

        let position = self.current_position();

        let statement = match self.current_type() {
            Identifier => {
                let backup_index = self.index;
                let position = self.current_position();
                let name = self.eat_type(&Identifier)?;

                match self.current_lexeme().as_str() {
                    "=" => {
                        self.next()?;

                        Statement::new(
                            StatementNode::Assignment(
                                Expression::new(ExpressionNode::Identifier(name), position.clone()),
                                self.parse_expression()?,
                            ),
                            position,
                        )
                    }

                    _ => {
                        let expression =
                            Expression::new(ExpressionNode::Identifier(name), position.clone());

                        if let Some(result) = self.try_parse_compound(&expression)? {
                            result
                        } else {
                            self.index = backup_index;

                            let expression = self.parse_atom()?;

                            if let Some(result) = self.try_parse_compound(&expression)? {
                                result
                            } else {
                                self.index = backup_index;

                                let expression = self.parse_expression()?;
                                let position = expression.pos.clone();

                                if self.current_lexeme() == "=" {
                                    self.next()?;

                                    Statement::new(
                                        StatementNode::Assignment(
                                            expression,
                                            self.parse_expression()?,
                                        ),
                                        position,
                                    )
                                } else {
                                    Statement::new(StatementNode::Expression(expression), position)
                                }
                            }
                        }
                    }
                }
            },

            Keyword => match self.current_lexeme().as_str() {
                "return" => {
                    self.next()?;

                    if ["}", "\n"].contains(&self.current_lexeme().as_str()) {
                        Statement::new(
                        StatementNode::Return(None),
                        position
                        )
                    } else {
                        Statement::new(
                        StatementNode::Return(Some(Rc::new(self.parse_expression()?))),
                        self.span_from(position)
                        )
                    }
                },

                "pub" => {
                    self.next()?;

                    return Ok(
                        Statement::new(
                            StatementNode::Public(
                                Rc::new(self.parse_statement()?)
                            ),
                            position,
                        )
                    )
                },

                "funk" => {
                    self.next()?;

                    let name = self.eat_type(&TokenType::Identifier)?;

                    let params = if self.current_lexeme() == "(" {
                        self.parse_block_of(("(", ")"), &Self::_parse_name_comma)?
                    } else {
                        Vec::new()
                    };

                    self.eat_lexeme(":")?;

                    let body = if self.current_lexeme() == "\n" {
                        self.next()?;

                        self.parse_body()?
                    } else {
                        vec!(self.parse_statement()?)
                    };

                    return Ok(
                        Statement::new(
                            StatementNode::Function(
                                name,
                                params,
                                body,
                            ),
                            position,
                        )
                    )
                },

                _ => {
                    let expression = self.parse_expression()?;

                    let position = expression.pos.clone();

                    Statement::new(
                        StatementNode::Expression(expression),
                        position,
                    )
                },
            },

            _ => {
                let expression = self.parse_expression()?;
                let position = expression.pos.clone();

                if let Some(result) = self.try_parse_compound(&expression)? {
                    result
                } else {
                    if self.current_lexeme() == "=" {
                        self.next()?;

                        Statement::new(
                            StatementNode::Assignment(expression, self.parse_expression()?),
                            position,
                        )
                    } else {
                        Statement::new(StatementNode::Expression(expression), position)
                    }
                }
            }
        };

        self.new_line()?;

        Ok(statement)
    }

    fn parse_body(&mut self) -> Result<Vec<Statement>, ()> {
        let backup_indent = self.indent;
        self.indent       = self.get_indent();

        if self.indent_standard == 0 {
            self.indent_standard = self.indent
        } else {
            if self.indent % self.indent_standard != 0 {
                return Err(
                    response!(
                        Wrong(format!("found inconsistently indented token")),
                        self.source.file,
                        self.current_position()
                    )
                )
            }
        }

        let mut stack = Vec::new();

        while !self.is_dedent() && self.remaining() > 0 {
            let statement = self.parse_statement()?;

            self.next_newline()?;

            stack.push(statement)
        }

        self.indent = backup_indent;

        Ok(stack)
    }

    fn try_parse_compound(&mut self, left: &Expression) -> Result<Option<Statement>, ()> {
        if self.current_type() != TokenType::Operator {
            return Ok(None);
        }

        let backup_index = self.index;

        let c = self.eat_type(&TokenType::Operator)?;

        let mut result = None;

        if self::Operator::is_compoundable(&c) {
            let op = self::Operator::from_str(&c).unwrap().0;

            let position = self.current_position();

            if self.current_lexeme() == "=" {
                self.next()?;

                let right = self.parse_expression()?;
                let ass = Statement::new(
                    StatementNode::Assignment(
                        left.clone(),
                        Expression::new(
                            ExpressionNode::Binary(Rc::new(left.clone()), op, Rc::new(right)),
                            self.span_from(position.clone()),
                        ),
                    ),
                    self.span_from(position),
                );

                result = Some(ass)
            } else {
                self.index = backup_index
            }
        }

        Ok(result)
    }

    fn parse_expression(&mut self) -> Result<Expression, ()> {
        let atom = self.parse_atom()?;

        if self.current_type() == TokenType::Operator {
            self.parse_binary(atom)
        } else {
            Ok(atom)
        }
    }

    fn parse_atom(&mut self) -> Result<Expression, ()> {
        use self::TokenType::*;

        if self.remaining() == 0 {
            Ok(Expression::new(
                ExpressionNode::EOF,
                self.current_position(),
            ))
        } else {
            let token_type = self.current_type().clone();
            let position = self.current_position();

            let expression = match token_type {
                Int => Expression::new(
                    ExpressionNode::Int(self.eat()?.parse::<u64>().unwrap()),
                    position,
                ),

                Float => Expression::new(
                    ExpressionNode::Float(self.eat()?.parse::<f64>().unwrap()),
                    position,
                ),

                Char => Expression::new(
                    ExpressionNode::Char(self.eat()?.chars().last().unwrap()),
                    position,
                ),

                Str => Expression::new(ExpressionNode::Str(self.eat()?), position),

                Identifier => Expression::new(ExpressionNode::Identifier(self.eat()?), position),

                Bool => Expression::new(ExpressionNode::Bool(self.eat()? == "true"), position),

                Symbol => match self.current_lexeme().as_str() {
                    "[" => Expression::new(
                        ExpressionNode::Array(self.parse_block_of(("[", "]"), &Self::_parse_expression_comma)?),
                        self.span_from(position)
                    ),

                    "(" => {
                        self.next()?;
                        self.next_newline()?;

                        if self.current_lexeme() == ")" && self.current_type() == TokenType::Symbol {
                        self.next()?;

                        Expression::new(
                            ExpressionNode::Empty,
                            self.span_from(position)
                        )
                        } else {
                        let expression = self.parse_expression()?;

                        self.eat_lexeme(")")?;

                        expression
                        }
                    },

                    ref symbol => return Err(
                        response!(
                            Wrong(format!("unexpected symbol `{}`", symbol)),
                            self.source.file,
                            self.current_position()
                        )
                    )
                },

                ref token_type => {
                    return Err(response!(
                        Wrong(format!("unexpected token `{}`", token_type)),
                        self.source.file,
                        self.current_position()
                    ))
                }
            };

            if self.remaining() > 0 {
                self.parse_postfix(expression)
            } else {
                Ok(expression)
            }
        }
    }

    fn parse_postfix(&mut self, expression: Expression) -> Result<Expression, ()> {
        if self.remaining() == 0 {
            return Ok(expression);
        }

        match self.current_type() {
            TokenType::Symbol => match self.current_lexeme().as_str() {
                "(" => {
                    let args = self.parse_block_of(("(", ")"), &Self::_parse_expression_comma)?;

                    let position = expression.pos.clone();

                    let call = Expression::new(
                        ExpressionNode::Call(Rc::new(expression), args),
                        self.span_from(position),
                    );

                    self.parse_postfix(call)
                }

                "[" => {
                    self.next()?;

                    let expr = self.parse_expression()?;

                    self.eat_lexeme("]")?;

                    let position = expression.pos.clone();

                    let index = Expression::new(
                        ExpressionNode::Index(Rc::new(expression), Rc::new(expr), true),
                        self.span_from(position),
                    );

                    self.parse_postfix(index)
                }

                _ => Ok(expression),
            },

            _ => Ok(expression),
        }
    }

    fn parse_binary(&mut self, left: Expression) -> Result<Expression, ()> {
        let left_position = left.pos.clone();

        let mut expression_stack = vec![left];
        let mut operator_stack = vec![Operator::from_str(&self.eat()?).unwrap()];

        expression_stack.push(self.parse_atom()?);

        while operator_stack.len() > 0 {
            while self.current_type() == TokenType::Operator {
                let position = self.current_position();
                let (operator, precedence) = Operator::from_str(&self.eat()?).unwrap();

                if precedence < operator_stack.last().unwrap().1 {
                    let right = expression_stack.pop().unwrap();
                    let left = expression_stack.pop().unwrap();

                    expression_stack.push(Expression::new(
                        ExpressionNode::Binary(
                            Rc::new(left),
                            operator_stack.pop().unwrap().0,
                            Rc::new(right),
                        ),
                        self.current_position(),
                    ));

                    if self.remaining() > 0 {
                        expression_stack.push(self.parse_atom()?);
                        operator_stack.push((operator, precedence))
                    } else {
                        return Err(response!(
                            Wrong("reached EOF in operation"),
                            self.source.file,
                            position
                        ));
                    }
                } else {
                    expression_stack.push(self.parse_atom()?);
                    operator_stack.push((operator, precedence))
                }
            }

            let right = expression_stack.pop().unwrap();
            let left = expression_stack.pop().unwrap();

            expression_stack.push(Expression::new(
                ExpressionNode::Binary(
                    Rc::new(left),
                    operator_stack.pop().unwrap().0,
                    Rc::new(right),
                ),
                self.current_position(),
            ));
        }

        let expression = expression_stack.pop().unwrap();

        Ok(Expression::new(
            expression.node,
            self.span_from(left_position),
        ))
    }

    fn parse_block_of<B>(
        &mut self,
        delimeters: (&str, &str),
        parse_with: &Fn(&mut Self) -> Result<Option<B>, ()>,
    ) -> Result<Vec<B>, ()> {
        self.eat_lexeme(delimeters.0)?;

        if self.current_lexeme() == delimeters.1 {
            self.next()?;

            return Ok(Vec::new());
        }

        let mut block_tokens = Vec::new();
        let mut nest_count = 1;

        while nest_count > 0 {
            if self.current_lexeme() == delimeters.1 && self.current_type() == TokenType::Symbol {
                nest_count -= 1
            } else if self.current_lexeme() == delimeters.0
                && self.current_type() == TokenType::Symbol
            {
                nest_count += 1
            }

            if nest_count == 0 {
                break;
            } else {
                block_tokens.push(self.current());

                self.next()?
            }
        }

        self.eat_lexeme(delimeters.1)?;

        if !block_tokens.is_empty() {
            let mut parser = Parser::new(block_tokens, self.source);
            let mut block = Vec::new();

            while let Some(element) = parse_with(&mut parser)? {
                block.push(element)
            }

            Ok(block)
        } else {
            Ok(Vec::new())
        }
    }

    fn _parse_name_comma(self: &mut Self) -> Result<Option<String>, ()> {
        if self.remaining() == 0 {
            Ok(None)
        } else {
            if self.remaining() > 0 && self.current_lexeme() == "\n" {
                self.next()?
            }

            let t = self.eat_type(&TokenType::Identifier)?;

            if self.remaining() > 0 {
                if ![",", "\n"].contains(&self.current_lexeme().as_str()) {
                    return Err(
                        response!(
                            Wrong(format!("expected `,` or newline, found `{}`", self.current_lexeme())),
                            self.source.file,
                            self.current_position()
                        )
                    )
                } else {
                    self.next()?;
                }

                if self.remaining() > 0 && self.current_lexeme() == "\n" {
                    self.next()?
                }
            }

            Ok(Some(t))
        }
    }

    fn _parse_expression(self: &mut Self) -> Result<Option<Expression>, ()> {
        let expression = self.parse_expression()?;

        match expression.node {
            ExpressionNode::EOF => Ok(None),
            _ => Ok(Some(expression)),
        }
    }

    fn _parse_expression_comma(self: &mut Self) -> Result<Option<Expression>, ()> {
        if self.remaining() > 0 && self.current_lexeme() == "\n" {
            self.next()?
        }

        let expression = Self::_parse_expression(self);

        if self.remaining() > 0 && self.current_lexeme() == "\n" {
            self.next()?
        }

        if self.remaining() > 0 {
            self.eat_lexeme(",")?;

            if self.remaining() > 0 && self.current_lexeme() == "\n" {
                self.next()?
            }
        }

        expression
    }

    fn new_line(&mut self) -> Result<(), ()> {
        if self.remaining() > 0 {
            match self.current_lexeme().as_str() {
                "\n" => self.next(),
                _ => Err(response!(
                    Wrong(format!(
                        "expected new line found: `{}`",
                        self.current_lexeme()
                    )),
                    self.source.file,
                    self.current_position()
                )),
            }
        } else {
            Ok(())
        }
    }

    fn next_newline(&mut self) -> Result<(), ()> {
        while self.current_lexeme() == "\n" && self.remaining() > 0 {
            self.next()?
        }

        Ok(())
    }

    fn get_indent(&self) -> usize {
        self.current().slice.0 - 1
    }

    fn is_dedent(&self) -> bool {
        self.get_indent() < self.indent && self.current_lexeme() != "\n"
    }

    fn next(&mut self) -> Result<(), ()> {
        if self.index <= self.tokens.len() {
            self.index += 1;
            Ok(())
        } else {
            Err(response!(
                Wrong("moving outside token stack"),
                self.source.file,
                self.current_position()
            ))
        }
    }

    fn remaining(&self) -> usize {
        self.tokens.len().saturating_sub(self.index)
    }

    fn current_position(&self) -> Pos {
        let current = self.current();

        Pos(current.line.clone(), current.slice)
    }

    fn span_from(&self, left_position: Pos) -> Pos {
        let Pos(ref line, ref slice) = left_position;
        let Pos(_, ref slice2) = self.current_position();

        Pos(
            line.clone(),
            (
                slice.0,
                if slice2.1 < line.1.len() {
                    slice2.1
                } else {
                    line.1.len()
                },
            ),
        )
    }

    fn current(&self) -> Token {
        if self.index > self.tokens.len() - 1 {
            self.tokens[self.tokens.len() - 1].clone()
        } else {
            self.tokens[self.index].clone()
        }
    }

    fn eat(&mut self) -> Result<String, ()> {
        let lexeme = self.current().lexeme;
        self.next()?;

        Ok(lexeme)
    }

    fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
        if self.current_lexeme() == lexeme {
            let lexeme = self.current().lexeme;
            self.next()?;

            Ok(lexeme)
        } else {
            Err(response!(
                Wrong(format!(
                    "expected `{}`, found `{}`",
                    lexeme,
                    self.current_lexeme()
                )),
                self.source.file,
                self.current_position()
            ))
        }
    }

    fn eat_type(&mut self, token_type: &TokenType) -> Result<String, ()> {
        if self.current_type() == *token_type {
            let lexeme = self.current().lexeme.clone();
            self.next()?;

            Ok(lexeme)
        } else {
            Err(response!(
                Wrong(format!(
                    "expected `{}`, found `{}`",
                    token_type,
                    self.current_type()
                )),
                self.source.file,
                self.current_position()
            ))
        }
    }

    fn current_lexeme(&self) -> String {
        self.current().lexeme.clone()
    }

    fn current_type(&self) -> TokenType {
        self.current().token_type
    }

    fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
        if self.current_type() == token_type {
            Ok(())
        } else {
            Err(response!(
                Wrong(format!(
                    "expected `{}`, found `{}`",
                    token_type,
                    self.current_type()
                )),
                self.source.file,
                self.current_position()
            ))
        }
    }

    fn expect_lexeme(&self, lexeme: &str) -> Result<(), ()> {
        if self.current_lexeme() == lexeme {
            Ok(())
        } else {
            Err(response!(
                Wrong(format!(
                    "expected `{}`, found `{}`",
                    lexeme,
                    self.current_lexeme()
                )),
                self.source.file,
                self.current_position()
            ))
        }
    }

    pub fn fold_expression(expression: &Expression) -> Result<Expression, ()> {
        use self::ExpressionNode::*;
        use self::Operator::*;

        let node = match expression.node {
            Binary(ref left, ref op, ref right) => {
                let node = match (
                    &Self::fold_expression(&*left)?.node,
                    op,
                    &Self::fold_expression(&*right)?.node,
                ) {
                    (&Int(ref a), &Add, &Int(ref b)) => Int(a + b),
                    (&Float(ref a), &Add, &Float(ref b)) => Float(a + b),
                    (&Int(ref a), &Sub, &Int(ref b)) => Int(a - b),
                    (&Float(ref a), &Sub, &Float(ref b)) => Float(a - b),
                    (&Int(ref a), &Mul, &Int(ref b)) => Int(a * b),
                    (&Float(ref a), &Mul, &Float(ref b)) => Float(a * b),
                    (&Int(ref a), &Div, &Int(ref b)) => Int(a / b),
                    (&Float(ref a), &Div, &Float(ref b)) => Float(a / b),

                    _ => expression.node.clone(),
                };

                Expression::new(node, expression.pos.clone())
            }

            _ => expression.clone(),
        };

        Ok(node)
    }
}
