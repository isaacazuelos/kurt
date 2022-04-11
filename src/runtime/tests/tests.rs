//! Test evaluation of pieces of code to make sure things work as expected.

macro_rules! test_eval {
    ($name: ident, $input: expr, $expected: expr) => {
        #[test]
        fn $name() {
            let obj = compiler::compile($input).unwrap();
            let mut rt = runtime::Runtime::new();
            assert!(rt.load(obj).is_ok());
            assert!(rt.start().is_ok());
            assert_eq!($expected, rt.last_result());
        }
    };
}

// Literal values
test_eval! { literal_char, "'a'", "'a'" }
test_eval! { literal_boolean, "true", "true" }
test_eval! { literal_number, "99", "99" }
test_eval! { literal_float, "1.5", "1.5" }
test_eval! { literal_string, r#" "Hello, world!" "#, r#""Hello, world!""# }
test_eval! { literal_keyword, " :hello ", ":hello" }

// Statement sequences
test_eval! { empty_input, "", "()" }
test_eval! { semicolons_only, ";;;;;", "()" }
test_eval! { no_trailing_semicolon, "1;2;3", "3" }
test_eval! { trailing_semicolon, "1;2;3;", "()" }

// Local `let` bindings
test_eval! { let_return_unit, "let x = 1", "()" }
test_eval! { let_local_lookup, "let x = 1; x", "1" }
test_eval! { let_local_twice, "let x = 1; let y = 2; x", "1" }
test_eval! { let_local_shadow, "let x = 1; let x = 2; x", "2" }
test_eval! { let_local_complex, "let x = 1; 100; ;;; let y = 2; x; 10; y", "2" }

// Scopes
test_eval! { scope_empty, "{ ; }", "()" }
test_eval! { scope_with_value, "{ 1 }", "1" }
test_eval! { scope_with_trailing, "{ 1; }", "()" }
test_eval! { scope_with_bindings, "{ let x = 1; x }", "1" }

// Functions
test_eval! { simple_function, "let x = () => {;}", "()" }
test_eval! { call, "((x) => x)(10)", "10" }
test_eval! { call_multiple_args, "((a, b) => b)(10, 20)", "20" }
