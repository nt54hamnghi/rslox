use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;

fn write_temp_lox(tempdir: &TempDir, source: &str) -> PathBuf {
    let path = tempdir.path().join("test.lox");
    fs::write(&path, source).expect("should write temp lox file");
    path
}

#[test]
fn test_run_print_multiple_statements_output() {
    let tempdir = TempDir::new("codecrafters-interpreter").expect("should create temp dir");
    let file = write_temp_lox(
        &tempdir,
        r#"
        print "baz"; print false;
        print true;
        print "bar"; print 76;
        "#,
    );

    // Cargo injects this env var for integration tests; it points to the built CLI binary.
    let output = Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg("run")
        .arg(&file)
        .output()
        .expect("binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let actual = stdout.lines().collect::<Vec<_>>();
    let expected = vec!["baz", "false", "true", "bar", "76"];
    assert_eq!(expected, actual);
}

#[test]
fn test_run_print_without_expression_reports_static_error() {
    let tempdir = TempDir::new("codecrafters-interpreter").expect("should create temp dir");
    let file = write_temp_lox(&tempdir, "print;\n");

    // Run the real binary to verify CLI-visible exit code and stderr behavior.
    let output = Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg("run")
        .arg(&file)
        .output()
        .expect("binary should run");

    assert_eq!(Some(65), output.status.code());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("[line 1] Error at ';': Expect expression"));
}
