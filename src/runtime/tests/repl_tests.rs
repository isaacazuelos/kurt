//! Tests to make sure the interfaces provided to build the repl actually work.
//! It's remarkable how often this breaks.

use compiler::ModuleBuilder;
use runtime::{Exit, VirtualMachine};

#[test]
fn empty() {
    let main = ModuleBuilder::default();
    let mut vm = VirtualMachine::new(main.build());
    let result = vm.start();
    assert_eq!(result.ok(), Some(Exit::Halt))
}

#[test]
fn simple() {
    let main = ModuleBuilder::default().input("1").unwrap();
    let mut vm = VirtualMachine::new(main.build());
    let result = vm.start();
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "1");
}

#[test]
fn simple_resume() {
    let mut main = ModuleBuilder::default().input("1").unwrap();
    let mut vm = VirtualMachine::new(main.build());
    let result = vm.start();
    assert_eq!(result.ok(), Some(Exit::Halt));
    assert_eq!(vm.last_result(), "1");

    main.push_input("2").unwrap();
    vm.reload_main(main.build()).unwrap();
    let result = vm.resume().unwrap();
    assert_eq!(result, Exit::Halt);
    assert_eq!(vm.last_result(), "2");
}

#[test]
fn locals() {
    let mut main = ModuleBuilder::default().input("let x = 1;").unwrap();
    let mut vm = VirtualMachine::new(main.build());
    let result = vm.start();
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
