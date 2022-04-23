//! Tests that make sure the compiler can resume compiling code.

use compiler::*;
use syntax::{Module, Parse};

macro_rules! test_recover {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let mut compiler = Compiler::default();

            let before = compiler.build();
            assert!(before.is_ok());

            let module = Module::parse($input).unwrap();

            let result = compiler.push(&module);
            assert!(result.is_err());

            let after = compiler.build();
            assert!(after.is_ok());

            let b = before.unwrap();
            let a = after.unwrap();

            assert_eq!(b, a, "before: {:#?}\nafter: {:#?}", b, a);
        }
    };
}

macro_rules! test_resume {
    ($name: ident, $before: expr, $after: expr) => {
        #[test]
        fn $name() {
            let mut compiler = Compiler::default();

            assert!(compiler.build().is_ok());

            let module = Module::parse($before).unwrap();
            let before = compiler.push(&module);
            assert!(before.is_ok());

            let module = Module::parse($after).unwrap();
            let after = compiler.push(&module);
            assert!(after.is_ok());
        }
    };
}

test_resume!(empty, "", "");
test_resume!(simple, "1", "2");
test_resume!(bindings, "let x = 1;", "x");
test_recover!(constants, "1; 2; 3; missing");
test_recover!(scope, "{ let x = true; }; x");
test_recover!(function_scoping, "() => { missing }; ");
