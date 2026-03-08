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

#[rstest]
#[case(
    r#"
    print "baz"; print false;
    print true;
    print "bar"; print 76;
    "#,
    &["baz", "false", "true", "bar", "76"]
)]
#[case(
    r#"
    (77 + 88 - 51) > (64 - 77) * 2;
    print !false;
    "hello" + "world" + "baz" == "helloworldbaz";
    print !false;
    "#,
    &["true", "true"]
)]
#[case(
    r#"
    51 - 60 >= -76 * 2 / 76 + 29;
    false == false;
    ("baz" == "hello") == ("world" != "foo");
    print false;
    "#,
    &["false"]
)]
fn test_run_success_cases(#[case] source: &str, #[case] expected_stdout: &[&str]) {
    let output = run_source(source);

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let actual = stdout.lines().collect::<Vec<_>>();
    assert_eq!(expected_stdout, actual);
}

#[test]
fn test_run_print_without_expression_reports_static_error() {
    let output = run_source("print;\n");

    assert_eq!(Some(65), output.status.code());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("[line 1] Error at ';': Expect expression"));
}

// #[test]
// fn test_run_print_without_expression_reports_static_error() {
//     let output = run_source("print;\n");

//     let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
//     assert_eq!("[line 1] Error at ';': Expect expression\n", stderr);
// }

#[rstest]
#[case(
    r#"
    print "the expression below is invalid";
    39 + "world";
    print "this should not be printed";
    "#,
    "the expression below is invalid",
    Some("this should not be printed")
)]
#[case(
    r#"
    print "62" + "foo";
    print true * (64 + 13);
    "#,
    "62foo",
    None
)]
fn test_run_runtime_error_cases(
    #[case] source: &str,
    #[case] expected_stdout_fragment: &str,
    #[case] forbidden_stdout_fragment: Option<&str>,
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
