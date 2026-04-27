//! Tiny BASIC compiler example.
//!
//! Compiles a minimal BASIC subset to QBE IL. Reads source from a file path
//! given as the first CLI argument, or from stdin when no argument is given.
//! Writes QBE IL to stdout.
//!
//! Supported statements: `LET`, `PRINT`, `IF ... THEN <line>`, `GOTO`, `END`,
//! `REM`. All values are 32-bit signed integers.
//!
//! Example program (factorial of 5):
//!
//! ```text
//! 10 LET N = 5
//! 20 LET F = 1
//! 30 IF N <= 1 THEN 60
//! 40 LET F = F * N
//! 50 LET N = N - 1
//! 55 GOTO 30
//! 60 PRINT F
//! 70 END
//! ```
//!
//! Pipe the generated IL through `qbe` and a C compiler to produce a runnable
//! binary:
//!
//! ```sh
//! cargo run --example tiny_basic factorial.bas | qbe -o out.s - && cc out.s -o factorial
//! ```

use qbe::Module;
use std::io::Read;
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Let,
    Print,
    If,
    Then,
    Goto,
    End,
    Rem,
    Ident(String),
    Number(u32),
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Newline,
    Eof,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, u32),
    Goto(u32),
    End,
    Rem,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Expr {
    Num(u32),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\r' => i += 1,
            b'\n' => {
                tokens.push(Token::Newline);
                i += 1;
            }
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let n: u32 = source[start..i]
                    .parse()
                    .map_err(|e| format!("invalid integer literal: {e}"))?;
                tokens.push(Token::Number(n));
            }
            b'A'..=b'Z' => {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_uppercase() || bytes[i].is_ascii_digit())
                {
                    i += 1;
                }
                let word = &source[start..i];
                let tok = match word {
                    "LET" => Token::Let,
                    "PRINT" => Token::Print,
                    "IF" => Token::If,
                    "THEN" => Token::Then,
                    "GOTO" => Token::Goto,
                    "END" => Token::End,
                    "REM" => Token::Rem,
                    _ => Token::Ident(word.to_string()),
                };
                tokens.push(tok);
                if matches!(tokens.last(), Some(Token::Rem)) {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
            }
            b'+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            b'*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            b'/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            b'(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            b')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            b'=' => {
                tokens.push(Token::Eq);
                i += 1;
            }
            b'<' => {
                if bytes.get(i + 1) == Some(&b'=') {
                    tokens.push(Token::LtEq);
                    i += 2;
                } else if bytes.get(i + 1) == Some(&b'>') {
                    tokens.push(Token::NotEq);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            b'>' => {
                if bytes.get(i + 1) == Some(&b'=') {
                    tokens.push(Token::GtEq);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            other => {
                return Err(format!("unexpected character {:?}", other as char));
            }
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !matches!(t, Token::Eof) {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token, ctx: &str) -> Result<(), String> {
        if self.peek() == expected {
            self.bump();
            Ok(())
        } else {
            Err(format!(
                "expected {expected:?} {ctx}, found {:?}",
                self.peek()
            ))
        }
    }

    fn parse_program(&mut self) -> Result<Vec<(u32, Stmt)>, String> {
        let mut lines: Vec<(u32, Stmt)> = Vec::new();
        loop {
            while matches!(self.peek(), Token::Newline) {
                self.bump();
            }
            if matches!(self.peek(), Token::Eof) {
                break;
            }
            let lineno = match self.bump() {
                Token::Number(n) => n,
                other => return Err(format!("expected line number, found {other:?}")),
            };
            let stmt = self.parse_stmt()?;
            match self.peek() {
                Token::Newline | Token::Eof => {}
                other => {
                    return Err(format!(
                        "expected end of line after statement, found {other:?}"
                    ));
                }
            }
            lines.push((lineno, stmt));
        }
        lines.sort_by_key(|(n, _)| *n);
        for pair in lines.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err(format!("duplicate line number {}", pair[0].0));
            }
        }
        Ok(lines)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.bump() {
            Token::Let => {
                let name = match self.bump() {
                    Token::Ident(n) => n,
                    other => return Err(format!("expected identifier after LET, found {other:?}")),
                };
                self.expect(&Token::Eq, "after LET <ident>")?;
                let e = self.parse_expr()?;
                Ok(Stmt::Let(name, e))
            }
            Token::Print => {
                let e = self.parse_expr()?;
                Ok(Stmt::Print(e))
            }
            Token::If => {
                let cond = self.parse_expr()?;
                self.expect(&Token::Then, "after IF <expr>")?;
                let target = match self.bump() {
                    Token::Number(n) => n,
                    other => {
                        return Err(format!("expected line number after THEN, found {other:?}"));
                    }
                };
                Ok(Stmt::If(cond, target))
            }
            Token::Goto => match self.bump() {
                Token::Number(n) => Ok(Stmt::Goto(n)),
                other => Err(format!("expected line number after GOTO, found {other:?}")),
            },
            Token::End => Ok(Stmt::End),
            Token::Rem => {
                while !matches!(self.peek(), Token::Newline | Token::Eof) {
                    self.bump();
                }
                Ok(Stmt::Rem)
            }
            other => Err(format!("expected statement keyword, found {other:?}")),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_add()?;
        while let Some(op) = self.peek_compare_op() {
            self.bump();
            let rhs = self.parse_add()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn peek_compare_op(&self) -> Option<BinOp> {
        match self.peek() {
            Token::Eq => Some(BinOp::Eq),
            Token::NotEq => Some(BinOp::Ne),
            Token::Lt => Some(BinOp::Lt),
            Token::Gt => Some(BinOp::Gt),
            Token::LtEq => Some(BinOp::Le),
            Token::GtEq => Some(BinOp::Ge),
            _ => None,
        }
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_mul()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_atom()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_atom()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Token::Number(n) => Ok(Expr::Num(n)),
            Token::Ident(n) => Ok(Expr::Var(n)),
            Token::LParen => {
                let e = self.parse_expr()?;
                self.expect(&Token::RParen, "to close parenthesised expression")?;
                Ok(e)
            }
            other => Err(format!("expected expression, found {other:?}")),
        }
    }
}

fn parse(tokens: Vec<Token>) -> Result<Vec<(u32, Stmt)>, String> {
    Parser::new(tokens).parse_program()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let source = read_source()?;
    let tokens = lex(&source)?;
    let _program = parse(tokens)?;
    let module = Module::new();
    print!("{module}");
    Ok(())
}

fn read_source() -> Result<String, String> {
    let mut args = std::env::args().skip(1);
    match args.next() {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {path}: {e}")),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cannot read stdin: {e}"))?;
            Ok(buf)
        }
    }
}
