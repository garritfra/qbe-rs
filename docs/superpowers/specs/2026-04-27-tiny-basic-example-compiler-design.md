# Tiny BASIC example compiler

Addresses [issue #9](https://github.com/garritfra/qbe-rs/issues/9): "In the
`example/` directory, we should write a very simple compiler to demonstrate how
to use this crate."

The existing `examples/hello_world.rs` builds a QBE IL module by hand. This new
example takes the next step: a real source language → AST → QBE IL pipeline,
small enough to fit in a single file but large enough to exercise the interesting
parts of the crate (variables, branches, loops, calls, data definitions).

## Source language: Tiny BASIC

Each source line has the form `<lineno> <statement>`. Lines may appear in any
order in the source; the parser sorts them by `lineno`. Blank lines and lines
beginning with `REM` are comments.

Statements:

- `LET <IDENT> = <expr>` — assignment.
- `PRINT <expr>` — print integer expression followed by newline.
- `IF <expr> THEN <lineno>` — if `<expr>` is non-zero, jump to that line;
  otherwise fall through to the next line in source order.
- `GOTO <lineno>` — unconditional jump.
- `END` — terminate the program (returns 0).
- `REM <text>` — comment; parsed and discarded.

Lexical rules:

- Identifiers match `[A-Z][A-Z0-9]*`.
- Integer literals are non-negative decimal; unary minus is not in the grammar
  (negative values are produced by `0 - X`).
- Whitespace is space and tab; line breaks separate statements.
- Source is case-sensitive; all keywords are uppercase.

Expressions:

- Atoms: integer literal, identifier, parenthesised expression.
- Binary operators with standard precedence (highest first):
  1. `*`, `/`
  2. `+`, `-`
  3. `=`, `<>`, `<`, `>`, `<=`, `>=` (yield `1` or `0`)
- All values are 32-bit signed integers (`Type::Word`).

Example program (factorial of 5):

```
10 LET N = 5
20 LET F = 1
30 IF N <= 1 THEN 60
40 LET F = F * N
50 LET N = N - 1
55 GOTO 30
60 PRINT F
70 END
```

Expected output: `120`.

## Architecture

Single file `examples/tiny_basic.rs`, no external dependencies. Three small
modules grouped by responsibility, plus `main`.

### Module 1: Lexer

Converts `&str` source into `Vec<Token>`.

```rust
enum Token {
    // Keywords
    Let, Print, If, Then, Goto, End, Rem,
    // Atoms
    Ident(String),
    Number(u32),
    // Operators
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    Plus, Minus, Star, Slash,
    LParen, RParen,
    // Structure
    Newline,
    Eof,
}
```

The lexer is a straightforward character cursor:

- Skip space and tab.
- A newline emits `Newline` and advances.
- The keyword `REM` is emitted as a token; the parser, not the lexer, is
  responsible for discarding the rest of the line. (See parser.)
- Digits accumulate into `Number`.
- Letters accumulate into an identifier; if the result matches a keyword,
  emit the keyword token, otherwise `Ident`.
- Operator characters are looked up directly; `<` peeks ahead for `<=` and
  `<>`, `>` peeks ahead for `>=`.
- Any other character is a lex error.

### Module 2: Parser

Converts `Vec<Token>` into `Vec<(u32, Stmt)>` sorted by line number.

```rust
enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, u32),
    Goto(u32),
    End,
    Rem,
}

enum Expr {
    Num(u32),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

enum BinOp { Add, Sub, Mul, Div, Eq, Ne, Lt, Gt, Le, Ge }
```

The parser is recursive-descent. Top level loops over `Newline`-separated
statements, each beginning with a line number. When the keyword after the
line number is `REM`, the parser drops every remaining token on that line
until the next `Newline`/`Eof` and produces `Stmt::Rem`. Expressions use
precedence climbing with three levels (multiplicative, additive,
comparison). After parsing all statements, the list is sorted by line
number; duplicate line numbers are an error.

### Module 3: Codegen

Converts `&[(u32, Stmt)]` into a `qbe::Module`.

The generated module contains:

- One private `DataDef` `fmt_int = { b "%d\n", b 0 }`, used by every `PRINT`.
- One exported `Function` `main` returning `Type::Word`.

Inside `main`:

1. **Entry block `@entry`** — for each variable name encountered anywhere in
   the program, emit `%<name> =l alloc4 4` followed by `storew 0, %<name>`.
   This zero-initialises every variable, matching classic BASIC semantics.
   The block ends with `jmp @line_<first lineno>` (or `jmp @end_program` if
   the program is empty).

2. **One block per source line** labelled `@line_<n>`. The codegen first
   builds a `HashSet<u32>` of all line numbers in the program so jump
   targets can be validated. The body is then generated per statement type:

   - `LET X = e`: lower `e` into a temporary `%t`, then `storew %t, %X`.
   - `PRINT e`: lower `e` into `%t`, then
     `call $printf(l $fmt_int, ..., w %t)` with the variadic index set to 1
     (matching `examples/hello_world.rs`).
   - `IF e THEN N`: lower `e` into `%t`, then
     `jnz %t, @line_N, @<next-line-label>`. If `N` is not in the
     line-number set, codegen returns an error.
   - `GOTO N`: `jmp @line_N`. If `N` is not in the line-number set, codegen
     returns an error.
   - `END`: `ret 0`.
   - `Rem`: emits no instructions.

   If a block does not already end in a jump (`LET`, `PRINT`, `Rem`), append
   a fall-through `jmp @<next-line-label>`.

3. **Final block `@end_program`** — `ret 0`. This is the fall-through target
   when execution runs off the last source line.

Expression lowering walks the `Expr` tree, emitting one QBE temporary per
sub-expression. Temporaries are named `%t<counter>` with a monotonic counter
maintained by the codegen struct. Comparisons use `Instr::Cmp(Type::Word, ...)`
which already yields 0/1.

### `main`

```
fn main() {
    let source = match args[1] {
        Some(path) => fs::read_to_string(path),
        None => read_stdin(),
    };
    let tokens = lex(&source)?;
    let program = parse(tokens)?;
    let module = codegen(&program);
    print!("{module}");
}
```

On any error, print `error: <msg>` to stderr and exit with code 1.

## Data flow

```
source string
  -> Lexer  -> Vec<Token>
  -> Parser -> Vec<(u32, Stmt)>  (sorted, validated)
  -> Codegen -> qbe::Module
  -> Display -> stdout
```

Each stage takes ownership of its input and produces a self-contained output;
no shared mutable state crosses stage boundaries. This makes each stage
testable in isolation and keeps the example readable top-to-bottom.

## Error handling

Pragmatic for an example. Every fallible function returns `Result<T, String>`;
errors are formatted with `format!` and bubbled to `main`. The first error
aborts the run with `eprintln!("error: {e}"); std::process::exit(1)`.

No source spans, no error recovery, no `thiserror` dep. Errors covered:

- Lexer: unexpected character.
- Parser: unexpected token, missing line number, missing `THEN`, malformed
  expression, duplicate line numbers.
- Codegen-time validation: `GOTO`/`IF...THEN` referencing a line number
  that does not appear in the program.

## Testing

- Inline doc-comment at the top of `tiny_basic.rs` showing the factorial
  program above and the expected output (so readers see the end-to-end use
  immediately).
- No new unit tests in the example. Examples are demo code; library
  correctness is covered by `src/tests.rs`. Manual verification: pipe the
  factorial program through the example and through `qbe`, confirm `120`.

## Documentation updates

- `README.md`: extend the "Getting Started" paragraph to mention `tiny_basic`
  alongside `hello_world`, e.g.
  > "If you don't know where to get started, check out the `hello_world`
  > example for the bare API surface, or `tiny_basic` for an end-to-end
  > compile-from-source pipeline."
- `CHANGELOG.md`: add an entry under an "Unreleased" section noting the new
  example.

## Out of scope

Explicitly not included to keep the example focused:

- User-defined functions / `GOSUB` / `RETURN`.
- `INPUT` and `FOR ... NEXT`.
- String literals and multi-arg `PRINT`.
- Floating-point.
- Negative integer literals (use `0 - X`).
- Source diagnostics with line/column.
- Optimisation passes.

If a future contributor wants any of these, they can extend the example or
add a sibling example.

## Trade-offs considered

**Single file vs. submodule.** A single `.rs` keeps the example trivial to
discover (`cargo run --example tiny_basic`) and matches `hello_world.rs`'s
style. Splitting would require an `[[example]]` manifest entry and an
`examples/tiny_basic/` directory; not worth it at ~400 LOC.

**Stack slots for variables vs. SSA temporaries.** Stack slots win. BASIC
variables are mutable and live across blocks, so SSA would require phi
nodes at every control-flow join. Stack slots match what `clang -O0` emits
for local variables, keep the codegen straightforward, and demonstrate
`Alloc4` / `Store` / `Load` — interesting parts of the API that
`hello_world.rs` does not exercise.

**Line-numbered control flow vs. structured blocks.** BASIC's
line-number-as-label model maps almost 1:1 onto QBE's block labels and
demonstrates `Jmp` and `Jnz` naturally. A structured `if`/`while` source
language would require the codegen to invent fresh labels, which is also
fine but adds a layer of mechanical bookkeeping that obscures the QBE API.
