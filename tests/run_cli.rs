use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rstest::rstest;
use tempdir::TempDir;

fn write_temp_lox(tempdir: &TempDir, source: &str) -> PathBuf {
    // Keep fixture creation in one place so each test only defines source text.
    let path = tempdir.path().join("test.lox");
    fs::write(&path, source).expect("should write temp lox file");
    path
}

fn run_source(source: &str) -> std::process::Output {
    // TempDir is removed automatically when dropped at the end of the helper scope.
    let tempdir = TempDir::new("codecrafters-interpreter").expect("should create temp dir");
    let file = write_temp_lox(&tempdir, source);

    // Cargo injects this env var for integration tests; it points to the built CLI binary.
    Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg("run")
        .arg(&file)
        .output()
        .expect("binary should run")
}

fn assert_success_output(source: &str, expected_stdout: &str) {
    let output = run_source(source);

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert_eq!(expected_stdout, stdout);

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(
        stderr.is_empty(),
        "successful execution should not write stderr"
    );
}

#[test]
fn test_print_requires_expression_reports_static_error_and_exit_65() {
    let output = run_source("print;\n");

    assert_eq!(Some(65), output.status.code());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("[line 1] Error at ';': Expect expression"));
}

#[rstest]
#[case(
    r#"
    // Multiple statements in a single line should work
    print "baz"; print false;
    print true;
    print "bar"; print 35;
    "#,
    "baz\nfalse\ntrue\nbar\n35\n"
)]
#[case(
    r#"
    // Leading whitespace should be ignored
    print 92;
        print 92 + 30;
            print 92 + 30 + 74;
    "#,
    "92\n122\n196\n"
)]
fn test_multiple_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program tests that statements are executed
    // even if they don't have any side effects
    (37 + 48 - 85) > (93 - 37) * 2;
    print !true;
    "bar" + "quz" + "world" == "barquzworld";
    print !true;
    "#,
    "false\nfalse\n"
)]
#[case(
    r#"
    // This program tests statements that don't have any side effects
    80 - 50 >= -95 * 2 / 95 + 16;
    true == true;
    ("bar" == "baz") == ("quz" != "world");
    print true;
    "#,
    "true\n"
)]
fn test_expression_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // Variables are initialized to the correct value
    var quz = 10;
    print quz;
    "#,
    "10\n"
)]
#[case(
    r#"
    // Declares multiple variables and prints arithmetic on them
    var baz = 41;
    var bar = 41;
    print baz + bar;
    var hello = 41;
    print baz + bar + hello;
    "#,
    "82\n123\n"
)]
#[case(
    r#"
    // Assigns arithmetic expression to variable, then prints it
    var foo = (8 * (79 + 79)) / 4 + 79;
    print foo;
    "#,
    "395\n"
)]
#[case(
    r#"
    // Declares variables and performs operations on them
    var quz = 94;
    var foo = quz;
    print foo + quz;
    "#,
    "188\n"
)]
fn test_variable_declarations_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // Declares a variable without initializing it, so its value is nil.
    var quz;
    print quz;
    "#,
    "nil\n"
)]
#[case(
    r#"
    // Declares an initialized variable and an uninitialized variable.
    var quz = "bar";
    var baz;
    print baz;
    "#,
    "nil\n"
)]
#[case(
    r#"
    // Multiple uninitialized variables should default to nil.
    var bar = 29;
    var quz;
    var world;
    print quz;
    "#,
    "nil\n"
)]
#[case(
    r#"
    // Uninitialized variables remain nil alongside initialized ones.
    var bar = 33 + 87 * 95;
    print bar;
    var quz = 87 * 95;
    print bar + quz;
    var world;
    print world;
    "#,
    "8298\n16563\nnil\n"
)]
fn test_variable_initialization_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    var world = "before";
    print world;
    var world = "after";
    print world;
    "#,
    "before\nafter\n"
)]
#[case(
    r#"
    var hello = "after";
    var hello = "before";
    // Using a previously declared variable's value to initialize a new variable should work.
    var hello = hello;
    print hello;
    "#,
    "before\n"
)]
#[case(
    r#"
    // This program declares and initializes multiple variables and prints their values.
    var bar = 2;
    print bar;
    var bar = 3;
    print bar;
    var baz = 5;
    print baz;
    var bar = baz;
    print bar;
    "#,
    "2\n3\n5\n5\n"
)]
fn test_variable_redeclaration_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    var baz;
    baz = 1;
    print baz;
    // The assignment operator should return the value that was assigned.
    print baz = 2;
    print baz;
    "#,
    "1\n2\n2\n"
)]
#[case(
    r#"
    // This program tests that the assignment operator works on any declared variable.
    var baz = 28;
    var quz = 28;
    quz = baz;
    baz = quz;
    print baz + quz;
    "#,
    "56\n"
)]
#[case(
    r#"
    var hello;
    var baz;

    // The assignment operator should return the value that was assigned.
    hello = baz = 71 + 94 * 43;
    print hello;
    print baz;
    "#,
    "4113\n4113\n"
)]
#[case(
    r#"
    var foo = 63;
    var bar;
    var quz;

    // The assignment operator should return the value that was assigned.
    foo = bar = quz = foo * 2;
    print foo;
    print bar;
    print bar;
    "#,
    "126\n126\n126\n"
)]
fn test_assignment_operation_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    print "the expression below is invalid";
    43 + "hello";
    print "this should not be printed";
    "#,
    "the expression below is invalid",
    Some("this should not be printed")
)]
#[case(
    r#"
    print "56" + "hello";
    print false * (92 + 96);
    "#,
    "56hello",
    None
)]
fn test_runtime_errors_report_stderr_and_exit_70(
    #[case] source: &str,                   // Full program text passed to the CLI.
    #[case] expected_stdout_fragment: &str, // Output that must appear before the runtime error.
    #[case] forbidden_stdout_fragment: Option<&str>, // Output that must not appear after the error.
) {
    // Run the real binary to verify CLI-visible exit code and stderr behavior.
    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains(expected_stdout_fragment));
    if let Some(forbidden) = forbidden_stdout_fragment {
        assert!(!stdout.contains(forbidden));
    }

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Operands must be numbers."));
}

#[rstest]
#[case(
    r#"
    // This program tries to access a variable before it is declared
    // It leads to a runtime error
    print 34;
    print x;
    "#,
    Some("34"),
    None,
    "Undefined variable 'x'."
)]
#[case(
    r#"
    // This program tries to access a variable before it is declared
    // It leads to a runtime error
    var world = 56;
    print bar;
    "#,
    None,
    None,
    "Undefined variable 'bar'."
)]
#[case(
    r#"
    // This program tries to access a variable before it is declared
    // It leads to a runtime error
    var hello = 73;
    var result = (hello + quz) / foo;
    print result;
    "#,
    None,
    None,
    "Undefined variable 'quz'."
)]
#[case(
    r#"
    // This program tries to access a variable before it is declared
    // It leads to a runtime error
    var bar = 73;
    var world = 95;
    var hello = 54;
    print bar + world + hello + quz; print 30;
    "#,
    None,
    Some("30"),
    "Undefined variable 'quz'."
)]
#[case(
    r#"
    // As hello is not declared before
    var baz = hello; // expect runtime error
    "#,
    None,
    None,
    "Undefined variable 'hello'."
)]
fn test_undefined_variable_runtime_errors_report_stderr_and_exit_70(
    #[case] source: &str, // Full program text passed to the CLI.
    #[case] expected_stdout_fragment: Option<&str>, // Output that should appear before failure, if any.
    #[case] forbidden_stdout_fragment: Option<&str>, // Output that must not appear because execution stops on error.
    #[case] expected_stderr_fragment: &str,          // Error text that must be reported on stderr.
) {
    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    if let Some(expected) = expected_stdout_fragment {
        assert!(stdout.contains(expected));
    }
    if let Some(forbidden) = forbidden_stdout_fragment {
        assert!(!stdout.contains(forbidden));
    }

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains(expected_stderr_fragment));
}
