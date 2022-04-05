//! Test evaluation of pieces of code to make sure things work as expected.

macro_rules! test_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            assert!(compiler::compile($input).is_ok())
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
                "failed to compile with {}",
                result.unwrap_err()
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
