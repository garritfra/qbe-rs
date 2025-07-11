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
    assert_eq!(format!("{val}"), "%temp42");

    let val = Value::Global("main".into());
    assert_eq!(format!("{val}"), "$main");

    let val = Value::Const(1337);
    assert_eq!(format!("{val}"), "1337");
}

#[test]
fn block() {
    let blk = Block {
        label: "start".into(),
        items: vec![BlockItem::Statement(Statement::Volatile(Instr::Ret(None)))],
    };

    let formatted = format!("{blk}");
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");

    let blk = Block {
        label: "start".into(),
        items: vec![
            BlockItem::Comment("Comment".into()),
            BlockItem::Statement(Statement::Assign(
                Value::Temporary("foo".into()),
                Type::Word,
                Instr::Add(Value::Const(2), Value::Const(2)),
            )),
            BlockItem::Statement(Statement::Volatile(Instr::Ret(Some(Value::Temporary(
                "foo".into(),
            ))))),
        ],
    };

    let formatted = format!("{blk}");
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\t# Comment");
    assert_eq!(lines.next().unwrap(), "\t%foo =w add 2, 2");
    assert_eq!(lines.next().unwrap(), "\tret %foo");
}

#[test]
fn instr_blit() {
    let blk = Block {
        label: "start".into(),
        items: vec![BlockItem::Statement(Statement::Volatile(Instr::Blit(
            Value::Temporary("src".into()),
            Value::Temporary("dst".into()),
            4,
        )))],
    };

    let formatted = format!("{blk}");
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
            items: vec![BlockItem::Statement(Statement::Volatile(Instr::Ret(None)))],
        }],
    };

    let formatted = format!("{func}");
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

    let formatted = format!("{datadef}");
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

    let formatted = format!("{typedef}");
    assert_eq!(formatted, "type :person = { l, w 2, b }");

    let ty = Type::Aggregate(&typedef);
    let formatted = format!("{ty}");
    assert_eq!(formatted, ":person");
}

#[test]
fn type_size() {
    assert!(Type::Byte.size() == 1);
    assert!(Type::SignedByte.size() == 1);
    assert!(Type::UnsignedByte.size() == 1);
    assert!(Type::Halfword.size() == 2);
    assert!(Type::SignedHalfword.size() == 2);
    assert!(Type::UnsignedHalfword.size() == 2);
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
    assert_eq!(aggregate.size(), 24);
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

    assert_eq!(aggregate.size(), 40);
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
    assert_eq!(Type::UnsignedByte.into_abi(), Type::Word);
    assert_eq!(Type::SignedByte.into_abi(), Type::Word);
    assert_eq!(Type::Halfword.into_abi(), Type::Word);
    assert_eq!(Type::UnsignedHalfword.into_abi(), Type::Word);
    assert_eq!(Type::SignedHalfword.into_abi(), Type::Word);
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
    assert_eq!(Type::UnsignedByte.into_base(), Type::Word);
    assert_eq!(Type::SignedHalfword.into_base(), Type::Word);
    assert_eq!(Type::Halfword.into_base(), Type::Word);
    assert_eq!(Type::UnsignedHalfword.into_base(), Type::Word);
    assert_eq!(Type::SignedHalfword.into_base(), Type::Word);
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

#[test]
fn variadic_call() {
    let instr = Instr::Call(
        "printf".into(),
        vec![
            (Type::Long, Value::Global("fmt".into())),
            (Type::Word, Value::Const(0)),
        ],
        Some(1),
    );

    assert_eq!(instr.to_string(), "call $printf(l $fmt, ..., w 0)");
}

#[test]
fn module_fmt_order() {
    // Create a module
    let mut module = Module::new();

    // Add a type definition to the module
    let typedef = TypeDef {
        name: "test_type".into(),
        align: None,
        items: vec![(Type::Long, 1)],
    };
    module.add_type(typedef);

    // Add a function to the module
    let mut func = Function::new(Linkage::public(), "test_func", Vec::new(), None);

    // Add a block to the function and an instruction to the block
    let block = func.add_block("entry");
    block.add_instr(Instr::Ret(None));

    module.add_function(func);

    // Add some data to the module for completeness
    let data = DataDef::new(
        Linkage::private(),
        "test_data",
        None,
        vec![(Type::Word, DataItem::Const(42))],
    );
    module.add_data(data);

    // Format the module to a string
    let formatted = format!("{module}");

    // Verify the order: types, then functions, then data
    let type_pos = formatted
        .find("type :test_type")
        .expect("Type definition not found");
    let func_pos = formatted
        .find("export function $test_func")
        .expect("Function not found");
    let data_pos = formatted
        .find("data $test_data")
        .expect("Data definition not found");

    assert!(
        type_pos < func_pos,
        "Type definition should appear before function"
    );
    assert!(
        func_pos < data_pos,
        "Function should appear before data definition"
    );
}

#[test]
fn comparison_types() {
    // Test ordered/unordered comparisons for floating point
    let ordered_cmp = Statement::Volatile(Instr::Cmp(
        Type::Double,
        Cmp::O,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{ordered_cmp}"), "cod %a, %b");

    let unordered_cmp = Statement::Volatile(Instr::Cmp(
        Type::Single,
        Cmp::Uo,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{unordered_cmp}"), "cuos %a, %b");

    // Test unsigned comparisons for integers
    let unsigned_lt = Statement::Volatile(Instr::Cmp(
        Type::Word,
        Cmp::Ult,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{unsigned_lt}"), "cultw %a, %b");

    let unsigned_le = Statement::Volatile(Instr::Cmp(
        Type::Long,
        Cmp::Ule,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{unsigned_le}"), "culel %a, %b");

    let unsigned_gt = Statement::Volatile(Instr::Cmp(
        Type::Word,
        Cmp::Ugt,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{unsigned_gt}"), "cugtw %a, %b");

    let unsigned_ge = Statement::Volatile(Instr::Cmp(
        Type::Long,
        Cmp::Uge,
        Value::Temporary("a".into()),
        Value::Temporary("b".into()),
    ));
    assert_eq!(format!("{unsigned_ge}"), "cugel %a, %b");
}

#[test]
fn unsigned_arithmetic_instructions() {
    let udiv = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Udiv(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    assert_eq!(format!("{udiv}"), "%result =w udiv %a, %b");

    let urem = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Urem(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    assert_eq!(format!("{urem}"), "%result =l urem %a, %b");
}

#[test]
fn shift_instructions() {
    let sar = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Sar(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    assert_eq!(format!("{sar}"), "%result =w sar %a, %b");

    let shr = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Shr(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    assert_eq!(format!("{shr}"), "%result =l shr %a, %b");

    let shl = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Shl(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    assert_eq!(format!("{shl}"), "%result =w shl %a, %b");
}

#[test]
fn cast_instruction() {
    let cast_int_to_float = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Single,
        Instr::Cast(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{cast_int_to_float}"), "%result =s cast %a");

    let cast_float_to_int = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Cast(Value::Temporary("f".into())),
    );
    assert_eq!(format!("{cast_float_to_int}"), "%result =w cast %f");
}

#[test]
fn extension_operations() {
    let extsw = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Extsw(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extsw}"), "%result =l extsw %a");

    let extuw = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Extuw(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extuw}"), "%result =l extuw %a");

    let extsh = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Extsh(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extsh}"), "%result =w extsh %a");

    let extuh = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Extuh(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extuh}"), "%result =w extuh %a");

    let extsb = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Extsb(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extsb}"), "%result =w extsb %a");

    let extub = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Extub(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{extub}"), "%result =w extub %a");
}

#[test]
fn float_precision_conversion() {
    let exts = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Double,
        Instr::Exts(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{exts}"), "%result =d exts %a");

    let truncd = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Single,
        Instr::Truncd(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{truncd}"), "%result =s truncd %a");
}

#[test]
fn float_integer_conversions() {
    let stosi = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Stosi(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{stosi}"), "%result =w stosi %a");

    let stoui = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Stoui(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{stoui}"), "%result =w stoui %a");

    let dtosi = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Dtosi(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{dtosi}"), "%result =l dtosi %a");

    let dtoui = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Long,
        Instr::Dtoui(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{dtoui}"), "%result =l dtoui %a");

    let swtof = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Single,
        Instr::Swtof(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{swtof}"), "%result =s swtof %a");

    let uwtof = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Single,
        Instr::Uwtof(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{uwtof}"), "%result =s uwtof %a");

    let sltof = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Double,
        Instr::Sltof(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{sltof}"), "%result =d sltof %a");

    let ultof = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Double,
        Instr::Ultof(Value::Temporary("a".into())),
    );
    assert_eq!(format!("{ultof}"), "%result =d ultof %a");
}

#[test]
fn variadic_instructions() {
    let vastart = Statement::Volatile(Instr::Vastart(Value::Temporary("ap".into())));
    assert_eq!(format!("{vastart}"), "vastart %ap");

    let vaarg = Statement::Assign(
        Value::Temporary("arg".into()),
        Type::Word,
        Instr::Vaarg(Type::Word, Value::Temporary("ap".into())),
    );
    assert_eq!(format!("{vaarg}"), "%arg =w vaargw %ap");
}

#[test]
fn phi_instruction() {
    let phi = Instr::Phi(
        "ift".into(),
        Value::Const(2),
        "iff".into(),
        Value::Temporary("3".into()),
    );
    assert_eq!(format!("{phi}"), "phi @ift 2, @iff %3");

    let phi = Statement::Assign(
        Value::Temporary("result".into()),
        Type::Word,
        Instr::Phi(
            "start".into(),
            Value::Temporary("1".into()),
            "loop".into(),
            Value::Global("tmp".into()),
        ),
    );
    assert_eq!(format!("{phi}"), "%result =w phi @start %1, @loop $tmp");
}

#[test]
fn halt_instruction() {
    let hlt = Statement::Volatile(Instr::Hlt);
    assert_eq!(format!("{hlt}"), "hlt");
}

#[test]
fn thread_local_linkage() {
    let thread_local = Linkage::thread_local();
    assert_eq!(format!("{thread_local}"), "thread ");

    let exported_thread_local = Linkage::exported_thread_local();
    assert_eq!(format!("{exported_thread_local}"), "export thread ");

    let thread_local_with_section = Linkage::thread_local_with_section("data");
    assert_eq!(
        format!("{thread_local_with_section}"),
        "thread section \"data\" "
    );

    // Test in a data definition
    let data_def = DataDef {
        linkage: Linkage::thread_local(),
        name: "thread_var".into(),
        align: None,
        items: vec![(Type::Word, DataItem::Const(42))],
    };
    assert_eq!(format!("{data_def}"), "thread data $thread_var = { w 42 }");
}

#[test]
fn zero_initialized_data() {
    let zero_data = DataItem::Zero(1000);
    assert_eq!(format!("{zero_data}"), "z 1000");

    // Test in a data definition
    let data_def = DataDef {
        linkage: Linkage::private(),
        name: "zero_array".into(),
        align: None,
        items: vec![(Type::Byte, DataItem::Zero(1000))],
    };
    assert_eq!(format!("{data_def}"), "data $zero_array = { b z 1000 }");

    // Test mixed with other items
    let data_def = DataDef {
        linkage: Linkage::private(),
        name: "mixed_data".into(),
        align: None,
        items: vec![
            (Type::Word, DataItem::Const(1)),
            (Type::Byte, DataItem::Zero(10)),
            (Type::Word, DataItem::Const(2)),
        ],
    };
    assert_eq!(
        format!("{data_def}"),
        "data $mixed_data = { w 1, b z 10, w 2 }"
    );
}

#[test]
fn complex_block_with_multiple_instructions() {
    // Create a block using several instructions
    let mut block = Block {
        label: "test_block".into(),
        items: Vec::new(),
    };

    // Add unsigned division
    block.assign_instr(
        Value::Temporary("udiv_result".into()),
        Type::Word,
        Instr::Udiv(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );

    // Add a shift operation
    block.assign_instr(
        Value::Temporary("shift_result".into()),
        Type::Word,
        Instr::Shl(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );

    // Add a cast operation
    block.assign_instr(
        Value::Temporary("cast_result".into()),
        Type::Single,
        Instr::Cast(Value::Temporary("shift_result".into())),
    );

    // Add a comparison (unordered)
    block.assign_instr(
        Value::Temporary("cmp_result".into()),
        Type::Word,
        Instr::Cmp(
            Type::Single,
            Cmp::Uo,
            Value::Temporary("cast_result".into()),
            Value::Temporary("x".into()),
        ),
    );

    // Add a termination
    block.add_instr(Instr::Hlt);

    // Format the block and check the output
    let formatted = format!("{block}");
    let lines: Vec<&str> = formatted.lines().collect();

    assert_eq!(lines[0], "@test_block");
    assert_eq!(lines[1], "\t%udiv_result =w udiv %a, %b");
    assert_eq!(lines[2], "\t%shift_result =w shl %a, %b");
    assert_eq!(lines[3], "\t%cast_result =s cast %shift_result");
    assert_eq!(lines[4], "\t%cmp_result =w cuos %cast_result, %x");
    assert_eq!(lines[5], "\thlt");
}

#[test]
fn assign_instr_aggregate_type_coercion() {
    let mut block = Block {
        label: "test_block".into(),
        items: Vec::new(),
    };

    let typedef = TypeDef {
        name: "person".into(),
        align: None,
        items: vec![(Type::Long, 1), (Type::Word, 2), (Type::Byte, 1)],
    };

    block.assign_instr(
        Value::Temporary("human".into()),
        Type::Aggregate(&typedef),
        Instr::Alloc8(Type::Aggregate(&typedef).size()),
    );

    block.assign_instr(
        Value::Temporary("result".into()),
        Type::Aggregate(&typedef),
        Instr::Call("new_person".into(), vec![], None),
    );

    let formatted = format!("{block}");
    let lines: Vec<&str> = formatted.lines().collect();

    assert_eq!(lines[0], "@test_block");
    assert_eq!(lines[1], "\t%human =l alloc8 24");
    assert_eq!(lines[2], "\t%result =:person call $new_person()");
}
