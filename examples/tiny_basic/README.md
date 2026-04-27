# Tiny BASIC

A small but complete compiler example: a BASIC subset → QBE IL → native binary.
The compiler itself lives in [`main.rs`](main.rs) (lexer + parser + codegen,
~600 lines, no extra dependencies). This README shows how to run it.

## Prerequisites

- A recent Rust toolchain (matches `rust-toolchain.toml` at the repo root).
- [`qbe`](https://c9x.me/compile/) on your `PATH`.
  - macOS: `brew install qbe`
  - Linux: `apt install qbe` or build from source.
- A C compiler (`cc`), used only to link the assembly QBE produces.

## Run a sample program

From the repo root:

```sh
cargo run --example tiny_basic examples/tiny_basic/factorial.bas \
  | qbe -o /tmp/out.s - \
  && cc /tmp/out.s -o /tmp/program \
  && /tmp/program
```

Expected output: `120` (factorial of 5).

The same recipe works for the other sample programs:

| Program | What it prints |
|---|---|
| [`hello.bas`](hello.bas) | `42` (smallest possible Tiny BASIC program) |
| [`factorial.bas`](factorial.bas) | `120` (5! via `IF`/`GOTO` loop) |
| [`fibonacci.bas`](fibonacci.bas) | `55` (10th Fibonacci number) |

Just substitute the source path in the command above.

## What you get for free

If you only want to see the generated QBE IL without running it:

```sh
cargo run --example tiny_basic examples/tiny_basic/factorial.bas
```

The compiler reads source from the path you pass on the command line, or from
stdin if you pass nothing. Errors go to stderr with a `error: <msg>` prefix and
the process exits 1.

## Tiny BASIC language

Every line is `<lineno> <statement>`; lines may appear in any order and are
sorted by `<lineno>` before codegen. All values are 32-bit signed integers.

Statements:

- `LET <IDENT> = <expr>` — assignment.
- `PRINT <expr>` — print integer expression followed by newline.
- `IF <expr> THEN <lineno>` — jump if `<expr>` is non-zero, otherwise fall
  through.
- `GOTO <lineno>` — unconditional jump.
- `END` — return 0.
- `REM <text>` — comment, ignored.

Identifiers are uppercase: `[A-Z][A-Z0-9]*`. Operators have standard
precedence: `*` `/` bind tightest, then `+` `-`, then comparisons (`=`, `<>`,
`<`, `>`, `<=`, `>=`). Comparisons yield `1` (true) or `0` (false). Negative
literals aren't part of the grammar; write `0 - X`.

That's the whole language. See `factorial.bas` for the canonical control-flow
example.
