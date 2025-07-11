// Copyright 2022 Garrit Franke
// Copyright 2021 Alexey Yerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # QBE Rust
//!
//! A Rust library for programmatically generating QBE Intermediate Language code.
//!
//! [QBE](https://c9x.me/compile/) is a compiler backend that transforms simple intermediate
//! representation (IR) into executable machine code. This library provides Rust data structures
//! and functions to generate valid QBE IL.
//!
//! ## Basic Example
//!
//! ```rust
//! use qbe::{Module, Function, Linkage, Type, Value, Instr};
//!
//! // Create a new module
//! let mut module = Module::new();
//!
//! // Add a simple function that returns the sum of two integers
//! let mut func = Function::new(
//!     Linkage::public(),
//!     "add",
//!     vec![
//!         (Type::Word, Value::Temporary("a".to_string())),
//!         (Type::Word, Value::Temporary("b".to_string())),
//!     ],
//!     Some(Type::Word),
//! );
//!
//! // Add a block to the function
//! let mut block = func.add_block("start");
//!
//! // Add two arguments and store result in "sum"
//! block.assign_instr(
//!     Value::Temporary("sum".to_string()),
//!     Type::Word,
//!     Instr::Add(
//!         Value::Temporary("a".to_string()),
//!         Value::Temporary("b".to_string()),
//!     ),
//! );
//!
//! // Return the sum
//! block.add_instr(Instr::Ret(Some(Value::Temporary("sum".to_string()))));
//!
//! // Add the function to the module
//! module.add_function(func);
//!
//! // Generate QBE IL code
//! println!("{}", module);
//! ```
//!
//! This generates the following QBE IL:
//! ```ssa
//! export function w $add(w %a, w %b) {
//! @start
//!     %sum =w add %a, %b
//!     ret %sum
//! }
//! ```

use std::fmt;

#[cfg(test)]
mod tests;

/// QBE comparison operations used in conditional instructions.
///
/// The result of a comparison is 1 if the condition is true, and 0 if false.
///
/// # Examples
///
/// ```rust
/// use qbe::{Cmp, Instr, Type, Value};
///
/// // Compare if %a is less than %b (signed comparison)
/// let slt_instr = Instr::Cmp(
///     Type::Word,
///     Cmp::Slt,
///     Value::Temporary("a".to_string()),
///     Value::Temporary("b".to_string()),
/// );
///
/// // Check if two values are equal
/// let eq_instr = Instr::Cmp(
///     Type::Word,
///     Cmp::Eq,
///     Value::Temporary("x".to_string()),
///     Value::Const(0),
/// );
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub enum Cmp {
    /// Returns 1 if first value is less than second, respecting signedness
    Slt,
    /// Returns 1 if first value is less than or equal to second, respecting signedness
    Sle,
    /// Returns 1 if first value is greater than second, respecting signedness
    Sgt,
    /// Returns 1 if first value is greater than or equal to second, respecting signedness
    Sge,
    /// Returns 1 if values are equal
    Eq,
    /// Returns 1 if values are not equal
    Ne,
    /// Returns 1 if both operands are not NaN (ordered comparison)
    O,
    /// Returns 1 if at least one operand is NaN (unordered comparison)
    Uo,
    /// Returns 1 if first value is less than second, unsigned comparison
    Ult,
    /// Returns 1 if first value is less than or equal to second, unsigned comparison
    Ule,
    /// Returns 1 if first value is greater than second, unsigned comparison
    Ugt,
    /// Returns 1 if first value is greater than or equal to second, unsigned comparison
    Uge,
}

/// QBE instructions representing operations in the intermediate language.
///
/// # Examples
///
/// ## Arithmetic Operations
/// ```rust
/// use qbe::{Instr, Value};
///
/// // Addition: %result = %a + %b
/// let add = Instr::Add(
///     Value::Temporary("a".to_string()),
///     Value::Temporary("b".to_string()),
/// );
///
/// // Multiplication: %result = %x * 5
/// let mul = Instr::Mul(
///     Value::Temporary("x".to_string()),
///     Value::Const(5),
/// );
/// ```
///
/// ## Memory Operations
/// ```rust
/// use qbe::{Instr, Type, Value};
///
/// // Allocate 8 bytes on the stack with 8-byte alignment
/// let alloc = Instr::Alloc8(8);
///
/// // Store a word to memory: store %value, %ptr
/// let store = Instr::Store(
///     Type::Word,
///     Value::Temporary("ptr".to_string()),
///     Value::Temporary("value".to_string()),
/// );
///
/// // Load a word from memory: %result = load %ptr
/// let load = Instr::Load(
///     Type::Word,
///     Value::Temporary("ptr".to_string()),
/// );
/// ```
///
/// ## Control Flow
/// ```rust
/// use qbe::{Instr, Value};
///
/// // Conditional jump based on %condition
/// let branch = Instr::Jnz(
///     Value::Temporary("condition".to_string()),
///     "true_branch".to_string(),
///     "false_branch".to_string(),
/// );
///
/// // Return a value from a function
/// let ret = Instr::Ret(Some(Value::Temporary("result".to_string())));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Instr<'a> {
    /// Adds values of two temporaries together
    Add(Value, Value),
    /// Subtracts the second value from the first one
    Sub(Value, Value),
    /// Multiplies values of two temporaries
    Mul(Value, Value),
    /// Divides the first value by the second one
    Div(Value, Value),
    /// Returns a remainder from division
    Rem(Value, Value),
    /// Performs a comparion between values
    Cmp(Type<'a>, Cmp, Value, Value),
    /// Performs a bitwise AND on values
    And(Value, Value),
    /// Performs a bitwise OR on values
    Or(Value, Value),
    /// Copies either a temporary or a literal value
    Copy(Value),
    /// Return from a function, optionally with a value
    Ret(Option<Value>),
    /// Jumps to first label if a value is nonzero or to the second one otherwise
    Jnz(Value, String, String),
    /// Unconditionally jumps to a label
    Jmp(String),
    /// Calls a function
    Call(String, Vec<(Type<'a>, Value)>, Option<u64>),
    /// Allocates a 4-byte aligned area on the stack
    Alloc4(u32),
    /// Allocates a 8-byte aligned area on the stack
    Alloc8(u64),
    /// Allocates a 16-byte aligned area on the stack
    Alloc16(u128),
    /// Stores a value into memory pointed to by destination.
    /// `(type, destination, value)`
    Store(Type<'a>, Value, Value),
    /// Loads a value from memory pointed to by source
    /// `(type, source)`
    Load(Type<'a>, Value),
    /// `(source, destination, n)`
    ///
    /// Copy `n` bytes from the source address to the destination address.
    ///
    /// n must be a constant value.
    ///
    /// ## Minimum supported QBE version
    /// `1.1`
    Blit(Value, Value, u64),

    /// Debug file.
    DbgFile(String),
    /// Debug line.
    ///
    /// Takes line number and an optional column.
    DbgLoc(u64, Option<u64>),

    // Unsigned arithmetic
    /// Performs unsigned division of the first value by the second one
    Udiv(Value, Value),
    /// Returns the remainder from unsigned division
    Urem(Value, Value),

    // Shifts
    /// Shift arithmetic right (preserves sign)
    Sar(Value, Value),
    /// Shift logical right (fills with zeros)
    Shr(Value, Value),
    /// Shift left (fills with zeros)
    Shl(Value, Value),

    // Type conversions
    /// Cast between integer and floating point of the same width
    Cast(Value),

    // Extension operations
    /// Sign-extends a word to a long
    Extsw(Value),
    /// Zero-extends a word to a long
    Extuw(Value),
    /// Sign-extends a halfword to a word or long
    Extsh(Value),
    /// Zero-extends a halfword to a word or long
    Extuh(Value),
    /// Sign-extends a byte to a word or long
    Extsb(Value),
    /// Zero-extends a byte to a word or long
    Extub(Value),
    /// Extends a single-precision float to double-precision
    Exts(Value),
    /// Truncates a double-precision float to single-precision
    Truncd(Value),

    // Float-integer conversions
    /// Converts a single-precision float to a signed integer
    Stosi(Value),
    /// Converts a single-precision float to an unsigned integer
    Stoui(Value),
    /// Converts a double-precision float to a signed integer
    Dtosi(Value),
    /// Converts a double-precision float to an unsigned integer
    Dtoui(Value),
    /// Converts a signed word to a float
    Swtof(Value),
    /// Converts an unsigned word to a float
    Uwtof(Value),
    /// Converts a signed long to a float
    Sltof(Value),
    /// Converts an unsigned long to a float
    Ultof(Value),

    // Variadic function support
    /// Initializes a variable argument list
    Vastart(Value),
    /// Fetches the next argument from a variable argument list
    Vaarg(Type<'a>, Value),

    // Phi instruction
    /// Selects value based on the control flow path into a block.
    Phi(String, Value, String, Value),

    // Program termination
    /// Terminates the program with an error
    Hlt,
}

impl fmt::Display for Instr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add(lhs, rhs) => write!(f, "add {lhs}, {rhs}"),
            Self::Sub(lhs, rhs) => write!(f, "sub {lhs}, {rhs}"),
            Self::Mul(lhs, rhs) => write!(f, "mul {lhs}, {rhs}"),
            Self::Div(lhs, rhs) => write!(f, "div {lhs}, {rhs}"),
            Self::Rem(lhs, rhs) => write!(f, "rem {lhs}, {rhs}"),
            Self::Cmp(ty, cmp, lhs, rhs) => {
                assert!(
                    !matches!(ty, Type::Aggregate(_)),
                    "Cannot compare aggregate types"
                );

                write!(
                    f,
                    "c{}{} {}, {}",
                    match cmp {
                        Cmp::Slt => "slt",
                        Cmp::Sle => "sle",
                        Cmp::Sgt => "sgt",
                        Cmp::Sge => "sge",
                        Cmp::Eq => "eq",
                        Cmp::Ne => "ne",
                        Cmp::O => "o",
                        Cmp::Uo => "uo",
                        Cmp::Ult => "ult",
                        Cmp::Ule => "ule",
                        Cmp::Ugt => "ugt",
                        Cmp::Uge => "uge",
                    },
                    ty,
                    lhs,
                    rhs,
                )
            }
            Self::And(lhs, rhs) => write!(f, "and {lhs}, {rhs}"),
            Self::Or(lhs, rhs) => write!(f, "or {lhs}, {rhs}"),
            Self::Copy(val) => write!(f, "copy {val}"),
            Self::Ret(val) => match val {
                Some(val) => write!(f, "ret {val}"),
                None => write!(f, "ret"),
            },
            Self::DbgFile(val) => write!(f, r#"dbgfile "{val}""#),
            Self::DbgLoc(lineno, column) => match column {
                Some(val) => write!(f, "dbgloc {lineno}, {val}"),
                None => write!(f, "dbgloc {lineno}"),
            },
            Self::Jnz(val, if_nonzero, if_zero) => {
                write!(f, "jnz {val}, @{if_nonzero}, @{if_zero}")
            }
            Self::Jmp(label) => write!(f, "jmp @{label}"),
            Self::Call(name, args, opt_variadic_i) => {
                let mut args_fmt = args
                    .iter()
                    .map(|(ty, temp)| format!("{ty} {temp}"))
                    .collect::<Vec<String>>();
                if let Some(i) = *opt_variadic_i {
                    args_fmt.insert(i as usize, "...".to_string());
                }

                write!(f, "call ${}({})", name, args_fmt.join(", "),)
            }
            Self::Alloc4(size) => write!(f, "alloc4 {size}"),
            Self::Alloc8(size) => write!(f, "alloc8 {size}"),
            Self::Alloc16(size) => write!(f, "alloc16 {size}"),
            Self::Store(ty, dest, value) => {
                if matches!(ty, Type::Aggregate(_)) {
                    unimplemented!("Store to an aggregate type");
                }

                write!(f, "store{ty} {value}, {dest}")
            }
            Self::Load(ty, src) => {
                if matches!(ty, Type::Aggregate(_)) {
                    unimplemented!("Load aggregate type");
                }

                write!(f, "load{ty} {src}")
            }
            Self::Blit(src, dst, n) => write!(f, "blit {src}, {dst}, {n}"),
            Self::Udiv(lhs, rhs) => write!(f, "udiv {lhs}, {rhs}"),
            Self::Urem(lhs, rhs) => write!(f, "urem {lhs}, {rhs}"),
            Self::Sar(lhs, rhs) => write!(f, "sar {lhs}, {rhs}"),
            Self::Shr(lhs, rhs) => write!(f, "shr {lhs}, {rhs}"),
            Self::Shl(lhs, rhs) => write!(f, "shl {lhs}, {rhs}"),
            Self::Cast(val) => write!(f, "cast {val}"),
            Self::Extsw(val) => write!(f, "extsw {val}"),
            Self::Extuw(val) => write!(f, "extuw {val}"),
            Self::Extsh(val) => write!(f, "extsh {val}"),
            Self::Extuh(val) => write!(f, "extuh {val}"),
            Self::Extsb(val) => write!(f, "extsb {val}"),
            Self::Extub(val) => write!(f, "extub {val}"),
            Self::Exts(val) => write!(f, "exts {val}"),
            Self::Truncd(val) => write!(f, "truncd {val}"),
            Self::Stosi(val) => write!(f, "stosi {val}"),
            Self::Stoui(val) => write!(f, "stoui {val}"),
            Self::Dtosi(val) => write!(f, "dtosi {val}"),
            Self::Dtoui(val) => write!(f, "dtoui {val}"),
            Self::Swtof(val) => write!(f, "swtof {val}"),
            Self::Uwtof(val) => write!(f, "uwtof {val}"),
            Self::Sltof(val) => write!(f, "sltof {val}"),
            Self::Ultof(val) => write!(f, "ultof {val}"),
            Self::Vastart(val) => write!(f, "vastart {val}"),
            Self::Vaarg(ty, val) => write!(f, "vaarg{ty} {val}"),
            Self::Phi(label_1, val_if_label_1, label_2, val_if_label_2) => {
                write!(
                    f,
                    "phi @{label_1} {val_if_label_1}, @{label_2} {val_if_label_2}"
                )
            }
            Self::Hlt => write!(f, "hlt"),
        }
    }
}

/// QBE types used to specify the size and representation of values.
///
/// QBE has a minimal type system with base types and extended types.
/// Base types are used for temporaries, while extended types can be used
/// in aggregate types and data definitions.
///
/// # Examples
///
/// ```rust
/// use qbe::Type;
///
/// // Base types
/// let word = Type::Word;     // 32-bit integer
/// let long = Type::Long;     // 64-bit integer
/// let single = Type::Single; // 32-bit float
/// let double = Type::Double; // 64-bit float
///
/// // Extended types
/// let byte = Type::Byte;     // 8-bit value
/// let halfword = Type::Halfword; // 16-bit value
///
/// // Get type sizes in bytes
/// assert_eq!(word.size(), 4);
/// assert_eq!(byte.size(), 1);
/// ```
///
/// ## Type Conversions
///
/// ```rust
/// use qbe::Type;
///
/// // Convert extended type to corresponding base type
/// let base = Type::Byte.into_base();
/// assert_eq!(base, Type::Word);
///
/// // Convert to ABI-compatible type for function parameters
/// let abi = Type::SignedByte.into_abi();
/// assert_eq!(abi, Type::Word);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Type<'a> {
    // Base types
    Word,
    Long,
    Single,
    Double,

    // Internal types
    Zero,

    // Extended types
    Byte,
    SignedByte,
    UnsignedByte,
    Halfword,
    SignedHalfword,
    UnsignedHalfword,

    /// Aggregate type with a specified name
    Aggregate(&'a TypeDef<'a>),
}

impl Type<'_> {
    /// Returns a C ABI type. Extended types are converted to closest base
    /// types
    pub fn into_abi(self) -> Self {
        match self {
            Self::Byte
            | Self::SignedByte
            | Self::UnsignedByte
            | Self::Halfword
            | Self::SignedHalfword
            | Self::UnsignedHalfword => Self::Word,
            other => other,
        }
    }

    /// Returns the closest base type
    pub fn into_base(self) -> Self {
        match self {
            Self::Byte
            | Self::SignedByte
            | Self::UnsignedByte
            | Self::Halfword
            | Self::SignedHalfword
            | Self::UnsignedHalfword => Self::Word,
            Self::Aggregate(_) => Self::Long,
            other => other,
        }
    }

    /// Returns byte size for values of the type
    pub fn size(&self) -> u64 {
        match self {
            Self::Byte | Self::SignedByte | Self::UnsignedByte | Self::Zero => 1,
            Self::Halfword | Self::SignedHalfword | Self::UnsignedHalfword => 2,
            Self::Word | Self::Single => 4,
            Self::Long | Self::Double => 8,
            Self::Aggregate(td) => {
                let mut offset = 0;

                // calculation taken from: https://en.wikipedia.org/wiki/Data_structure_alignment#Computing%20padding
                for (item, repeat) in td.items.iter() {
                    let align = item.align();
                    let size = *repeat as u64 * item.size();
                    let padding = (align - (offset % align)) % align;
                    offset += padding + size;
                }

                let align = self.align();
                let padding = (align - (offset % align)) % align;

                // size is the final offset with the padding that is left
                offset + padding
            }
        }
    }

    /// Returns byte alignment for values of the type
    pub fn align(&self) -> u64 {
        match self {
            Self::Aggregate(td) => {
                if let Some(align) = td.align {
                    return align;
                }

                // the alignment of a type is the maximum alignment of its members
                // when there's no members, the alignment is usuallly defined to be 1.
                td.items
                    .iter()
                    .map(|item| item.0.align())
                    .max()
                    .unwrap_or(1)
            }

            _ => self.size(),
        }
    }
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Byte => write!(f, "b"),
            Self::SignedByte => write!(f, "sb"),
            Self::UnsignedByte => write!(f, "ub"),
            Self::Halfword => write!(f, "h"),
            Self::SignedHalfword => write!(f, "sh"),
            Self::UnsignedHalfword => write!(f, "uh"),
            Self::Word => write!(f, "w"),
            Self::Long => write!(f, "l"),
            Self::Single => write!(f, "s"),
            Self::Double => write!(f, "d"),
            Self::Zero => write!(f, "z"),
            Self::Aggregate(td) => write!(f, ":{}", td.name),
        }
    }
}

/// QBE value that is accepted by instructions
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Value {
    /// `%`-temporary
    Temporary(String),
    /// `$`-global
    Global(String),
    /// Constant
    Const(u64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Temporary(name) => write!(f, "%{name}"),
            Self::Global(name) => write!(f, "${name}"),
            Self::Const(value) => write!(f, "{value}"),
        }
    }
}

/// QBE data definition
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct DataDef<'a> {
    pub linkage: Linkage,
    pub name: String,
    pub align: Option<u64>,
    pub items: Vec<(Type<'a>, DataItem)>,
}

impl<'a> DataDef<'a> {
    pub fn new(
        linkage: Linkage,
        name: impl Into<String>,
        align: Option<u64>,
        items: Vec<(Type<'a>, DataItem)>,
    ) -> Self {
        Self {
            linkage,
            name: name.into(),
            align,
            items,
        }
    }
}

impl fmt::Display for DataDef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}data ${} = ", self.linkage, self.name)?;

        if let Some(align) = self.align {
            write!(f, "align {align} ")?;
        }
        write!(
            f,
            "{{ {} }}",
            self.items
                .iter()
                .map(|(ty, item)| format!("{ty} {item}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Data definition item
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DataItem {
    /// Symbol and offset
    Symbol(String, Option<u64>),
    /// String
    Str(String),
    /// Constant
    Const(u64),
    /// Zero-initialized data of specified size
    Zero(u64),
}

impl fmt::Display for DataItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(name, offset) => match offset {
                Some(off) => write!(f, "${name} +{off}"),
                None => write!(f, "${name}"),
            },
            Self::Str(string) => write!(f, "\"{string}\""),
            Self::Const(val) => write!(f, "{val}"),
            Self::Zero(size) => write!(f, "z {size}"),
        }
    }
}

/// QBE aggregate type definition
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct TypeDef<'a> {
    pub name: String,
    pub align: Option<u64>,
    // TODO: Opaque types?
    pub items: Vec<(Type<'a>, usize)>,
}

impl fmt::Display for TypeDef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type :{} = ", self.name)?;
        if let Some(align) = self.align {
            write!(f, "align {align} ")?;
        }

        write!(
            f,
            "{{ {} }}",
            self.items
                .iter()
                .map(|(ty, count)| if *count > 1 {
                    format!("{ty} {count}")
                } else {
                    format!("{ty}")
                })
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

/// An IR statement
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Statement<'a> {
    Assign(Value, Type<'a>, Instr<'a>),
    Volatile(Instr<'a>),
}

impl fmt::Display for Statement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Assign(temp, ty, instr) => {
                assert!(matches!(temp, Value::Temporary(_)));
                write!(f, "{temp} ={ty} {instr}")
            }
            Self::Volatile(instr) => write!(f, "{instr}"),
        }
    }
}

/// A block of QBE instructions with a label.
///
/// Blocks are the basic units of control flow in QBE. Each block has a label
/// that can be the target of jumps, and contains a sequence of instructions.
/// A block typically ends with a control flow instruction like jump or return.
///
/// # Examples
///
/// ```rust
/// use qbe::{Block, BlockItem, Instr, Statement, Type, Value};
///
/// // Create a block for a loop body
/// let mut block = Block {
///     label: "loop".to_string(),
///     items: Vec::new(),
/// };
///
/// // Add a helpful comment
/// block.add_comment("Loop body - increment counter and accumulate sum");
///
/// // Increment loop counter: %i = %i + 1
/// block.assign_instr(
///     Value::Temporary("i".to_string()),
///     Type::Word,
///     Instr::Add(
///         Value::Temporary("i".to_string()),
///         Value::Const(1),
///     ),
/// );
///
/// // Update sum: %sum = %sum + %value
/// block.assign_instr(
///     Value::Temporary("sum".to_string()),
///     Type::Word,
///     Instr::Add(
///         Value::Temporary("sum".to_string()),
///         Value::Temporary("value".to_string()),
///     ),
/// );
///
/// // Jump to condition check block
/// block.add_instr(Instr::Jmp("cond".to_string()));
///
/// // Check if block ends with a jump (it does)
/// assert!(block.jumps());
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Block<'a> {
    /// Label before the block
    pub label: String,

    /// A list of statements in the block
    pub items: Vec<BlockItem<'a>>,
}

/// See [`Block::items`];
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BlockItem<'a> {
    Statement(Statement<'a>),
    Comment(String),
}

impl fmt::Display for BlockItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Statement(stmt) => write!(f, "{stmt}"),
            Self::Comment(comment) => write!(f, "# {comment}"),
        }
    }
}

impl<'a> Block<'a> {
    pub fn add_comment(&mut self, contents: impl Into<String>) {
        self.items.push(BlockItem::Comment(contents.into()));
    }

    /// Adds a new instruction to the block
    pub fn add_instr(&mut self, instr: Instr<'a>) {
        self.items
            .push(BlockItem::Statement(Statement::Volatile(instr)));
    }

    /// Adds a new instruction assigned to a temporary
    pub fn assign_instr(&mut self, temp: Value, ty: Type<'a>, instr: Instr<'a>) {
        let final_type = match instr {
            Instr::Call(_, _, _) => ty,
            _ => ty.into_base(),
        };

        self.items.push(BlockItem::Statement(Statement::Assign(
            temp, final_type, instr,
        )));
    }

    /// Returns true if the block's last instruction is a jump
    pub fn jumps(&self) -> bool {
        let last = self.items.last();

        if let Some(BlockItem::Statement(Statement::Volatile(instr))) = last {
            matches!(instr, Instr::Ret(_) | Instr::Jmp(_) | Instr::Jnz(..))
        } else {
            false
        }
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@{}", self.label)?;

        write!(
            f,
            "{}",
            self.items
                .iter()
                .map(|instr| format!("\t{instr}"))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

/// A QBE function definition.
///
/// A function consists of a name, linkage information, arguments, return type,
/// and a collection of blocks containing the function's implementation.
///
/// # Examples
///
/// ```rust
/// use qbe::{Function, Linkage, Type, Value, Instr, Cmp};
///
/// // Create a function that checks if a number is even
/// let mut is_even = Function::new(
///     Linkage::public(),
///     "is_even",
///     vec![(Type::Word, Value::Temporary("n".to_string()))],
///     Some(Type::Word), // Returns 1 if even, 0 if odd
/// );
///
/// // Add the start block
/// let mut start = is_even.add_block("start");
///
/// // Calculate n % 2 (by using n & 1)
/// start.assign_instr(
///     Value::Temporary("remainder".to_string()),
///     Type::Word,
///     Instr::And(
///         Value::Temporary("n".to_string()),
///         Value::Const(1),
///     ),
/// );
///
/// // Check if remainder is 0 (even number)
/// start.assign_instr(
///     Value::Temporary("is_zero".to_string()),
///     Type::Word,
///     Instr::Cmp(
///         Type::Word,
///         Cmp::Eq,
///         Value::Temporary("remainder".to_string()),
///         Value::Const(0),
///     ),
/// );
///
/// // Return the result
/// start.add_instr(Instr::Ret(Some(Value::Temporary("is_zero".to_string()))));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Function<'a> {
    /// Function's linkage
    pub linkage: Linkage,

    /// Function name
    pub name: String,

    /// Function arguments
    pub arguments: Vec<(Type<'a>, Value)>,

    /// Return type
    pub return_ty: Option<Type<'a>>,

    /// Labelled blocks
    pub blocks: Vec<Block<'a>>,
}

impl<'a> Function<'a> {
    /// Instantiates an empty function and returns it
    pub fn new(
        linkage: Linkage,
        name: impl Into<String>,
        arguments: Vec<(Type<'a>, Value)>,
        return_ty: Option<Type<'a>>,
    ) -> Self {
        Function {
            linkage,
            name: name.into(),
            arguments,
            return_ty,
            blocks: Vec::new(),
        }
    }

    /// Adds a new empty block with a specified label and returns a reference to it
    pub fn add_block(&mut self, label: impl Into<String>) -> &mut Block<'a> {
        self.blocks.push(Block {
            label: label.into(),
            items: Vec::new(),
        });
        self.blocks.last_mut().unwrap()
    }

    /// Returns a reference to the last block
    #[deprecated(
        since = "3.0.0",
        note = "Use `self.blocks.last()` or `self.blocks.last_mut()` instead."
    )]
    pub fn last_block(&mut self) -> &Block {
        self.blocks
            .last()
            .expect("Function must have at least one block")
    }

    /// Adds a new instruction to the last block
    pub fn add_instr(&mut self, instr: Instr<'a>) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .add_instr(instr);
    }

    /// Adds a new instruction assigned to a temporary
    pub fn assign_instr(&mut self, temp: Value, ty: Type<'a>, instr: Instr<'a>) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .assign_instr(temp, ty, instr);
    }
}

impl fmt::Display for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}function", self.linkage)?;
        if let Some(ty) = &self.return_ty {
            write!(f, " {ty}")?;
        }

        writeln!(
            f,
            " ${name}({args}) {{",
            name = self.name,
            args = self
                .arguments
                .iter()
                .map(|(ty, temp)| format!("{ty} {temp}"))
                .collect::<Vec<String>>()
                .join(", "),
        )?;

        for blk in self.blocks.iter() {
            writeln!(f, "{blk}")?;
        }

        write!(f, "}}")
    }
}

/// Linkage of a function or data defintion (e.g. section and
/// private/public status)
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Linkage {
    /// Specifies whether the target is going to be accessible publicly
    pub exported: bool,

    /// Specifies target's section
    pub section: Option<String>,

    /// Specifies target's section flags
    pub secflags: Option<String>,

    /// Specifies whether the target is stored in thread-local storage
    pub thread_local: bool,
}

impl Linkage {
    /// Returns the default configuration for private linkage
    pub fn private() -> Linkage {
        Linkage {
            exported: false,
            section: None,
            secflags: None,
            thread_local: false,
        }
    }

    /// Returns the configuration for private linkage with a provided section
    pub fn private_with_section(section: impl Into<String>) -> Linkage {
        Linkage {
            exported: false,
            section: Some(section.into()),
            secflags: None,
            thread_local: false,
        }
    }

    /// Returns the default configuration for public linkage
    pub fn public() -> Linkage {
        Linkage {
            exported: true,
            section: None,
            secflags: None,
            thread_local: false,
        }
    }

    /// Returns the configuration for public linkage with a provided section
    pub fn public_with_section(section: impl Into<String>) -> Linkage {
        Linkage {
            exported: true,
            section: Some(section.into()),
            secflags: None,
            thread_local: false,
        }
    }

    pub fn thread_local() -> Linkage {
        Linkage {
            exported: false,
            thread_local: true,
            section: None,
            secflags: None,
        }
    }

    pub fn exported_thread_local() -> Linkage {
        Linkage {
            exported: true,
            thread_local: true,
            section: None,
            secflags: None,
        }
    }

    pub fn thread_local_with_section(section: impl Into<String>) -> Linkage {
        Linkage {
            exported: false,
            thread_local: true,
            section: Some(section.into()),
            secflags: None,
        }
    }
}

impl fmt::Display for Linkage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.exported {
            write!(f, "export ")?;
        }
        if self.thread_local {
            write!(f, "thread ")?;
        }
        if let Some(section) = &self.section {
            // TODO: escape it, possibly
            write!(f, "section \"{section}\"")?;
            if let Some(secflags) = &self.secflags {
                write!(f, " \"{secflags}\"")?;
            }
            write!(f, " ")?;
        }

        Ok(())
    }
}

/// A complete QBE IL module.
///
/// A module contains all the functions, data definitions, and type definitions
/// that make up a QBE IL file. When converted to a string, it produces valid
/// QBE IL code that can be compiled by QBE.
///
/// # Examples
///
/// ```rust
/// use qbe::{Module, Function, DataDef, TypeDef, Linkage, Type, Value, Instr, DataItem};
///
/// // Create a new module
/// let mut module = Module::new();
///
/// // Add a string constant
/// let hello_str = DataDef::new(
///     Linkage::private(),
///     "hello",
///     None,
///     vec![
///         (Type::Byte, DataItem::Str("Hello, World!\n".to_string())),
///         (Type::Byte, DataItem::Const(0)), // Null terminator
///     ],
/// );
/// module.add_data(hello_str);
///
/// // Add a main function that prints the string
/// let mut main = Function::new(
///     Linkage::public(),
///     "main",
///     vec![],
///     Some(Type::Word),
/// );
///
/// let mut start = main.add_block("start");
///
/// // Call printf with the string: %r = call $printf(l $hello)
/// start.assign_instr(
///     Value::Temporary("r".to_string()),
///     Type::Word,
///     Instr::Call(
///         "printf".to_string(),
///         vec![(Type::Long, Value::Global("hello".to_string()))],
///         None,
///     ),
/// );
///
/// // Return 0
/// start.add_instr(Instr::Ret(Some(Value::Const(0))));
///
/// // Add the function to the module
/// module.add_function(main);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Module<'a> {
    functions: Vec<Function<'a>>,
    types: Vec<TypeDef<'a>>,
    data: Vec<DataDef<'a>>,
}

impl<'a> Module<'a> {
    /// Creates a new module
    pub fn new() -> Module<'a> {
        Module {
            functions: Vec::new(),
            types: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Adds a function to the module, returning a reference to it for later
    /// modification
    pub fn add_function(&mut self, func: Function<'a>) -> &mut Function<'a> {
        self.functions.push(func);
        self.functions.last_mut().unwrap()
    }

    /// Adds a type definition to the module, returning a reference to it for
    /// later modification
    pub fn add_type(&mut self, def: TypeDef<'a>) -> &mut TypeDef<'a> {
        self.types.push(def);
        self.types.last_mut().unwrap()
    }

    /// Adds a data definition to the module
    pub fn add_data(&mut self, data: DataDef<'a>) -> &mut DataDef<'a> {
        self.data.push(data);
        self.data.last_mut().unwrap()
    }
}

impl fmt::Display for Module<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ty in self.types.iter() {
            writeln!(f, "{ty}")?;
        }
        for func in self.functions.iter() {
            writeln!(f, "{func}")?;
        }
        for data in self.data.iter() {
            writeln!(f, "{data}")?;
        }
        Ok(())
    }
}
