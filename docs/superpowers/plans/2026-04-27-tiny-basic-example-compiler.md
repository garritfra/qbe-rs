# Tiny BASIC Example Compiler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `examples/tiny_basic.rs` — a self-contained BASIC-subset compiler (lexer + parser + codegen) that emits QBE IL via the `qbe` crate, addressing [issue #9](https://github.com/garritfra/qbe-rs/issues/9).

**Architecture:** Single Rust file with three pass-style modules (lexer → parser → codegen) sharing simple `enum`-based IRs. Emits a `qbe::Module` whose `Display` impl produces valid QBE IL on stdout. Verified end-to-end by piping output through `qbe` and `cc` and running the resulting binary.

**Tech Stack:** Rust 1.94 (per `rust-toolchain.toml`), the `qbe` crate from this workspace, no extra dependencies. `qbe` and `cc` binaries on PATH for final integration test.

**Spec:** [`docs/superpowers/specs/2026-04-27-tiny-basic-example-compiler-design.md`](../specs/2026-04-27-tiny-basic-example-compiler-design.md).

---

## Conventions used by every task

- Verification commands assume the worktree root as cwd.
- After every code-changing step, the next step is "verify" then "commit". A task's commit lands all the changes for that task as one logical commit.
- The example file `examples/tiny_basic.rs` is built up incrementally; later tasks edit code added by earlier tasks. Each task's snippet is a complete drop-in replacement for the section it touches.
- Use `2>/dev/null` only when explicitly shown; otherwise let stderr surface during verification.

---

## Task 1: Scaffolding — empty example that compiles and prints a placeholder module

**Files:**
- Create: `examples/tiny_basic.rs`

**Goal:** Establish the file, the `main` shape (read source, print module), and confirm it compiles and runs end-to-end with a hardcoded empty module so later tasks have a stable foundation.

- [ ] **Step 1: Create `examples/tiny_basic.rs` with a minimal main**

```rust
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
    let _source = read_source()?;
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
```

- [ ] **Step 2: Build the example**

Run: `cargo build --example tiny_basic`
Expected: clean build, no warnings.

- [ ] **Step 3: Smoke-test with empty stdin**

Run: `echo -n '' | cargo run --quiet --example tiny_basic`
Expected: empty output (an empty `Module` displays as nothing). Exit code 0.

- [ ] **Step 4: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples: scaffold tiny_basic compiler skeleton"
```

---

## Task 2: Lexer

**Files:**
- Modify: `examples/tiny_basic.rs` (append lexer module + invoke from `run`)

**Goal:** Convert the source string into a `Vec<Token>`. After this task, `run` lexes the input and prints the token stream as a debug comment so we can verify visually.

- [ ] **Step 1: Add `Token` enum and `lex` function**

Append above `fn main` (after the doc comment block):

```rust
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
```

- [ ] **Step 2: Wire the lexer into `run` (debug visibility)**

Replace the body of `fn run` with:

```rust
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
```

(stderr — so it does not pollute the IL on stdout. We will remove this in Task 4.)

- [ ] **Step 3: Build and smoke-test the lexer**

Run:
```sh
printf '10 LET X = 5\n20 PRINT X + 1\n30 IF X <> 0 THEN 10\n40 END\n' \
  | cargo run --quiet --example tiny_basic 2>&1 >/dev/null
```

Expected (order matters; `<>` lexed as `NotEq`, `<=` would be `LtEq`):
```
# Number(10)
# Let
# Ident("X")
# Eq
# Number(5)
# Newline
# Number(20)
# Print
# Ident("X")
# Plus
# Number(1)
# Newline
# Number(30)
# If
# Ident("X")
# NotEq
# Number(0)
# Then
# Number(10)
# Newline
# Number(40)
# End
# Newline
# Eof
```

If any token in the sequence above is missing or different, fix the lexer before continuing.

- [ ] **Step 4: Verify error path**

Run: `printf '10 LET ?\n' | cargo run --quiet --example tiny_basic; echo exit=$?`
Expected: `error: unexpected character '?'` on stderr, `exit=1`.

- [ ] **Step 5: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): add lexer"
```

---

## Task 3: Parser — statements without expressions

**Files:**
- Modify: `examples/tiny_basic.rs` (append AST + parser, leave expression parsing as a stub returning `Expr::Num(0)`)

**Goal:** Parse a stream of `Token`s into `Vec<(u32, Stmt)>`, validate line numbers, handle `REM`. Expression parsing is stubbed so we can wire the structure end-to-end and replace the stub in Task 4.

- [ ] **Step 1: Add AST types**

Append above `fn lex`:

```rust
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
```

- [ ] **Step 2: Add the parser**

Append after `fn lex`:

```rust
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
        Ok(Expr::Num(0))
    }
}

fn parse(tokens: Vec<Token>) -> Result<Vec<(u32, Stmt)>, String> {
    Parser::new(tokens).parse_program()
}
```

- [ ] **Step 3: Wire the parser into `run` and dump statements to stderr**

Replace `fn run` body with:

```rust
fn run() -> Result<(), String> {
    let source = read_source()?;
    let tokens = lex(&source)?;
    let program = parse(tokens)?;
    let module = Module::new();
    for (n, stmt) in &program {
        eprintln!("# {n}: {stmt:?}");
    }
    print!("{module}");
    Ok(())
}
```

- [ ] **Step 4: Verify statements parse**

Run:
```sh
printf '20 PRINT 0\n10 LET X = 0\n30 IF 0 THEN 10\n40 GOTO 10\n50 REM hello world\n60 END\n' \
  | cargo run --quiet --example tiny_basic 2>&1 >/dev/null
```

Expected (note the lines are sorted by lineno; expression payloads are still `Num(0)` because of the stub):
```
# 10: Let("X", Num(0))
# 20: Print(Num(0))
# 30: If(Num(0), 10)
# 40: Goto(10)
# 50: Rem
# 60: End
```

- [ ] **Step 5: Verify duplicate-line-number error**

Run: `printf '10 END\n10 END\n' | cargo run --quiet --example tiny_basic; echo exit=$?`
Expected: `error: duplicate line number 10` and `exit=1`.

- [ ] **Step 6: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): add parser scaffold (statements only)"
```

---

## Task 4: Parser — expressions

**Files:**
- Modify: `examples/tiny_basic.rs` (replace stubbed `parse_expr` with real precedence-climbing implementation, drop debug prints)

**Goal:** Implement full expression parsing with three precedence levels (mul/div, add/sub, comparison). Remove the stderr debug prints introduced in earlier tasks.

- [ ] **Step 1: Replace `parse_expr` and add helpers**

Replace the stub `parse_expr` method with:

```rust
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
```

- [ ] **Step 2: Drop the stderr debug prints in `run`**

Replace `fn run` body with:

```rust
fn run() -> Result<(), String> {
    let source = read_source()?;
    let tokens = lex(&source)?;
    let _program = parse(tokens)?;
    let module = Module::new();
    print!("{module}");
    Ok(())
}
```

- [ ] **Step 3: Build and verify nothing fails on the factorial program**

Save the factorial program to a temp file and parse it:

```sh
cat > /tmp/factorial.bas <<'EOF'
10 LET N = 5
20 LET F = 1
30 IF N <= 1 THEN 60
40 LET F = F * N
50 LET N = N - 1
55 GOTO 30
60 PRINT F
70 END
EOF
cargo run --quiet --example tiny_basic /tmp/factorial.bas
```

Expected: empty stdout (no codegen yet), exit code 0.

- [ ] **Step 4: Verify a malformed expression errors**

Run: `printf '10 LET X = 1 +\n' | cargo run --quiet --example tiny_basic; echo exit=$?`
Expected: `error: expected expression, found Newline` and `exit=1`.

- [ ] **Step 5: Verify precedence by inspecting parser output (temporary debug print)**

Temporarily add `eprintln!("{:?}", _program);` after `let _program = parse(tokens)?;` and run:

```sh
printf '10 LET X = 1 + 2 * 3\n' | cargo run --quiet --example tiny_basic 2>&1 >/dev/null
```

Expected: the AST shows `BinOp(Add, Num(1), BinOp(Mul, Num(2), Num(3)))` — i.e., `*` binds tighter than `+`. Then **remove the debug print** before the next step.

- [ ] **Step 6: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): parse arithmetic and comparison expressions"
```

---

## Task 5: Codegen — skeleton (entry block, end_program block, fmt data)

**Files:**
- Modify: `examples/tiny_basic.rs` (add `Codegen` struct that produces an empty-but-valid `main` function and the `fmt_int` data def; wire into `run`)

**Goal:** Produce a syntactically valid QBE module for any (parseable) program: declares all variables, has an `@entry` block jumping to `@end_program`, has `@end_program` returning 0, and adds the `fmt_int` data def. Statements are still ignored — those land in Tasks 6–8.

- [ ] **Step 1: Add the `Codegen` struct and skeleton emission**

Append below `fn parse`:

```rust
use qbe::{DataDef, DataItem, Function, Instr, Linkage, Type, Value};
use std::collections::HashSet;

struct Codegen {
    module: Module,
    #[allow(dead_code)]
    line_set: HashSet<u32>,
    line_order: Vec<u32>,
}

impl Codegen {
    fn new(program: &[(u32, Stmt)]) -> Self {
        let line_order: Vec<u32> = program.iter().map(|(n, _)| *n).collect();
        let line_set: HashSet<u32> = line_order.iter().copied().collect();
        Self {
            module: Module::new(),
            line_set,
            line_order,
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
            main.assign_instr(
                Value::Temporary(v.clone()),
                Type::Long,
                Instr::Alloc4(4),
            );
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

        for (idx, (n, _stmt)) in program.iter().enumerate() {
            main.add_block(Self::line_label(*n));
            // Statements are emitted in later tasks. For now, every line falls
            // through to its successor so the function is still well-formed.
            main.add_instr(Instr::Jmp(self.next_label(idx)));
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
```

Note: `self.line_set` is unused at this point; the `#[allow(dead_code)]`
above keeps clippy quiet. Task 7 removes the attribute when the field is
read.

- [ ] **Step 2: Wire codegen into `run`**

Replace `fn run` body with:

```rust
fn run() -> Result<(), String> {
    let source = read_source()?;
    let tokens = lex(&source)?;
    let program = parse(tokens)?;
    let module = codegen(&program)?;
    print!("{module}");
    Ok(())
}
```

Also drop the now-unused `use qbe::Module;` at the top of the file (the new `use qbe::{...}` line covers it). Final imports at the top of the file should be:

```rust
use qbe::{DataDef, DataItem, Function, Instr, Linkage, Module, Type, Value};
use std::collections::HashSet;
use std::io::Read;
use std::process::ExitCode;
```

(Move all `use` statements to the top of the file, replacing both the original `use qbe::Module;` and the second `use qbe::{...}` block added above. Keep `use std::collections::HashSet;` here too.)

- [ ] **Step 3: Build and verify the skeleton emits valid IL**

Run:
```sh
printf '10 LET X = 0\n20 PRINT X\n30 END\n' | cargo run --quiet --example tiny_basic
```

Expected output (exact match):
```
data $fmt_int = { b "%d\n", b 0 }
export function w $main() {
@entry
	%X =l alloc4 4
	storew 0, %X
	jmp @line_10
@line_10
	jmp @line_20
@line_20
	jmp @line_30
@line_30
	jmp @end_program
@end_program
	ret 0
}

```

- [ ] **Step 4: Verify it round-trips through `qbe`**

Run:
```sh
printf '10 LET X = 0\n20 PRINT X\n30 END\n' | cargo run --quiet --example tiny_basic | qbe -o /tmp/skeleton.s -
echo $?
```

Expected: exit code 0, no stderr output from qbe (a `.s` file is produced; we will run it after Task 9).

- [ ] **Step 5: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): emit codegen skeleton (entry, blocks, end_program)"
```

---

## Task 6: Codegen — `LET`, `PRINT`, expressions

**Files:**
- Modify: `examples/tiny_basic.rs` (add expression lowering, replace placeholder `Jmp` with real statement emission for `LET`, `PRINT`, `Rem`)

**Goal:** Make `LET` and `PRINT` actually generate code. `Rem` becomes a no-op block (still falls through). After this task, `END`-less programs that only use `LET` and `PRINT` produce correct output when piped through `qbe` and `cc`.

- [ ] **Step 1: Add a temp counter to `Codegen`**

In the `Codegen` struct, add a `next_temp: u32` field:

```rust
struct Codegen {
    module: Module,
    line_set: HashSet<u32>,
    line_order: Vec<u32>,
    next_temp: u32,
}
```

(also drop the `#[allow(dead_code)]` on `line_set` if Task 7 has already
been applied; otherwise leave it — it gets removed in Task 7 anyway.)

In `Codegen::new`, initialise `next_temp: 0`:

```rust
        Self {
            module: Module::new(),
            line_set,
            line_order,
            next_temp: 0,
        }
```

- [ ] **Step 2: Add expression lowering and statement emission**

Add the following methods to `impl Codegen` (place just before `fn emit`):

```rust
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
                    BinOp::Eq => Instr::Cmp(Type::Word, qbe::Cmp::Eq, lv, rv),
                    BinOp::Ne => Instr::Cmp(Type::Word, qbe::Cmp::Ne, lv, rv),
                    BinOp::Lt => Instr::Cmp(Type::Word, qbe::Cmp::Slt, lv, rv),
                    BinOp::Gt => Instr::Cmp(Type::Word, qbe::Cmp::Sgt, lv, rv),
                    BinOp::Le => Instr::Cmp(Type::Word, qbe::Cmp::Sle, lv, rv),
                    BinOp::Ge => Instr::Cmp(Type::Word, qbe::Cmp::Sge, lv, rv),
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
                func.add_instr(Instr::Store(
                    Type::Word,
                    Value::Temporary(name.clone()),
                    v,
                ));
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
            Stmt::If(_, _) | Stmt::Goto(_) | Stmt::End => {
                // Implemented in Tasks 7 and 8.
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
        }
        Ok(())
    }
```

- [ ] **Step 3: Replace the placeholder `jmp` in the per-line loop with a `lower_stmt` call**

In `fn emit`, replace this loop:

```rust
        for (idx, (n, _stmt)) in program.iter().enumerate() {
            main.add_block(Self::line_label(*n));
            main.add_instr(Instr::Jmp(self.next_label(idx)));
        }
```

with:

```rust
        for (idx, (n, stmt)) in program.iter().enumerate() {
            main.add_block(Self::line_label(*n));
            let next = self.next_label(idx);
            self.lower_stmt(&mut main, stmt, &next)?;
        }
```

- [ ] **Step 4: Build and inspect output for a `LET` + `PRINT` program**

Run:
```sh
printf '10 LET X = 1 + 2 * 3\n20 PRINT X\n' | cargo run --quiet --example tiny_basic
```

Expected output:
```
data $fmt_int = { b "%d\n", b 0 }
export function w $main() {
@entry
	%X =l alloc4 4
	storew 0, %X
	jmp @line_10
@line_10
	%t0 =w mul 2, 3
	%t1 =w add 1, %t0
	storew %t1, %X
	jmp @line_20
@line_20
	%t2 =w loadw %X
	call $printf(l $fmt_int, ..., w %t2)
	jmp @end_program
@end_program
	ret 0
}

```

- [ ] **Step 5: Verify by running through qbe + cc**

Run:
```sh
printf '10 LET X = 1 + 2 * 3\n20 PRINT X\n' \
  | cargo run --quiet --example tiny_basic \
  | qbe -o /tmp/expr.s -
cc /tmp/expr.s -o /tmp/expr
/tmp/expr
echo exit=$?
```

Expected: prints `7` followed by `exit=0`.

- [ ] **Step 6: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): codegen LET, PRINT, expressions"
```

---

## Task 7: Codegen — `IF`, `GOTO`, target validation

**Files:**
- Modify: `examples/tiny_basic.rs` (extend `lower_stmt` for `If` and `Goto`; remove the `#[allow(dead_code)]` from `line_set`)

**Goal:** Implement conditional and unconditional jumps with line-number validation.

- [ ] **Step 1: Implement `If` and `Goto` in `lower_stmt`**

Replace the placeholder match arm:

```rust
            Stmt::If(_, _) | Stmt::Goto(_) | Stmt::End => {
                // Implemented in Tasks 7 and 8.
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
```

with:

```rust
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
                // Implemented in Task 8.
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
```

- [ ] **Step 2: Drop the `#[allow(dead_code)]` from `line_set`**

Find the field declaration in the `Codegen` struct and remove the attribute:

```rust
    #[allow(dead_code)]
    line_set: HashSet<u32>,
```

becomes

```rust
    line_set: HashSet<u32>,
```

- [ ] **Step 3: Build and inspect IL for an `IF`/`GOTO` program**

Run:
```sh
printf '10 LET X = 1\n20 IF X = 1 THEN 40\n30 PRINT 0\n40 PRINT X\n' \
  | cargo run --quiet --example tiny_basic
```

Expected output (key lines):
```
@line_20
	%t0 =w loadw %X
	%t1 =w ceqw %t0, 1
	jnz %t1, @line_40, @line_30
```

The exact instructions for the other blocks should mirror Task 6's structure.

- [ ] **Step 4: Verify the program runs correctly**

Run:
```sh
printf '10 LET X = 1\n20 IF X = 1 THEN 40\n30 PRINT 0\n40 PRINT X\n' \
  | cargo run --quiet --example tiny_basic \
  | qbe -o /tmp/if.s -
cc /tmp/if.s -o /tmp/if
/tmp/if
echo exit=$?
```

Expected: prints `1` (because the `IF` skipped over `30 PRINT 0`), `exit=0`.

- [ ] **Step 5: Verify a bad target errors at codegen time**

Run: `printf '10 GOTO 999\n' | cargo run --quiet --example tiny_basic; echo exit=$?`
Expected: `error: GOTO target 999 is not a line number` and `exit=1`.

- [ ] **Step 6: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): codegen IF/GOTO with target validation"
```

---

## Task 8: Codegen — `END`

**Files:**
- Modify: `examples/tiny_basic.rs` (single match arm change)

**Goal:** `END` returns 0 directly instead of falling through to `end_program`.

- [ ] **Step 1: Replace the `End` arm in `lower_stmt`**

Find:

```rust
            Stmt::End => {
                // Implemented in Task 8.
                func.add_instr(Instr::Jmp(next_label.to_string()));
            }
```

Replace with:

```rust
            Stmt::End => {
                func.add_instr(Instr::Ret(Some(Value::Const(0))));
            }
```

- [ ] **Step 2: Build and inspect IL**

Run:
```sh
printf '10 PRINT 42\n20 END\n30 PRINT 99\n' | cargo run --quiet --example tiny_basic
```

Expected: the `@line_20` block ends with `ret 0`, and `@line_30` is still present (it is unreachable, which QBE accepts) and falls through to `@end_program` via `jmp`.

- [ ] **Step 3: Verify behaviour**

Run:
```sh
printf '10 PRINT 42\n20 END\n30 PRINT 99\n' \
  | cargo run --quiet --example tiny_basic \
  | qbe -o /tmp/end.s -
cc /tmp/end.s -o /tmp/end
/tmp/end
echo exit=$?
```

Expected: prints `42` only (line 30 is skipped because line 20 returned), `exit=0`.

- [ ] **Step 4: Commit**

```bash
git add examples/tiny_basic.rs
git commit -m "examples(tiny_basic): codegen END as ret 0"
```

---

## Task 9: End-to-end factorial verification

**Files:**
- No new edits (verification only).

**Goal:** Confirm the doc-comment factorial program compiles, links, and prints `120`.

- [ ] **Step 1: Save the factorial program**

Run:
```sh
cat > /tmp/factorial.bas <<'EOF'
10 LET N = 5
20 LET F = 1
30 IF N <= 1 THEN 60
40 LET F = F * N
50 LET N = N - 1
55 GOTO 30
60 PRINT F
70 END
EOF
```

- [ ] **Step 2: Compile and link**

Run:
```sh
cargo run --quiet --example tiny_basic /tmp/factorial.bas | qbe -o /tmp/factorial.s -
cc /tmp/factorial.s -o /tmp/factorial
```

Expected: both commands exit 0 with no stderr output.

- [ ] **Step 3: Run the binary**

Run:
```sh
/tmp/factorial
echo exit=$?
```

Expected: `120` followed by `exit=0`.

- [ ] **Step 4: Run all crate tests to confirm we did not break the library**

Run: `cargo test --quiet`
Expected: all tests pass, no warnings introduced by the example.

- [ ] **Step 5: Run clippy on the example**

Run: `cargo clippy --example tiny_basic -- -D warnings`
Expected: clean exit. If clippy lints fire, fix them inline (typical fixes: unused `mut`, redundant clones — but the design above tries to avoid these).

- [ ] **Step 6: No commit — this task is verification only.**

---

## Task 10: Documentation — README and CHANGELOG

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`

**Goal:** Point new users at the new example and record the addition.

- [ ] **Step 1: Update `README.md`**

In `README.md`, replace the line:

```
If you don't know where to get started, check out the `hello_world` example in
the `examples/` directory.
```

with:

```
If you don't know where to get started, check out the examples in the
`examples/` directory:

- `hello_world` — the smallest possible use of the API; build a `Module` by
  hand and print it.
- `tiny_basic` — an end-to-end compiler for a BASIC subset (lexer, parser,
  codegen). Run it with
  `cargo run --example tiny_basic path/to/program.bas | qbe -o out.s -`.
```

- [ ] **Step 2: Update `CHANGELOG.md`**

Read the first ~30 lines of `CHANGELOG.md` to find the right place. If there is no `## Unreleased` section yet, add one at the top under the `# Changelog` (or equivalent) heading:

```markdown
## Unreleased

### Added

- New example `tiny_basic`: a BASIC-subset compiler that demonstrates an
  end-to-end source-to-IL pipeline (lexer, parser, codegen). Closes #9.
```

If an `## Unreleased` section already exists, append the bullet under its `### Added` subsection (creating the subsection if needed).

- [ ] **Step 3: Verify README renders sensibly**

Run: `cat README.md | head -30`
Expected: the new example list appears intact with proper Markdown formatting.

- [ ] **Step 4: Commit**

```bash
git add README.md CHANGELOG.md
git commit -m "docs: announce tiny_basic example, update changelog"
```

---

## Final verification (post-Task 10)

- [ ] **Run the full test suite once more:** `cargo test --quiet` — expect green.
- [ ] **Run clippy across the whole workspace:** `cargo clippy --all-targets -- -D warnings` — expect green.
- [ ] **Confirm the issue is addressed:** the `examples/` directory now contains a non-trivial compiler example, satisfying issue #9.
