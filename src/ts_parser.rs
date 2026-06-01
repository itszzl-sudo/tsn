use anyhow::Result;
use crate::hir::{BinOp, HirExpr, UnaryOp};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Clone)]
enum Token {
    Fn, Let, Const, If, Else, While, For, Return, Typeof, Break, Continue,
    Ident(String),
    Number(f64),
    String(String),
    Bool(bool),
    Null, Undefined,
    
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    
    Plus, Minus, Star, Slash, Percent,
    EqEqEq, NotEqEq, Lt, Le, Gt, Ge,
    AndAnd, OrOr,
    Not,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign,
    PlusPlus, MinusMinus,
    Question,
    
    Comma, Semicolon, Colon, Dot,
    
    Eof,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let tokens = Self::tokenize(source);
        Self { tokens, pos: 0 }
    }
    
    fn tokenize(source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = source.chars().peekable();
        
        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                }
                '(' => { tokens.push(Token::LParen); chars.next(); }
                ')' => { tokens.push(Token::RParen); chars.next(); }
                '{' => { tokens.push(Token::LBrace); chars.next(); }
                '}' => { tokens.push(Token::RBrace); chars.next(); }
                '[' => { tokens.push(Token::LBracket); chars.next(); }
                ']' => { tokens.push(Token::RBracket); chars.next(); }
                ',' => { tokens.push(Token::Comma); chars.next(); }
                ';' => { tokens.push(Token::Semicolon); chars.next(); }
                ':' => { tokens.push(Token::Colon); chars.next(); }
                '.' => { tokens.push(Token::Dot); chars.next(); }
                '+' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::PlusAssign);
                    } else if chars.peek() == Some(&'+') {
                        chars.next();
                        tokens.push(Token::PlusPlus);
                    } else {
                        tokens.push(Token::Plus);
                    }
                }
                '-' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::MinusAssign);
                    } else if chars.peek() == Some(&'-') {
                        chars.next();
                        tokens.push(Token::MinusMinus);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::StarAssign);
                    } else {
                        tokens.push(Token::Star);
                    }
                }
                '/' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::SlashAssign);
                    } else if chars.peek() == Some(&'/') {
                        while let Some(&ch) = chars.peek() {
                            if ch == '\n' {
                                chars.next();
                                break;
                            }
                            chars.next();
                        }
                    } else if chars.peek() == Some(&'*') {
                        chars.next();
                        loop {
                            if let Some(&ch) = chars.peek() {
                                chars.next();
                                if ch == '*' && chars.peek() == Some(&'/') {
                                    chars.next();
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    } else {
                        tokens.push(Token::Slash);
                    }
                }
                '%' => { tokens.push(Token::Percent); chars.next(); }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        if chars.peek() == Some(&'=') {
                            chars.next();
                            tokens.push(Token::EqEqEq);
                        } else {
                            tokens.push(Token::EqEqEq);
                        }
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        if chars.peek() == Some(&'=') {
                            chars.next();
                            tokens.push(Token::NotEqEq);
                        } else {
                            tokens.push(Token::NotEqEq);
                        }
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Le);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Ge);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '&' => {
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(Token::AndAnd);
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(Token::OrOr);
                    }
                }
                '?' => {
                    chars.next();
                    tokens.push(Token::Question);
                }
                '"' | '\'' => {
                    let quote = c;
                    chars.next();
                    let mut s = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == quote {
                            chars.next();
                            break;
                        }
                        s.push(ch);
                        chars.next();
                    }
                    tokens.push(Token::String(s));
                }
                '0'..='9' => {
                    let mut num = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == '.' {
                            num.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num.parse().unwrap_or(0.0)));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            ident.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    
                    let token = match ident.as_str() {
                        "function" => Token::Fn,
                        "let" => Token::Let,
                        "const" => Token::Const,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "for" => Token::For,
                        "return" => Token::Return,
                        "typeof" => Token::Typeof,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        "null" => Token::Null,
                        "undefined" => Token::Undefined,
                        _ => Token::Ident(ident),
                    };
                    tokens.push(token);
                }
                _ => {
                    chars.next();
                }
            }
        }
        
        tokens.push(Token::Eof);
        tokens
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }
    
    fn expect(&mut self, expected: &Token) -> Result<()> {
        let token = self.advance();
        if std::mem::discriminant(&token) != std::mem::discriminant(expected) {
            anyhow::bail!("Expected {:?}, got {:?}", expected, token);
        }
        Ok(())
    }
    
    pub fn parse_module(&mut self) -> Result<Vec<HirExpr>> {
        let mut stmts = Vec::new();
        
        while !matches!(self.peek(), Token::Eof) {
            if let Some(stmt) = self.parse_stmt()? {
                stmts.push(stmt);
            }
        }
        
        Ok(stmts)
    }
    
    fn parse_stmt(&mut self) -> Result<Option<HirExpr>> {
        match self.peek() {
            Token::Fn => Ok(Some(self.parse_function()?)),
            Token::Let | Token::Const => Ok(Some(self.parse_var()?)),
            Token::Return => Ok(Some(self.parse_return()?)),
            Token::If => Ok(Some(self.parse_if()?)),
            Token::While => Ok(Some(self.parse_while()?)),
            Token::For => Ok(Some(self.parse_for()?)),
            Token::LBrace => Ok(Some(self.parse_block()?)),
            Token::Break => {
                self.advance();
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                }
                Ok(Some(HirExpr::Break))
            }
            Token::Continue => {
                self.advance();
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                }
                Ok(Some(HirExpr::Continue))
            }
            Token::Semicolon => {
                self.advance();
                Ok(None)
            }
            _ => {
                let expr = self.parse_expr()?;
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                }
                Ok(Some(expr))
            }
        }
    }
    
    fn parse_function(&mut self) -> Result<HirExpr> {
        self.expect(&Token::Fn)?;
        let name = match self.advance() {
            Token::Ident(n) => n,
            _ => anyhow::bail!("Expected function name"),
        };
        
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen) {
            if let Token::Ident(p) = self.advance() {
                params.push(p);
            }
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RParen)?;
        
        let body = self.parse_block_stmts()?;
        
        Ok(HirExpr::Function { name, params, body })
    }
    
    fn parse_var(&mut self) -> Result<HirExpr> {
        let is_mut = matches!(self.advance(), Token::Let);
        let name = match self.advance() {
            Token::Ident(n) => n,
            _ => anyhow::bail!("Expected variable name"),
        };
        
        let init = if matches!(self.peek(), Token::Assign) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }
        
        Ok(HirExpr::Var { name, init, is_mut })
    }
    
    fn parse_return(&mut self) -> Result<HirExpr> {
        self.expect(&Token::Return)?;
        let val = if !matches!(self.peek(), Token::Semicolon | Token::RBrace) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }
        Ok(HirExpr::Return(val))
    }
    
    fn parse_if(&mut self) -> Result<HirExpr> {
        self.expect(&Token::If)?;
        self.expect(&Token::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.expect(&Token::RParen)?;
        
        let then_body = self.parse_block_stmts()?;
        
        let else_body = if matches!(self.peek(), Token::Else) {
            self.advance();
            Some(self.parse_block_stmts()?)
        } else {
            None
        };
        
        Ok(HirExpr::If { cond, then_body, else_body })
    }
    
    fn parse_while(&mut self) -> Result<HirExpr> {
        self.expect(&Token::While)?;
        self.expect(&Token::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.expect(&Token::RParen)?;
        
        let body = self.parse_block_stmts()?;
        
        Ok(HirExpr::While { cond, body })
    }
    
    fn parse_for(&mut self) -> Result<HirExpr> {
        self.expect(&Token::For)?;
        self.expect(&Token::LParen)?;
        
        let init = if matches!(self.peek(), Token::Let | Token::Const) {
            Some(Box::new(self.parse_var()?))
        } else if !matches!(self.peek(), Token::Semicolon) {
            let expr = self.parse_expr()?;
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
            Some(Box::new(expr))
        } else {
            None
        };
        if init.is_none() {
            self.expect(&Token::Semicolon)?;
        }
        
        let cond = if !matches!(self.peek(), Token::Semicolon) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(&Token::Semicolon)?;
        
        let update = if !matches!(self.peek(), Token::RParen) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(&Token::RParen)?;
        
        let body = self.parse_block_stmts()?;
        
        Ok(HirExpr::For { init, cond, update, body })
    }
    
    fn parse_block(&mut self) -> Result<HirExpr> {
        self.expect(&Token::LBrace)?;
        let stmts = self.parse_block_stmts()?;
        Ok(HirExpr::Block(stmts))
    }
    
    fn parse_block_stmts(&mut self) -> Result<Vec<HirExpr>> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            if let Some(stmt) = self.parse_stmt()? {
                stmts.push(stmt);
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }
    
    fn parse_expr(&mut self) -> Result<HirExpr> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<HirExpr> {
        let left = self.parse_ternary()?;
        
        match self.peek() {
            Token::Assign => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(HirExpr::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                })
            }
            Token::PlusAssign => {
                self.advance();
                let right = self.parse_assignment()?;
                let value = HirExpr::Binary {
                    op: BinOp::Add,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                };
                Ok(HirExpr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            Token::MinusAssign => {
                self.advance();
                let right = self.parse_assignment()?;
                let value = HirExpr::Binary {
                    op: BinOp::Sub,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                };
                Ok(HirExpr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            Token::StarAssign => {
                self.advance();
                let right = self.parse_assignment()?;
                let value = HirExpr::Binary {
                    op: BinOp::Mul,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                };
                Ok(HirExpr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            Token::SlashAssign => {
                self.advance();
                let right = self.parse_assignment()?;
                let value = HirExpr::Binary {
                    op: BinOp::Div,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                };
                Ok(HirExpr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            _ => Ok(left)
        }
    }
    
    fn parse_or(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_and()?;
        
        while matches!(self.peek(), Token::OrOr) {
            self.advance();
            let right = self.parse_and()?;
            left = HirExpr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_ternary(&mut self) -> Result<HirExpr> {
        let cond = self.parse_or()?;
        
        if matches!(self.peek(), Token::Question) {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(&Token::Colon)?;
            let else_expr = self.parse_ternary()?;
            
            Ok(HirExpr::Ternary {
                cond: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(cond)
        }
    }
    
    fn parse_and(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_equality()?;
        
        while matches!(self.peek(), Token::AndAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = HirExpr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_comparison()?;
        
        loop {
            let op = match self.peek() {
                Token::EqEqEq => BinOp::Eq,
                Token::NotEqEq => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = HirExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_additive()?;
        
        loop {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = HirExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_additive(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_multiplicative()?;
        
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = HirExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative(&mut self) -> Result<HirExpr> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = HirExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<HirExpr> {
        if matches!(self.peek(), Token::Typeof) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(HirExpr::Typeof(Box::new(operand)));
        }
        
        let op = match self.peek() {
            Token::Not => UnaryOp::Not,
            Token::Minus => UnaryOp::Neg,
            _ => return self.parse_primary(),
        };
        self.advance();
        let operand = self.parse_unary()?;
        Ok(HirExpr::Unary {
            op,
            operand: Box::new(operand),
        })
    }
    
    fn parse_primary(&mut self) -> Result<HirExpr> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(HirExpr::Number(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(HirExpr::String(s))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(HirExpr::Boolean(b))
            }
            Token::Null => {
                self.advance();
                Ok(HirExpr::Null)
            }
            Token::Undefined => {
                self.advance();
                Ok(HirExpr::Undefined)
            }
            Token::Ident(name) => {
                self.advance();
                
                let mut expr = HirExpr::Identifier(name);
                
                loop {
                    match self.peek() {
                        Token::LParen => {
                            self.advance();
                            let mut args = Vec::new();
                            while !matches!(self.peek(), Token::RParen) {
                                args.push(self.parse_expr()?);
                                if matches!(self.peek(), Token::Comma) {
                                    self.advance();
                                }
                            }
                            self.expect(&Token::RParen)?;
                            expr = HirExpr::Call {
                                callee: Box::new(expr),
                                args,
                            };
                        }
                        Token::LBracket => {
                            self.advance();
                            let index = self.parse_expr()?;
                            self.expect(&Token::RBracket)?;
                            expr = HirExpr::Index {
                                object: Box::new(expr),
                                index: Box::new(index),
                            };
                        }
                        Token::Dot => {
                            self.advance();
                            let name = match self.peek().clone() {
                                Token::Ident(name) => {
                                    self.advance();
                                    name
                                }
                                _ => anyhow::bail!("Expected property name after '.'"),
                            };
                            expr = HirExpr::Property {
                                object: Box::new(expr),
                                name,
                            };
                        }
                        Token::PlusPlus => {
                            self.advance();
                            let one = HirExpr::Number(1.0);
                            expr = HirExpr::Assign {
                                target: Box::new(expr.clone()),
                                value: Box::new(HirExpr::Binary {
                                    op: BinOp::Add,
                                    left: Box::new(expr),
                                    right: Box::new(one),
                                }),
                            };
                        }
                        Token::MinusMinus => {
                            self.advance();
                            let one = HirExpr::Number(1.0);
                            expr = HirExpr::Assign {
                                target: Box::new(expr.clone()),
                                value: Box::new(HirExpr::Binary {
                                    op: BinOp::Sub,
                                    left: Box::new(expr),
                                    right: Box::new(one),
                                }),
                            };
                        }
                        _ => break,
                    }
                }
                
                Ok(expr)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !matches!(self.peek(), Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(HirExpr::Array(elements))
            }
            Token::LBrace => {
                self.advance();
                let mut properties = Vec::new();
                while !matches!(self.peek(), Token::RBrace) {
                    let key = match self.peek().clone() {
                        Token::Ident(name) => {
                            self.advance();
                            name
                        }
                        Token::String(s) => {
                            self.advance();
                            s
                        }
                        _ => anyhow::bail!("Expected property name"),
                    };
                    self.expect(&Token::Colon)?;
                    let value = self.parse_expr()?;
                    properties.push((key, value));
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RBrace)?;
                Ok(HirExpr::Object(properties))
            }
            _ => anyhow::bail!("Unexpected token: {:?}", self.peek()),
        }
    }
}

pub fn parse(source: &str) -> Result<Vec<HirExpr>> {
    let mut parser = Parser::new(source);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = parse("let x = 42;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let result = parse("function add(a, b) { return a + b; }");
        assert!(result.is_ok());
        let hir = result.unwrap();
        assert_eq!(hir.len(), 1);
        if let HirExpr::Function { name, params, .. } = &hir[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        } else {
            panic!("Expected Function");
        }
    }

    #[test]
    fn test_parse_if_else() {
        let result = parse("if (x > 0) { print(x); } else { print(0); }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_while() {
        let result = parse("while (x > 0) { x = x - 1; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_for() {
        let result = parse("for (let i = 0; i < 10; i = i + 1) { print(i); }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let result = parse("let arr = [1, 2, 3];");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_object() {
        let result = parse("let obj = {x: 10, y: 20};");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string() {
        let result = parse("let s = \"hello\";");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_break_continue() {
        let result = parse("while (true) { if (x > 0) { break; } continue; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_typeof() {
        let result = parse("let t = typeof x;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ternary() {
        let result = parse("let y = x > 0 ? x : 0;");
        assert!(result.is_ok());
    }
}
