//! Tiny BASIC compiler example.
//!
//! Compiles a minimal BASIC subset to QBE IL. Reads source from a file path
//! given as the first CLI argument, or from stdin when no argument is given.
//! Writes QBE IL to stdout.
//!
//! Supported statements: `LET`, `PRINT`, `IF ... THEN <line>`, `GOTO`, `END`,
//! `REM`. All values are 32-bit signed integers. See `README.md` next to this
//! file for the language reference and ready-to-run sample programs.
//!
//! Quick start (from the repo root):
//!
//! ```sh
//! cargo run --example tiny_basic examples/tiny_basic/factorial.bas \
//!   | qbe -o /tmp/out.s - \
//!   && cc /tmp/out.s -o /tmp/program \
//!   && /tmp/program
//! ```
//!
//! Expected output: `120` (factorial of 5).

use qbe::{Cmp, DataDef, DataItem, Function, Instr, Linkage, Module, Type, Value};
use std::collections::HashSet;
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

#[derive(Debug, Clone)]
enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, u32),
    Goto(u32),
    End,
    Rem,
}

#[derive(Debug, Clone)]
enum Expr {
    Num(u32),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

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
            Token::Rem => Ok(Stmt::Rem),
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

struct Codegen {
    module: Module,
    line_set: HashSet<u32>,
    line_order: Vec<u32>,
    next_temp: u32,
}

impl Codegen {
    fn new(program: &[(u32, Stmt)]) -> Self {
        let line_order: Vec<u32> = program.iter().map(|(n, _)| *n).collect();
        let line_set: HashSet<u32> = line_order.iter().copied().collect();
        Self {
            module: Module::new(),
            line_set,
            line_order,
            next_temp: 0,
        }
    }

    fn line_label(n: u32) -> String {
        format!("line_{n}")
    }

    fn next_label(&self, idx: usize) -> String {
        match self.line_order.get(idx + 1) {
            Some(n) => Self::line_label(*n),
            None => "end_program".to_string(),
        }
    }

    fn fresh_temp(&mut self) -> Value {
        let v = Value::Temporary(format!("t{}", self.next_temp));
        self.next_temp += 1;
        v
    }

    fn lower_expr(&mut self, func: &mut Function, e: &Expr) -> Value {
        match e {
            Expr::Num(n) => Value::Const(*n as u64),
            Expr::Var(name) => {
                let dest = self.fresh_temp();
                func.assign_instr(
                    dest.clone(),
                    Type::Word,
                    Instr::Load(Type::Word, Value::Temporary(name.clone())),
                );
                dest
            }
            Expr::BinOp(op, l, r) => {
                let lv = self.lower_expr(func, l);
                let rv = self.lower_expr(func, r);
                let dest = self.fresh_temp();
                let instr = match op {
                    BinOp::Add => Instr::Add(lv, rv),
                    BinOp::Sub => Instr::Sub(lv, rv),
                    BinOp::Mul => Instr::Mul(lv, rv),
                    BinOp::Div => Instr::Div(lv, rv),
                    BinOp::Eq => Instr::Cmp(Type::Word, Cmp::Eq, lv, rv),
                    BinOp::Ne => Instr::Cmp(Type::Word, Cmp::Ne, lv, rv),
                    BinOp::Lt => Instr::Cmp(Type::Word, Cmp::Slt, lv, rv),
                    BinOp::Gt => Instr::Cmp(Type::Word, Cmp::Sgt, lv, rv),
                    BinOp::Le => Instr::Cmp(Type::Word, Cmp::Sle, lv, rv),
                    BinOp::Ge => Instr::Cmp(Type::Word, Cmp::Sge, lv, rv),
                };
                func.assign_instr(dest.clone(), Type::Word, instr);
                dest
            }
        }
    }

    fn lower_stmt(
        &mut self,
        func: &mut Function,
        stmt: &Stmt,
        next_label: &str,
    ) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, e) => {
                let v = self.lower_expr(func, e);
                func.add_instr(Instr::Store(Type::Word, Value::Temporary(name.clone()), v));
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
            Stmt::Print(e) => {
                let v = self.lower_expr(func, e);
                func.add_instr(Instr::Call(
                    "printf".to_string(),
                    vec![
                        (Type::Long, Value::Global("fmt_int".to_string())),
                        (Type::Word, v),
                    ],
                    Some(1),
                ));
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
            Stmt::Rem => {
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
            Stmt::If(cond, target) => {
                if !self.line_set.contains(target) {
                    return Err(format!("IF...THEN target {target} is not a line number"));
                }
                let v = self.lower_expr(func, cond);
                func.add_instr(Instr::Jnz(
                    v,
                    Self::line_label(*target),
                    next_label.to_string(),
                ));
            }
            Stmt::Goto(target) => {
                if !self.line_set.contains(target) {
                    return Err(format!("GOTO target {target} is not a line number"));
                }
                func.add_instr(Instr::Jmp(Self::line_label(*target)));
            }
            Stmt::End => {
                func.add_instr(Instr::Ret(Some(Value::Const(0))));
            }
        }
        Ok(())
    }

    fn emit(mut self, program: &[(u32, Stmt)]) -> Result<Module, String> {
        self.module.add_data(DataDef::new(
            Linkage::private(),
            "fmt_int",
            None,
            vec![
                (Type::Byte, DataItem::Str("%d\\n".to_string())),
                (Type::Byte, DataItem::Const(0)),
            ],
        ));

        let mut main = Function::new(Linkage::public(), "main", Vec::new(), Some(Type::Word));

        let vars = collect_vars(program);

        main.add_block("entry");
        for v in &vars {
            main.assign_instr(Value::Temporary(v.clone()), Type::Long, Instr::Alloc4(4));
            main.add_instr(Instr::Store(
                Type::Word,
                Value::Temporary(v.clone()),
                Value::Const(0),
            ));
        }
        let first_label = match self.line_order.first() {
            Some(n) => Self::line_label(*n),
            None => "end_program".to_string(),
        };
        main.add_instr(Instr::Jmp(first_label));

        for (idx, (n, stmt)) in program.iter().enumerate() {
            main.add_block(Self::line_label(*n));
            let next = self.next_label(idx);
            self.lower_stmt(&mut main, stmt, &next)?;
        }

        main.add_block("end_program");
        main.add_instr(Instr::Ret(Some(Value::Const(0))));

        self.module.add_function(main);
        Ok(self.module)
    }
}

fn collect_vars(program: &[(u32, Stmt)]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut order: Vec<String> = Vec::new();
    for (_, stmt) in program {
        match stmt {
            Stmt::Let(name, e) => {
                if seen.insert(name.clone()) {
                    order.push(name.clone());
                }
                collect_expr_vars(e, &mut seen, &mut order);
            }
            Stmt::Print(e) | Stmt::If(e, _) => collect_expr_vars(e, &mut seen, &mut order),
            Stmt::Goto(_) | Stmt::End | Stmt::Rem => {}
        }
    }
    order
}

fn collect_expr_vars(e: &Expr, seen: &mut HashSet<String>, order: &mut Vec<String>) {
    match e {
        Expr::Num(_) => {}
        Expr::Var(n) => {
            if seen.insert(n.clone()) {
                order.push(n.clone());
            }
        }
        Expr::BinOp(_, l, r) => {
            collect_expr_vars(l, seen, order);
            collect_expr_vars(r, seen, order);
        }
    }
}

fn codegen(program: &[(u32, Stmt)]) -> Result<Module, String> {
    Codegen::new(program).emit(program)
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
    let program = parse(tokens)?;
    let module = codegen(&program)?;
    print!("{module}");
    Ok(())
}

fn read_source() -> Result<String, String> {
    let mut args = std::env::args().skip(1);
    match args.next() {
        Some(path) => {
            std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))
        }
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cannot read stdin: {e}"))?;
            Ok(buf)
        }
    }
}
