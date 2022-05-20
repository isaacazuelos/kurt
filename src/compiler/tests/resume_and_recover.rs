//! Tests that make sure the compiler can resume compiling code.

use compiler::ModuleBuilder;

macro_rules! test_recover {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let mut builder = ModuleBuilder::default();

            let before = builder.build();

            let result = builder.push_input($input);
            assert!(result.is_err());

            let after = builder.build();

            assert_eq!(
                before, after,
                "before: {:#?}\nafter: {:#?}",
                before, after
            );
        }
    };
}

macro_rules! test_resume {
    ($name: ident, $before: expr, $after: expr) => {
        #[test]
        fn $name() {
            let mut builder = ModuleBuilder::default();

            let before = builder.push_input($before);
            assert!(before.is_ok());

            let after = builder.push_input($after);
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
