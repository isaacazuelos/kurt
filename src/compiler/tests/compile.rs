//! Test the compilation pieces of code to make sure they compile.
//!
//! They're not executed, and the results of compilation aren't verified. These
//! are more just sanity tests to make sure the constructs even compile as
//! expected.

macro_rules! test_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = compiler::Module::try_from($input);
            assert!(result.is_ok(), "failed to compile with {:#?}", result)
        }
    };
}

macro_rules! test_no_compile {
    ($name: ident, $input: expr) => {
        #[test]
        fn $name() {
            let result = compiler::Module::try_from($input);
            assert!(
                result.is_err(),
                "should have failed, but compiled with {:#?}",
                result.unwrap()
            )
        }
    };
}

mod values {
    test_compile! { literal, "1" }
    test_compile! { grouping, "(1)" }
}

mod statements_and_scopes {
    test_compile! { empty, "" }
    test_compile! { empty_statements, ";;" }
    test_compile! { scope, "{1; 2}; {}"}
    test_compile! { binding, "let x = 1;" }
    test_compile! { out_of_scope_shadow, "let x = 0; { let x = 1; }; x" }

    test_no_compile! { out_of_scope, "{ let x = 1; }; x" }
    test_no_compile! { missing_binding, "missing" }
}

mod functions {
    test_compile! { function, "(x) => x" }
    test_compile! { call, "let id = (x) => x; id(1)" }
    test_compile! { capture, "let a = 1; let f = () => a;" }
    test_compile! { rec, "let rec f = (n) => if n == 0 {1} else {n * f(n - 1)};" }

    test_no_compile! { missing_rec, "let f = (n) => if n == 0 {1} else {n * f(n - 1)};" }
}

mod branching {
    test_compile! { if_only, "if true { 1 }" }
    test_compile! { if_else, "if true { 1 } else { 2 }" }
    test_compile! { early_return, "let f = () => {return 7;};"}
    test_compile! { while_loop, "while false { 1 }" }
    test_compile! { loop_loop, "loop { 1 }" }
    test_compile! { loop_break, "loop { break; }" }
    test_compile! { loop_break_value, "loop { break 6; }" }
    test_compile! { loop_continue, "loop { continue; }" }

    test_no_compile! { loop_no_continue_value, "loop { continue 6; }" }
}

mod assignment {
    test_compile! { assignment_to_id, "let x = 1; x = 3;" }
    test_compile! { assignment_to_sub, "let x = 1; x[7] = 2;" }
    test_compile! { assignment_to_capture, "let x = 0; let inc = () => x = x + 1" }
}

mod rec_data {
    // This should work, one day
    test_no_compile! { rec_data, "let rec f = [1, f];" }
}
