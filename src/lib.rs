// Copyright 2022 Garrit Franke
// Copyright 2021 Alexey Yerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

#[cfg(test)]
mod tests;

/// QBE comparision
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
}

/// QBE instruction
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
    Call(String, Vec<(Type<'a>, Value)>),
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
}

impl<'a> fmt::Display for Instr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add(lhs, rhs) => write!(f, "add {}, {}", lhs, rhs),
            Self::Sub(lhs, rhs) => write!(f, "sub {}, {}", lhs, rhs),
            Self::Mul(lhs, rhs) => write!(f, "mul {}, {}", lhs, rhs),
            Self::Div(lhs, rhs) => write!(f, "div {}, {}", lhs, rhs),
            Self::Rem(lhs, rhs) => write!(f, "rem {}, {}", lhs, rhs),
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
            Self::Alloc4(size) => write!(f, "alloc4 {}", size),
            Self::Alloc8(size) => write!(f, "alloc8 {}", size),
            Self::Alloc16(size) => write!(f, "alloc16 {}", size),
            Self::Store(ty, dest, value) => {
                if matches!(ty, Type::Aggregate(_)) {
                    unimplemented!("Store to an aggregate type");
                }

                write!(f, "store{} {}, {}", ty, value, dest)
            }
            Self::Load(ty, src) => {
                if matches!(ty, Type::Aggregate(_)) {
                    unimplemented!("Load aggregate type");
                }

                write!(f, "load{} {}", ty, src)
            }
            Self::Blit(src, dst, n) => write!(f, "blit {}, {}, {}", src, dst, n),
        }
    }
}

/// QBE type
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Type<'a> {
    // Base types
    Word,
    Long,
    Single,
    Double,

    // Extended types
    Byte,
    Halfword,

    /// Aggregate type with a specified name
    Aggregate(&'a TypeDef<'a>),
}

impl<'a> Type<'a> {
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
            Self::Byte => 1,
            Self::Halfword => 2,
            Self::Word | Self::Single => 4,
            Self::Long | Self::Double => 8,
            Self::Aggregate(td) => {
                // TODO: correct for alignment
                let mut sz = 0_u64;
                for (item, repeat) in td.items.iter() {
                    sz += item.size() * (*repeat as u64);
                }
                sz
            }
        }
    }
}

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Byte => write!(f, "b"),
            Self::Halfword => write!(f, "h"),
            Self::Word => write!(f, "w"),
            Self::Long => write!(f, "l"),
            Self::Single => write!(f, "s"),
            Self::Double => write!(f, "d"),
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
            Self::Temporary(name) => write!(f, "%{}", name),
            Self::Global(name) => write!(f, "${}", name),
            Self::Const(value) => write!(f, "{}", value),
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

impl<'a> fmt::Display for DataDef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}data ${} = ", self.linkage, self.name)?;

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
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DataItem {
    /// Symbol and offset
    Symbol(String, Option<u64>),
    /// String
    Str(String),
    /// Constant
    Const(u64),
}

impl fmt::Display for DataItem {
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
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct TypeDef<'a> {
    pub name: String,
    pub align: Option<u64>,
    // TODO: Opaque types?
    pub items: Vec<(Type<'a>, usize)>,
}

impl<'a> fmt::Display for TypeDef<'a> {
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
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Statement<'a> {
    Assign(Value, Type<'a>, Instr<'a>),
    Volatile(Instr<'a>),
}

impl<'a> fmt::Display for Statement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Assign(temp, ty, instr) => {
                assert!(matches!(temp, Value::Temporary(_)));
                write!(f, "{} ={} {}", temp, ty, instr)
            }
            Self::Volatile(instr) => write!(f, "{}", instr),
        }
    }
}

/// Function block with a label
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Block<'a> {
    /// Label before the block
    pub label: String,

    /// A list of statements in the block
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Block<'a> {
    /// Adds a new instruction to the block
    pub fn add_instr(&mut self, instr: Instr<'a>) {
        self.statements.push(Statement::Volatile(instr));
    }

    /// Adds a new instruction assigned to a temporary
    pub fn assign_instr(&mut self, temp: Value, ty: Type<'a>, instr: Instr<'a>) {
        self.statements
            .push(Statement::Assign(temp, ty.into_base(), instr));
    }

    /// Returns true if the block's last instruction is a jump
    pub fn jumps(&self) -> bool {
        let last = self.statements.last();

        if let Some(Statement::Volatile(instr)) = last {
            matches!(instr, Instr::Ret(_) | Instr::Jmp(_) | Instr::Jnz(..))
        } else {
            false
        }
    }
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "@{}", self.label)?;

        write!(
            f,
            "{}",
            self.statements
                .iter()
                .map(|instr| format!("\t{}", instr))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

/// QBE function
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
            statements: Vec::new(),
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

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}function", self.linkage)?;
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
}

impl Linkage {
    /// Returns the default configuration for private linkage
    pub fn private() -> Linkage {
        Linkage {
            exported: false,
            section: None,
            secflags: None,
        }
    }

    /// Returns the configuration for private linkage with a provided section
    pub fn private_with_section(section: impl Into<String>) -> Linkage {
        Linkage {
            exported: false,
            section: Some(section.into()),
            secflags: None,
        }
    }

    /// Returns the default configuration for public linkage
    pub fn public() -> Linkage {
        Linkage {
            exported: true,
            section: None,
            secflags: None,
        }
    }

    /// Returns the configuration for public linkage with a provided section
    pub fn public_with_section(section: impl Into<String>) -> Linkage {
        Linkage {
            exported: true,
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
        if let Some(section) = &self.section {
            // TODO: escape it, possibly
            write!(f, "section \"{}\"", section)?;
            if let Some(secflags) = &self.secflags {
                write!(f, " \"{}\"", secflags)?;
            }
            write!(f, " ")?;
        }

        Ok(())
    }
}

/// A complete IL file
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
        return self.functions.last_mut().unwrap();
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

impl<'a> fmt::Display for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for func in self.functions.iter() {
            writeln!(f, "{}", func)?;
        }
        for ty in self.types.iter() {
            writeln!(f, "{}", ty)?;
        }
        for data in self.data.iter() {
            writeln!(f, "{}", data)?;
        }
        Ok(())
    }
}
