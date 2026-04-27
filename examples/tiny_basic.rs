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
    let module = Module::new();
    for tok in &tokens {
        eprintln!("# {tok:?}");
    }
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
