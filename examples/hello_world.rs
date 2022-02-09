use qbe::*;

// Represents the hello world example from https://c9x.me/compile/

fn generate_add_func() -> QbeFunction {
    let arguments = vec![
        (QbeType::Word, QbeValue::Temporary("a".into())),
        (QbeType::Word, QbeValue::Temporary("b".into())),
    ];

    let mut statements = Vec::new();

    statements.push(QbeStatement::Assign(
        QbeValue::Temporary("c".into()),
        QbeType::Word,
        QbeInstr::Add(
            QbeValue::Temporary("a".into()),
            QbeValue::Temporary("b".into()),
        ),
    ));

    statements.push(QbeStatement::Volatile(QbeInstr::Ret(Some(
        QbeValue::Temporary("c".into()),
    ))));

    let blocks = vec![QbeBlock {
        label: "start".into(),
        statements,
    }];

    QbeFunction {
        exported: false,
        name: "add".into(),
        return_ty: Some(QbeType::Word),
        arguments,
        blocks,
    }
}

fn generate_main_func() -> QbeFunction {
    let mut statements = Vec::new();

    statements.push(QbeStatement::Assign(
        QbeValue::Temporary("r".into()),
        QbeType::Word,
        QbeInstr::Call(
            "add".into(),
            vec![
                (QbeType::Word, QbeValue::Const(1)),
                (QbeType::Word, QbeValue::Const(1)),
            ],
        ),
    ));

    // TODO: The example shows a variadic call. We don't have those yet

    statements.push(QbeStatement::Volatile(QbeInstr::Call(
        "printf".into(),
        vec![
            (QbeType::Long, QbeValue::Global("fmt".into())),
            (QbeType::Word, QbeValue::Temporary("r".into())),
        ],
    )));

    statements.push(QbeStatement::Volatile(QbeInstr::Ret(Some(
        QbeValue::Const(0),
    ))));

    let blocks = vec![QbeBlock {
        label: "start".into(),
        statements,
    }];

    QbeFunction {
        exported: true,
        name: "main".into(),
        return_ty: Some(QbeType::Word),
        arguments: Vec::new(),
        blocks,
    }
}

fn generate_data() -> QbeDataDef {
    let items = vec![
        (
            QbeType::Byte,
            QbeDataItem::Str("One and one make %d!\\n".into()),
        ),
        (QbeType::Byte, QbeDataItem::Const(0)),
    ];
    QbeDataDef {
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
