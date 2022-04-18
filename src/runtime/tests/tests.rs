//! Test evaluation of pieces of code to make sure things work as expected.

macro_rules! test_eval {
    ($name: ident, $input: expr, $expected: expr) => {
        #[test]
        fn $name() {
            let obj = compiler::compile($input).unwrap();
            let mut rt = runtime::Runtime::new();
            assert!(rt.load(obj).is_ok());
            let exit = rt.start();
            assert!(exit.is_ok(), "exited with {:?}", exit);
            let actual = rt.last_result();
            assert_eq!($expected, actual, "got {}", actual);
        }
    };
}

mod literals {
    test_eval! { literal_char, "'a'", "'a'" }
    test_eval! { literal_boolean, "true", "true" }
    test_eval! { literal_number, "99", "99" }
    test_eval! { literal_float, "1.5", "1.5" }
    test_eval! { literal_string, r#" "Hello, world!" "#, r#""Hello, world!""# }
    test_eval! { literal_keyword, " :hello ", ":hello" }
}

mod statement_sequences {
    test_eval! { empty_input, "", "()" }
    test_eval! { semicolons_only, ";;;;;", "()" }
    test_eval! { no_trailing_semicolon, "1;2;3", "3" }
    test_eval! { trailing_semicolon, "1;2;3;", "()" }
}

mod let_bindings {
    test_eval! { let_return_unit, "let x = 1", "()" }
    test_eval! { let_local_lookup, "let x = 1; x", "1" }
    test_eval! { let_local_twice, "let x = 1; let y = 2; x", "1" }
    test_eval! { let_local_shadow, "let x = 1; let x = 2; x", "2" }
    test_eval! { let_local_complex, "let x = 1; 100; ;;; let y = 2; x; 10; y", "2" }
}

mod scope {
    test_eval! { scope_empty, "{ ; }", "()" }
    test_eval! { scope_with_value, "{ 1 }", "1" }
    test_eval! { scope_with_trailing, "{ 1; }", "()" }
    test_eval! { scope_with_bindings, "{ let x = 1; x }", "1" }
}

mod functions {
    test_eval! { simple_function, "let x = () => {;}", "()" }
    test_eval! { call, "((x) => x)(10)", "10" }
    test_eval! { call_multiple_args, "((a, b) => b)(10, 20)", "20" }
    test_eval! { nested_call, "let snd = (x, y) => y; snd(snd(0, 1), snd(2, 3))", "3"}
}

mod lists {
    test_eval! { list_empty, "[]", "[ ]" }
    test_eval! { list, "[1,2,3]", "[ 1, 2, 3, ]" }
}

mod conditionals {
    test_eval! { if_only, "if true { 10 }", "10" }
    test_eval! { if_only_f, "if false { 10 }", "()" }
    test_eval! { if_else_t, "if true { 10 } else { 20 }", "10" }
    test_eval! { if_else_f, "if false { 10 } else { 20 }", "20" }
    // This one caught a bug in the GC, somehow.
    test_eval! { if_else_gc_bug, "if true { :t } else { :f }", ":t" }
}

mod primitive_operations {
    test_eval! { not, "!true", "false" }
    test_eval! { addition, "1 + 2", "3" }
    test_eval! { subtraction, "4 - 2", "2" }
    test_eval! { mul, "8 * 8", "64" }
    test_eval! { div, "100 / 3", "33" }
    test_eval! { pow, "2^10", "1024" }
    test_eval! { modulus, "125 % 2", "1" }
    test_eval! { bit_and, "7 & 15", "7" }
    test_eval! { bit_or, "7 | 16", "23" }
    test_eval! { bit_xor, "1 ⊕ 3", "2" }
    test_eval! { shift_left, "2 << 1", "4" }
    test_eval! { shift_arithmetic, "-4 >>> 1", "-2" }
    test_eval! { eq, "1 == 1", "true" }
    test_eval! { ne, "1 != 1", "false" }
    test_eval! { lt, "2 < 4", "true" }
    test_eval! { le, "2 <= 2", "true" }
    test_eval! { gt, "2 > 4", "false" }
    test_eval! { ge, "2 >= 1", "true" }
}
