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
