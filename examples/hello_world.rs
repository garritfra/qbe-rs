use qbe::*;

// Represents the hello world example from https://c9x.me/compile/

fn generate_add_func() -> Function {
    let arguments = vec![
        (Type::Word, Value::Temporary("a".into())),
        (Type::Word, Value::Temporary("b".into())),
    ];

    let statements = vec![
        Statement::Assign(
            Value::Temporary("c".into()),
            Type::Word,
            Instr::Add(Value::Temporary("a".into()), Value::Temporary("b".into())),
        ),
        Statement::Volatile(Instr::Ret(Some(Value::Temporary("c".into())))),
    ];

    let blocks = vec![Block {
        label: "start".into(),
        statements,
    }];

    Function {
        exported: false,
        name: "add".into(),
        return_ty: Some(Type::Word),
        arguments,
        blocks,
    }
}

fn generate_main_func() -> Function {
    let statements = vec![
        Statement::Assign(
            Value::Temporary("r".into()),
            Type::Word,
            Instr::Call(
                "add".into(),
                vec![(Type::Word, Value::Const(1)), (Type::Word, Value::Const(1))],
            ),
        ),
        Statement::Volatile(Instr::Call(
            "printf".into(),
            vec![
                (Type::Long, Value::Global("fmt".into())),
                (Type::Word, Value::Temporary("r".into())),
            ],
        )),
        Statement::Volatile(Instr::Ret(Some(Value::Const(0)))),
    ];

    // TODO: The example shows a variadic call. We don't have those yet

    let blocks = vec![Block {
        label: "start".into(),
        statements,
    }];

    Function {
        exported: true,
        name: "main".into(),
        return_ty: Some(Type::Word),
        arguments: Vec::new(),
        blocks,
    }
}

fn generate_data() -> DataDef {
    let items = vec![
        (Type::Byte, DataItem::Str("One and one make %d!\\n".into())),
        (Type::Byte, DataItem::Const(0)),
    ];
    DataDef {
        exported: false,
        name: "fmt".into(),
        align: None,
        items,
    }
}

fn main() {
    println!("{}", generate_add_func());
    println!("{}", generate_main_func());
    println!("{}", generate_data());
}
