use qbe::*;

// Represents the hello world example from https://c9x.me/compile/

fn generate_add_func(module: &mut Module) {
    let func = module.add_function(
        "add".into(),
        vec![
            (Type::Word, Value::Temporary("a".into())),
            (Type::Word, Value::Temporary("b".into())),
        ],
        Some(Type::Word),
    );

    func.add_block("start".into());
    func.assign_instr(
        Value::Temporary("c".into()),
        Type::Word,
        Instr::Add(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    func.add_instr(Instr::Ret(Some(Value::Temporary("c".into()))));
}

fn generate_main_func(module: &mut Module) {
    let func = module.add_function("main".into(), vec![], Some(Type::Word));

    func.add_block("start".into());
    func.assign_instr(
        Value::Temporary("r".into()),
        Type::Word,
        Instr::Call(
            "add".into(),
            vec![(Type::Word, Value::Const(1)), (Type::Word, Value::Const(1))],
        ),
    );
    // TODO: The example shows a variadic call. We don't have those yet
    func.add_instr(Instr::Call(
        "printf".into(),
        vec![
            (Type::Long, Value::Global("fmt".into())),
            (Type::Word, Value::Temporary("r".into())),
        ],
    ));
    func.add_instr(Instr::Ret(Some(Value::Const(0))));
}

fn generate_data(module: &mut Module) {
    let items = vec![
        (Type::Byte, DataItem::Str("One and one make %d!\\n".into())),
        (Type::Byte, DataItem::Const(0)),
    ];
    module.add_data("fmt".into(), None, items);
}

fn main() {
    let mut module = Module::new();
    generate_add_func(&mut module);
    generate_main_func(&mut module);
    generate_data(&mut module);
    println!("{}", module);
}
