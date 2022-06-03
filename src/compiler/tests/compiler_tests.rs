//! Test the compilation pieces of code to make sure they compile.
//!
//! They're not executed, and the results of compilation aren't verified. These
//! are more just sanity tests.

use compiler::Module;

macro_rules! test_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = Module::try_from($input);
            assert!(result.is_ok(), "failed to compile with {:#?}", result)
        }
    };
}

macro_rules! test_no_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = Module::try_from($input);
            assert!(
                result.is_err(),
                "should have failed, but compiled with {:#?}",
                result.unwrap()
            )
        }
    };
}

test_compile! { empty, "" }
test_compile! { empty_statements, ";;" }
test_compile! { literal, "1" }
test_compile! { binding, "let x = 1;" }
test_compile! { scope, "{1; 2}; {}"}
test_compile! { out_of_scope_shadow, "let x = 0; { let x = 1; }; x" }
test_compile! { grouping, "(1)" }
test_compile! { function, "(x) => x" }
test_compile! { call, "let id = (x) => x; id(1)" }
test_compile! { capture, "let a = 1; let f = () => a;" }
test_compile! { if_only, "if true { 1 }" }
test_compile! { if_else, "if true { 1 } else { 2 }" }
test_compile! { rec, "let rec f = (n) => if n == 0 {1} else {n * f(n - 1)};" }

test_no_compile! { out_of_scope, "{ let x = 1; }; x" }
test_no_compile! { missing_binding, "missing" }
test_no_compile! { missing_rec, "let f = (n) => if n == 0 {1} else {n * f(n - 1)};" }

// For now, this won't work.
test_no_compile! { rec_data, "let rec f = [1, f];" }

test_compile! { early_return, "let f = () => {return 7;};"}
