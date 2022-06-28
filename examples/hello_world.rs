use qbe::*;

// Represents the hello world example from https://c9x.me/compile/

fn generate_add_func(module: &mut Module) {
    let mut func = Function::new(
        Linkage::private(),
        "add",
        vec![
            (Type::Word, Value::Temporary("a".into())),
            (Type::Word, Value::Temporary("b".into())),
        ],
        Some(Type::Word),
    );

    func.add_block("start");
    func.assign_instr(
        Value::Temporary("c".into()),
        Type::Word,
        Instr::Add(Value::Temporary("a".into()), Value::Temporary("b".into())),
    );
    func.add_instr(Instr::Ret(Some(Value::Temporary("c".into()))));

    module.add_function(func);
}

fn generate_main_func(module: &mut Module) {
    let mut func = Function::new(Linkage::public(), "main", Vec::new(), Some(Type::Word));

    func.add_block("start");
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

    module.add_function(func);
}

fn generate_data(module: &mut Module) {
    let items = vec![
        (Type::Byte, DataItem::Str("One and one make %d!\\n".into())),
        (Type::Byte, DataItem::Const(0)),
    ];
    let data = DataDef::new(Linkage::private(), "fmt", None, items);
    module.add_data(data);
}

fn main() {
    let mut module = Module::new();
    generate_add_func(&mut module);
    generate_main_func(&mut module);
    generate_data(&mut module);
    println!("{}", module);
}
