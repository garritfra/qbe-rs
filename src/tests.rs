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
fn instr_blit() {
    let blk = Block {
        label: "start".into(),
        statements: vec![Statement::Volatile(Instr::Blit(
            Value::Temporary("src".into()),
            Value::Temporary("dst".into()),
            4,
        ))],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tblit %src, %dst, 4");
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
fn function_new_equivalence() {
    let func1 = Function {
        linkage: Linkage::public(),
        return_ty: None,
        name: "main".into(),
        arguments: Vec::new(),
        blocks: Vec::new(),
    };

    let func2 = Function::new(Linkage::public(), "main", Vec::new(), None);

    assert_eq!(func1, func2);
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
fn datadef_new_equivalence() {
    let datadef1 = DataDef {
        linkage: Linkage::public(),
        name: "hello".into(),
        align: None,
        items: vec![],
    };

    let datadef2 = DataDef::new(Linkage::public(), "hello", None, vec![]);

    assert_eq!(datadef1, datadef2);
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

    let ty = Type::Aggregate(&typedef);
    let formatted = format!("{}", ty);
    assert_eq!(formatted, ":person");
}

#[test]
fn type_size() {
    assert!(Type::Byte.size() == 1);
    assert!(Type::Halfword.size() == 2);
    assert!(Type::Word.size() == 4);
    assert!(Type::Single.size() == 4);
    assert!(Type::Long.size() == 8);
    assert!(Type::Double.size() == 8);

    let typedef = TypeDef {
        name: "person".into(),
        align: None,
        items: vec![(Type::Long, 1), (Type::Word, 2), (Type::Byte, 1)],
    };
    let aggregate = Type::Aggregate(&typedef);
    assert!(aggregate.size() == 17);
}

#[test]
fn type_size_nested_aggregate() {
    let inner = TypeDef {
        name: "dog".into(),
        align: None,
        items: vec![(Type::Long, 2)],
    };
    let inner_aggregate = Type::Aggregate(&inner);

    assert!(inner_aggregate.size() == 16);

    let typedef = TypeDef {
        name: "person".into(),
        align: None,
        items: vec![
            (Type::Long, 1),
            (Type::Word, 2),
            (Type::Byte, 1),
            (Type::Aggregate(&inner), 1),
        ],
    };
    let aggregate = Type::Aggregate(&typedef);

    assert!(aggregate.size() == 33);
}

#[test]
fn type_into_abi() {
    // Base types and aggregates should stay unchanged
    let unchanged = |ty: Type| assert_eq!(ty.clone().into_abi(), ty);
    unchanged(Type::Word);
    unchanged(Type::Long);
    unchanged(Type::Single);
    unchanged(Type::Double);
    let typedef = TypeDef {
        name: "foo".into(),
        align: None,
        items: Vec::new(),
    };
    unchanged(Type::Aggregate(&typedef));

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
    let typedef = TypeDef {
        name: "foo".into(),
        align: None,
        items: Vec::new(),
    };
    assert_eq!(Type::Aggregate(&typedef).into_base(), Type::Long);
}

#[test]
fn add_function_to_module() {
    let mut module = Module::new();

    let function = Function {
        linkage: Linkage::public(),
        name: "foo".into(),
        arguments: Vec::new(),
        blocks: Vec::new(),
        return_ty: None,
    };

    module.add_function(function.clone());

    assert_eq!(module.functions.into_iter().next().unwrap(), function);
}
