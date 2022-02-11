// Copyright 2022 Garrit Franke
// Copyright 2021 Alexey Yerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::*;

#[test]
fn qbe_value() {
    let val = Value::Temporary("temp42".into());
    assert_eq!(format!("{}", val), "%temp42");

    let val = Value::Global("main".into());
    assert_eq!(format!("{}", val), "$main");

    let val = Value::Const(1337);
    assert_eq!(format!("{}", val), "1337");
}

#[test]
fn block() {
    let blk = Block {
        label: "start".into(),
        statements: vec![Statement::Volatile(Instr::Ret(None))],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");

    let blk = Block {
        label: "start".into(),
        statements: vec![
            Statement::Assign(
                Value::Temporary("foo".into()),
                Type::Word,
                Instr::Add(Value::Const(2), Value::Const(2)),
            ),
            Statement::Volatile(Instr::Ret(Some(Value::Temporary("foo".into())))),
        ],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\t%foo =w add 2, 2");
    assert_eq!(lines.next().unwrap(), "\tret %foo");
}

#[test]
fn function() {
    let func = Function {
        linkage: Linkage::public(),
        return_ty: None,
        name: "main".into(),
        arguments: Vec::new(),
        blocks: vec![Block {
            label: "start".into(),
            statements: vec![Statement::Volatile(Instr::Ret(None))],
        }],
    };

    let formatted = format!("{}", func);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "export function $main() {");
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");
    assert_eq!(lines.next().unwrap(), "}");
}

#[test]
fn datadef() {
    let datadef = DataDef {
        linkage: Linkage::public(),
        name: "hello".into(),
        align: None,
        items: vec![
            (Type::Byte, DataItem::Str("Hello, World!".into())),
            (Type::Byte, DataItem::Const(0)),
        ],
    };

    let formatted = format!("{}", datadef);
    assert_eq!(
        formatted,
        "export data $hello = { b \"Hello, World!\", b 0 }"
    );
}

#[test]
fn typedef() {
    let typedef = TypeDef {
        name: "person".into(),
        align: None,
        items: vec![(Type::Long, 1), (Type::Word, 2), (Type::Byte, 1)],
    };

    let formatted = format!("{}", typedef);
    assert_eq!(formatted, "type :person = { l, w 2, b }");
}

#[test]
fn type_into_abi() {
    // Base types and aggregates should stay unchanged
    let unchanged = |ty: Type| assert_eq!(ty.clone().into_abi(), ty);
    unchanged(Type::Word);
    unchanged(Type::Long);
    unchanged(Type::Single);
    unchanged(Type::Double);
    unchanged(Type::Aggregate("foo".into()));

    // Extended types are transformed into closest base types
    assert_eq!(Type::Byte.into_abi(), Type::Word);
    assert_eq!(Type::Halfword.into_abi(), Type::Word);
}

#[test]
fn type_into_base() {
    // Base types should stay unchanged
    let unchanged = |ty: Type| assert_eq!(ty.clone().into_base(), ty);
    unchanged(Type::Word);
    unchanged(Type::Long);
    unchanged(Type::Single);
    unchanged(Type::Double);

    // Extended and aggregate types are transformed into closest base types
    assert_eq!(Type::Byte.into_base(), Type::Word);
    assert_eq!(Type::Halfword.into_base(), Type::Word);
    assert_eq!(Type::Aggregate("foo".into()).into_base(), Type::Long);
}
