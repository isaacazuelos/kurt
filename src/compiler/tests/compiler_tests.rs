//! Test evaluation of pieces of code to make sure things work as expected.

macro_rules! test_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = compiler::compile($input);
            assert!(result.is_ok(), "failed to compile with {:#?}", result)
        }
    };
}

macro_rules! test_no_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = compiler::compile($input);
            assert!(
                result.is_err(),
                "should have failed, but compiled with {}",
                result.unwrap()
            )
        }
    };
}

test_compile! { empty, "" }
test_compile! { empty_statements, ";;" }
test_compile! { literal, "1" }
test_compile! { binding, "let x = 1;" }
test_no_compile! { missing_binding, "missing" }
test_compile! { scope, "{1; 2}; {}"}
test_no_compile! { out_of_scope, "{ let x = 1; }; x" }
test_compile! { out_of_scope_shadow, "let x = 0; { let x = 1; }; x" }
test_compile! { grouping, "(1)" }
test_compile! { function, "(x) => x" }
test_compile! { call, "let id = (x) => x; id(1)" }
test_no_compile! { capture, "let a = 1; let f = () => a;" }