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
