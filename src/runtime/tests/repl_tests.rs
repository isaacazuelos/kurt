//! Tests to make sure the interfaces provided to build the repl actually work.
//! It's remarkable how often this breaks.

use compiler::ModuleBuilder;
use runtime::{Exit, VirtualMachine};

#[test]
fn empty() {
    let main = ModuleBuilder::default();
    let mut vm = VirtualMachine::default();
    let result = vm.load(main.build());
    assert_eq!(result.ok(), Some(Exit::Halt))
}

#[test]
fn simple() {
    let main = ModuleBuilder::default().input("1").unwrap();
    let mut vm = VirtualMachine::default();
    let result = vm.load(main.build());
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "1");
}

#[test]
fn empty_resume() {
    let mut builder = ModuleBuilder::default();
    let mut vm = VirtualMachine::default();

    let main = builder.build();
    let result = vm.load(main);
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "()");

    builder.push_input("2").unwrap();
    let new_main = builder.build();
    vm.reload_main(new_main).unwrap();
    let result = vm.resume().unwrap();
    assert_eq!(result, Exit::Halt);
    assert_eq!(vm.last_result(), "2");
}

#[test]
fn simple_resume() {
    let mut main = ModuleBuilder::default().input("1").unwrap();
    let mut vm = VirtualMachine::default();

    let result = vm.load(main.build());
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "1");

    main.push_input("2").unwrap();
    vm.reload_main(main.build()).unwrap();
    let result = vm.resume().unwrap();
    assert_eq!(result, Exit::Halt);
    assert_eq!(vm.last_result(), "2");
}

#[test]
fn bindings() {
    let mut main = ModuleBuilder::default().input("let x = 1;").unwrap();
    let mut vm = VirtualMachine::default();

    let result = vm.load(main.build());
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "()");

    main.push_input("let y = 2; y").unwrap();
    vm.reload_main(main.build()).unwrap();
    let result = vm.resume().unwrap();
    assert_eq!(result, Exit::Halt);
    assert_eq!(vm.last_result(), "2");

    main.push_input("x").unwrap();
    vm.reload_main(main.build()).unwrap();
    let result = vm.resume().unwrap();
    assert_eq!(result, Exit::Halt);
    assert_eq!(vm.last_result(), "1");
}
