// Copyright 2022 Garrit Franke
// Copyright 2021 Alexey Yerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

/// QBE comparision
#[derive(Debug)]
pub enum QbeCmp {
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
}

/// QBE instruction
#[derive(Debug)]
pub enum QbeInstr {
    /// Adds values of two temporaries together
    Add(QbeValue, QbeValue),
    /// Subtracts the second value from the first one
    Sub(QbeValue, QbeValue),
    /// Multiplies values of two temporaries
    Mul(QbeValue, QbeValue),
    /// Divides the first value by the second one
    Div(QbeValue, QbeValue),
    /// Returns a remainder from division
    Rem(QbeValue, QbeValue),
    /// Performs a comparion between values
    Cmp(QbeType, QbeCmp, QbeValue, QbeValue),
    /// Performs a bitwise AND on values
    And(QbeValue, QbeValue),
    /// Performs a bitwise OR on values
    Or(QbeValue, QbeValue),
    /// Copies either a temporary or a literal value
    Copy(QbeValue),
    /// Return from a function, optionally with a value
    Ret(Option<QbeValue>),
    /// Jumps to first label if a value is nonzero or to the second one otherwise
    Jnz(QbeValue, String, String),
    /// Unconditionally jumps to a label
    Jmp(String),
    /// Calls a function
    Call(String, Vec<(QbeType, QbeValue)>),
    /// Allocates a 8-byte aligned area on the stack
    Alloc8(u64),
    /// Stores a value into memory pointed to by destination.
    /// `(type, destination, value)`
    Store(QbeType, QbeValue, QbeValue),
    /// Loads a value from memory pointed to by source
    /// `(type, source)`
    Load(QbeType, QbeValue),
}

impl fmt::Display for QbeInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(lhs, rhs) => write!(f, "add {}, {}", lhs, rhs),
            Self::Sub(lhs, rhs) => write!(f, "sub {}, {}", lhs, rhs),
            Self::Mul(lhs, rhs) => write!(f, "mul {}, {}", lhs, rhs),
            Self::Div(lhs, rhs) => write!(f, "div {}, {}", lhs, rhs),
            Self::Rem(lhs, rhs) => write!(f, "rem {}, {}", lhs, rhs),
            Self::Cmp(ty, cmp, lhs, rhs) => {
                assert!(
                    !matches!(ty, QbeType::Aggregate(_)),
                    "Cannot compare aggregate types"
                );

                write!(
                    f,
                    "c{}{} {}, {}",
                    match cmp {
                        QbeCmp::Slt => "slt",
                        QbeCmp::Sle => "sle",
                        QbeCmp::Sgt => "sgt",
                        QbeCmp::Sge => "sge",
                        QbeCmp::Eq => "eq",
                        QbeCmp::Ne => "ne",
                    },
                    ty,
                    lhs,
                    rhs,
                )
            }
            Self::And(lhs, rhs) => write!(f, "and {}, {}", lhs, rhs),
            Self::Or(lhs, rhs) => write!(f, "or {}, {}", lhs, rhs),
            Self::Copy(val) => write!(f, "copy {}", val),
            Self::Ret(val) => match val {
                Some(val) => write!(f, "ret {}", val),
                None => write!(f, "ret"),
            },
            Self::Jnz(val, if_nonzero, if_zero) => {
                write!(f, "jnz {}, @{}, @{}", val, if_nonzero, if_zero)
            }
            Self::Jmp(label) => write!(f, "jmp @{}", label),
            Self::Call(name, args) => {
                write!(
                    f,
                    "call ${}({})",
                    name,
                    args.iter()
                        .map(|(ty, temp)| format!("{} {}", ty, temp))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
            Self::Alloc8(size) => write!(f, "alloc8 {}", size),
            Self::Store(ty, dest, value) => {
                if matches!(ty, QbeType::Aggregate(_)) {
                    unimplemented!("Store to an aggregate type");
                }

                write!(f, "store{} {}, {}", ty, value, dest)
            }
            Self::Load(ty, src) => {
                if matches!(ty, QbeType::Aggregate(_)) {
                    unimplemented!("Load aggregate type");
                }

                write!(f, "load{} {}", ty, src)
            }
        }
    }
}

/// QBE type
#[derive(Debug, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub enum QbeType {
    // Base types
    Word,
    Long,
    Single,
    Double,

    // Extended types
    Byte,
    Halfword,

    /// Aggregate type with a specified name
    Aggregate(String),
}

impl QbeType {
    /// Returns a C ABI type. Extended types are converted to closest base
    /// types
    pub fn into_abi(self) -> Self {
        match self {
            Self::Byte | Self::Halfword => Self::Word,
            other => other,
        }
    }

    /// Returns the closest base type
    pub fn into_base(self) -> Self {
        match self {
            Self::Byte | Self::Halfword => Self::Word,
            Self::Aggregate(_) => Self::Long,
            other => other,
        }
    }

    /// Returns byte size for values of the type
    pub fn size(&self) -> u64 {
        match self {
            Self::Word | Self::Single => 4,
            Self::Long | Self::Double => 8,
            Self::Byte => 1,
            Self::Halfword => 2,

            // Aggregate types are syntactic sugar for pointers ;)
            Self::Aggregate(_) => 8,
        }
    }
}

impl fmt::Display for QbeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word => write!(f, "w"),
            Self::Long => write!(f, "l"),
            Self::Single => write!(f, "s"),
            Self::Double => write!(f, "d"),

            Self::Byte => write!(f, "b"),
            Self::Halfword => write!(f, "h"),

            Self::Aggregate(name) => write!(f, ":{}", name),
        }
    }
}

/// QBE value that is accepted by instructions
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum QbeValue {
    /// `%`-temporary
    Temporary(String),
    /// `$`-global
    Global(String),
    /// Constant
    Const(u64),
}

impl fmt::Display for QbeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Temporary(name) => write!(f, "%{}", name),
            Self::Global(name) => write!(f, "${}", name),
            Self::Const(value) => write!(f, "{}", value),
        }
    }
}

/// QBE data definition
#[derive(Debug)]
pub struct QbeDataDef {
    pub exported: bool,
    pub name: String,
    pub align: Option<u64>,

    pub items: Vec<(QbeType, QbeDataItem)>,
}

impl fmt::Display for QbeDataDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exported {
            write!(f, "export ")?;
        }

        write!(f, "data ${} = ", self.name)?;

        if let Some(align) = self.align {
            write!(f, "align {} ", align)?;
        }
        write!(
            f,
            "{{ {} }}",
            self.items
                .iter()
                .map(|(ty, item)| format!("{} {}", ty, item))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Data definition item
#[derive(Debug)]
#[allow(dead_code)]
pub enum QbeDataItem {
    /// Symbol and offset
    Symbol(String, Option<u64>),
    /// String
    Str(String),
    /// Constant
    Const(u64),
}

impl fmt::Display for QbeDataItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(name, offset) => match offset {
                Some(off) => write!(f, "${} +{}", name, off),
                None => write!(f, "${}", name),
            },
            Self::Str(string) => write!(f, "\"{}\"", string),
            Self::Const(val) => write!(f, "{}", val),
        }
    }
}

/// QBE aggregate type definition
#[derive(Debug)]
pub struct QbeTypeDef {
    pub name: String,
    pub align: Option<u64>,
    // TODO: Opaque types?
    pub items: Vec<(QbeType, usize)>,
}

impl fmt::Display for QbeTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type :{} = ", self.name)?;
        if let Some(align) = self.align {
            write!(f, "align {} ", align)?;
        }

        write!(
            f,
            "{{ {} }}",
            self.items
                .iter()
                .map(|(ty, count)| if *count > 1 {
                    format!("{} {}", ty, count)
                } else {
                    format!("{}", ty)
                })
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

/// An IR statement
#[derive(Debug)]
pub enum QbeStatement {
    Assign(QbeValue, QbeType, QbeInstr),
    Volatile(QbeInstr),
}

impl fmt::Display for QbeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assign(temp, ty, instr) => {
                assert!(matches!(temp, QbeValue::Temporary(_)));
                write!(f, "{} ={} {}", temp, ty, instr)
            }
            Self::Volatile(instr) => write!(f, "{}", instr),
        }
    }
}

/// Function block with a label
#[derive(Debug)]
pub struct QbeBlock {
    /// Label before the block
    pub label: String,

    /// A list of instructions in the block
    pub instructions: Vec<QbeStatement>,
}

impl QbeBlock {
    /// Adds a new instruction to the block
    pub fn add_instr(&mut self, instr: QbeInstr) {
        self.instructions.push(QbeStatement::Volatile(instr));
    }

    /// Adds a new instruction assigned to a temporary
    pub fn assign_instr(&mut self, temp: QbeValue, ty: QbeType, instr: QbeInstr) {
        self.instructions
            .push(QbeStatement::Assign(temp, ty.into_base(), instr));
    }

    /// Returns true if the block's last instruction is a jump
    pub fn jumps(&self) -> bool {
        let last = self.instructions.last();

        if let Some(QbeStatement::Volatile(instr)) = last {
            matches!(
                instr,
                QbeInstr::Ret(_) | QbeInstr::Jmp(_) | QbeInstr::Jnz(..)
            )
        } else {
            false
        }
    }
}

impl fmt::Display for QbeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "@{}", self.label)?;

        write!(
            f,
            "{}",
            self.instructions
                .iter()
                .map(|instr| format!("\t{}", instr))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

/// QBE function
#[derive(Debug)]
pub struct QbeFunction {
    /// Should the function be available to outside users
    pub exported: bool,

    /// Function name
    pub name: String,

    /// Function arguments
    pub arguments: Vec<(QbeType, QbeValue)>,

    /// Return type
    pub return_ty: Option<QbeType>,

    /// Labelled blocks
    pub blocks: Vec<QbeBlock>,
}

impl QbeFunction {
    /// Adds a new empty block with a specified label
    pub fn add_block(&mut self, label: String) {
        self.blocks.push(QbeBlock {
            label,
            instructions: Vec::new(),
        });
    }

    pub fn last_block(&mut self) -> &QbeBlock {
        self.blocks
            .last()
            .expect("Function must have at least one block")
    }

    /// Adds a new instruction to the last block
    pub fn add_instr(&mut self, instr: QbeInstr) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .add_instr(instr);
    }

    /// Adds a new instruction assigned to a temporary
    pub fn assign_instr(&mut self, temp: QbeValue, ty: QbeType, instr: QbeInstr) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .assign_instr(temp, ty, instr);
    }
}

impl fmt::Display for QbeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exported {
            write!(f, "export ")?;
        }
        write!(f, "function")?;
        if let Some(ty) = &self.return_ty {
            write!(f, " {}", ty)?;
        }

        writeln!(
            f,
            " ${name}({args}) {{",
            name = self.name,
            args = self
                .arguments
                .iter()
                .map(|(ty, temp)| format!("{} {}", ty, temp))
                .collect::<Vec<String>>()
                .join(", "),
        )?;

        for blk in self.blocks.iter() {
            writeln!(f, "{}", blk)?;
        }

        write!(f, "}}")
    }
}
