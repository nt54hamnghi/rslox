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

fn assert_static_error(source: &str, expected_stderr_fragment: &str) {
    let output = run_source(source);

    assert_eq!(Some(65), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.is_empty(), "static errors should not write stdout");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains(expected_stderr_fragment));
}

#[test]
fn test_print_requires_expression_reports_static_error_and_exit_65() {
    let output = run_source("print;\n");

    assert_eq!(Some(65), output.status.code());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("[line 1] Error at ';': Expect expression"));
}

#[test]
fn test_block_requires_closing_brace_reports_static_error_and_exit_65() {
    let source = r#"
    {
        var foo = 42;
        var quz = 42;
        {
            print foo + quz;
        // Missing closing curly brace
        // Expect compile error
    }
    "#;

    let output = run_source(source);

    assert_eq!(Some(65), output.status.code());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("[line 10] Error at end: Expect '}' after block."));
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
    // This program tests that curly braces can be
    // used to group multiple statements into blocks
    {
        var quz = "bar";
        print quz;
    }
    "#,
    "bar\n"
)]
#[case(
    r#"
    // This program tests that blocks can be used
    // to group statements and variables
    // creating local scopes
    {
        var quz = "before";
        print quz;
    }
    {
        var quz = "after";
        print quz;
    }
    "#,
    "before\nafter\n"
)]
#[case(
    r#"
    // This program tests that scopes can be nested
    {
        var world = 88;
        {
            var bar = 88;
            print bar;
        }
        print world;
    }
    "#,
    "88\n88\n"
)]
fn test_block_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    var bar = (20 * 77) - 63;
    {
        // Local scope should be created
        var hello = "world" + "42";
        print hello;
    }
    print bar;
    "#,
    "world42\n1477\n"
)]
#[case(
    r#"
    // This program tests variable shadowing
    // across nested scopes
    {
        var world = "before";
        {
            var world = "after";
            print world;
        }
        print world;
    }
    "#,
    "after\nbefore\n"
)]
#[case(
    r#"
    // This program creates nested scopes and tests
    // local scopes and variable shadowing
    var world = "global world";
    var quz = "global quz";
    var foo = "global foo";
    {
      var world = "outer world";
      var quz = "outer quz";
      {
        var world = "inner world";
        print world;
        print quz;
        print foo;
      }
      print world;
      print quz;
      print foo;
    }
    print world;
    print quz;
    print foo;
    "#,
    "inner world\nouter quz\nglobal foo\nouter world\nouter quz\nglobal foo\nglobal world\nglobal quz\nglobal foo\n"
)]
fn test_scopes_success(#[case] source: &str, #[case] expected_stdout: &str) {
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
#[case(
    r#"// First declaration of variable 'a' in global
// scope
var a = "value";

// Redeclaring 'a' with its own value should be
// allowed in global scope
var a = a;
print a; // this should print "value"
"#,
    "value\n"
)]
fn test_variable_redeclaration_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"// Declare outer variable 'a' in global scope
var a = "outer";

{
  // Attempting to declare local variable'a'
  // initialized with itself
  var a = a; // expect compile error
}
"#,
    "[line 7] Error at 'a': Can't read local variable in its own initializer."
)]
#[case(
    r#"// Helper function that simply returns its argument
fun returnArg(arg) {
  return arg;
}

// Declare global variable 'b'
var b = "global";

{
  // Local variable declaration
  var a = "first";

  // Attempting to initialize local variable 'b'
  // using local variable 'b'
  // through a function call
  var b = returnArg(b); // expect compile error
  print b;
}

var b = b + " updated";
print b;
"#,
    "[line 16] Error at 'b': Can't read local variable in its own initializer."
)]
#[case(
    r#"fun outer() {
  // Declare variable 'a' in outer function scope
  var a = "outer";

  // Inner function with its own scope
  fun inner() {
    // Attempting to declare local 'a' initialized
    // with itself
    var a = a; // expect compile error
    print a;
  }

  inner();
}

outer();
"#,
    "[line 9] Error at 'a': Can't read local variable in its own initializer."
)]
fn test_self_initialization_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
}

#[rstest]
#[case(
    r#"{
  var a = "value";

  // Attempting to redeclare 'a' in the same scope
  var a = "other"; // expect compile error
}
"#,
    "[line 5] Error at 'a': Already a variable with this name in this scope."
)]
#[case(
    r#"// Function parameters are considered variables in
// the function's scope
fun foo(a) {
  // Attempting to declare a variable with same
  // name as parameter
  var a; // expect compile error
}
"#,
    "[line 6] Error at 'a': Already a variable with this name in this scope."
)]
#[case(
    r#"// Function parameters must have unique names
fun foo(arg, arg) { // expect compile error
  // Function body is irrelevant as the error
  // occurs in parameter list
  "body";
}
"#,
    "[line 2] Error at 'arg': Already a variable with this name in this scope."
)]
#[case(
    r#"// Due to the compile error on line 17
// Nothing should be printed
var a = "1";
print a;

var a;
print a;

var a = "2";
print a;

{
  // First declaration in local scope
  var a = "1";

  // Attempting to redeclare in local scope
  var a = "2"; // This should be a compile error
  print a;
}
"#,
    "[line 17] Error at 'a': Already a variable with this name in this scope."
)]
fn test_variable_redeclaration_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
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
    // This should print the string if the condition
    // evaluates to True
    if (false) print "foo";
    "#,
    ""
)]
#[case(
    r#"
    // This should print "block body" if the condition
    // evaluates to True
    if (true) {
      print "block body";
    }
    "#,
    "block body\n"
)]
#[case(
    r#"
    // This program tests whether the assignment
    // operation returns the value assigned.
    // The if condition should evaluate to true and
    // the inner boolean expression must be printed.
    // So, in this case the if condition evaluates to
    //true and prints the inner boolean expression
    var a = false;
    if (a = true) {
      print (a == true);
    }
    "#,
    "true\n"
)]
#[case(
    r#"
    // This program should print a different string
    // based on the value of age
    var stage = "unknown";
    var age = 44;
    if (age < 18) { stage = "child"; }
    if (age >= 18) { stage = "adult"; }
    print stage;

    var isAdult = age >= 18;
    if (isAdult) { print "eligible for voting"; }
    if (!isAdult) { print "not eligible for voting"; }
    "#,
    "adult\neligible for voting\n"
)]
fn test_if_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program uses a random boolean to decide
    // which branch to execute,
    // and then prints the appropriate string
    if (true) print "if"; else print "else";
    "#,
    "if\n"
)]
#[case(
    r#"
    // This program initializes age with a random
    // integer and then prints "adult"
    // if the age is greater than 18, otherwise it
    // prints "child"
    var age = 40;
    if (age > 18) print "adult"; else print "child";
    "#,
    "adult\n"
)]
#[case(
    r#"
    // This program uses a random boolean to decide
    // which branch to execute,
    // and then prints the appropriate string
    if (false) {
      print "if block";
    } else print "else statement";

    if (false) print "if statement"; else {
      print "else block";
    }
    "#,
    "else statement\nelse block\n"
)]
#[case(
    r#"
    // This program converts a random integer from
    // Celsius to Fahrenheit
    // and prints the result. It also prints a message
    // based on the temperature.
    var celsius = 52;
    var fahrenheit = 0;
    var isHot = false;

    {
      fahrenheit = celsius * 9 / 5 + 32;
      print celsius; print fahrenheit;

      if (celsius > 30) {
        isHot = true;
        print "It's a hot day. Stay hydrated!";
      } else {
        print "It's cold today. Wear a jacket!";
      }

      if (isHot) { print "Use sunscreen!"; }
    }
    "#,
    "52\n125.6\nIt's a hot day. Stay hydrated!\nUse sunscreen!\n"
)]
fn test_else_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program uses a random boolean to decide
    // which branch to execute,
    // and then prints the appropriate string
    if (true) print "if branch";
    else if (true) print "else-if branch";
    "#,
    "if branch\n"
)]
#[case(
    r#"
    // This program uses a random boolean to decide
    // which branch to execute,
    // and then prints the appropriate string
    if (true) {
      print "quz";
    } else if (true) print "quz";

    if (true) print "quz"; else if (true) {
      print "quz";
    }
    "#,
    "quz\nquz\n"
)]
#[case(
    r#"
    // This program uses multiple if statements to
    // categorize a person
    // into different life stages based on their age
    var age = 86;
    var stage = "unknown";
    if (age < 18) { stage = "child"; }
    else if (age >= 18) { stage = "adult"; }
    else if (age >= 65) { stage = "senior"; }
    else if (age >= 100) { stage = "centenarian"; }
    print stage;
    "#,
    "adult\n"
)]
#[case(
    r#"
    // This program uses multiple if statements to
    // determine eligibility for
    // voting, driving, and drinking based on a random
    // integer age
    var age = 65;

    var isAdult = age >= 18;
    if (isAdult) { print "eligible for voting: true"; }
    else { print "eligible for voting: false"; }

    if (age < 16) { print "not eligible for driving"; }
    else if (age < 18) { print "learner's permit"; }
    else { print "eligible for driving"; }

    if (age >= 21) { print "eligible for drinking"; }
    else { print "not eligible for drinking"; }
    "#,
    "eligible for voting: true\neligible for driving\neligible for drinking\n"
)]
fn test_else_if_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program uses nested if statements to print
    // a message
    if (true) if (true) print "nested true";
    "#,
    "nested true\n"
)]
#[case(
    r#"
    // This program uses nested if statements to print
    // a message
    if (true) {
      if (true) print "foo"; else print "foo";
    }
    "#,
    "foo\n"
)]
#[case(
    r#"
    // This program categorizes a person into
    // different life stages based on their age
    // Then based on the age, it prints a message
    // about the person's eligibility for voting,
    // driving, and drinking
    var stage = "unknown";
    var age = 34;
    if (age < 18) {
        if (age < 13) { stage = "child"; }
        else if (age < 16) {
            stage = "young teenager";
        }
        else { stage = "teenager"; }
    }
    else if (age < 65) {
        if (age < 30) { stage = "young adult"; }
        else if (age < 50) { stage = "adult"; }
        else { stage = "middle-aged adult"; }
    }
    else { stage = "senior"; }
    print stage;

    var isAdult = age >= 18;
    if (isAdult) {
        print "eligible for voting: true";
        if (age < 25) {
            print "first-time voter: likely";
        }
        else { print "first-time voter: unlikely"; }
    }
    else { print "eligible for voting: false"; }

    if (age < 16) { print "not eligible for driving"; }
    else if (age < 18) {
        print "eligible for driving: learner's permit";
        if (age < 17) {
            print "supervised driving required";
        }
        else {
            print "driving allowed with restrictions";
        }
    }
    else { print "eligible for driving"; }

    if (age < 21) {
        print "not eligible for drinking";
    }
    else {
        print "eligible for drinking";
        if (age < 25) {
            print "remember: drink responsibly!";
        }
    }
    "#,
    "adult\neligible for voting: true\nfirst-time voter: unlikely\neligible for driving\neligible for drinking\n"
)]
#[case(
    r#"
    // This program uses nested if statements to print
    // a message
    if (true) if (false) print "foo";
    else print "hello";
    "#,
    "hello\n"
)]
fn test_nested_if_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // The logical OR operator should return the first
    // value that is truthy
    if (false or "ok") print "hello";
    if (nil or "ok") print "hello";

    if (false or false) print "bar";
    if (true or "bar") print "bar";

    if (30 or "quz") print "quz";
    if ("quz" or "quz") print "quz";
    "#,
    "hello\nhello\nbar\nquz\nquz\n"
)]
#[case(
    r#"
    // This program uses the logical OR operator to
    // print the first value that is truthy
    print 77 or true;
    print false or 77;
    print false or false or true;

    print false or false;
    print false or false or false;
    print true or true or true or true;
    "#,
    "77\n77\ntrue\nfalse\nfalse\ntrue\n"
)]
#[case(
    r#"
    // This program relies on the fact that
    // assignments return the assigned value
    // And that the logical OR operator short-circuits
    // So, if the first assignment is truthy, it
    // wouldn't proceed to the subsequent assignments
    // And then prints the assigned values
    var a = "foo";
    var b = "foo";
    (a = false) or (b = true) or (a = "foo");
    print a;
    print b;
    "#,
    "false\ntrue\n"
)]
#[case(
    r#"
    // This program uses if conditions to get the stage
    // of a person's life based on their age, and then
    // prints if they are eligible for voting
    var stage = "unknown";
    var age = 65;
    if (age < 18) { stage = "child"; }
    if (age >= 18) { stage = "adult"; }
    print stage;

    var isAdult = age >= 18;
    if (isAdult) { print "eligible for voting"; }
    if (!isAdult) { print "not eligible for voting"; }
    "#,
    "adult\neligible for voting\n"
)]
fn test_logical_or_operator_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // The logical AND operator should return the
    // first falsy value
    if (false and "bad") print "bar";
    if (nil and "bad") print "bar";

    // If all values are truthy, it returns the last
    // value
    if (true and "hello") print "hello";
    if (24 and "baz") print "baz";
    if ("baz" and "baz") print "baz";
    if ("" and "world") print "world";
    "#,
    "hello\nbaz\nbaz\nworld\n"
)]
#[case(
    r#"
    // This program uses the logical AND operator to
    // print the first falsy value
    // Or the last value if all values are truthy
    print false and 1;
    print true and 1;
    print 28 and "quz" and false;

    print 28 and true;
    print 28 and "quz" and 28;
    "#,
    "false\n1\nfalse\ntrue\n28\n"
)]
#[case(
    r#"
    // This program relies on the fact that
    // assignments return the assigned value
    // And that the logical AND operator short-circuits
    // So, when it encounters a falsy value, it
    // wouldn't proceed to the subsequent assignments
    // And then prints the assigned values
    var a = "hello";
    var b = "hello";
    (a = true) and (b = false) and (a = "bad");
    print a;
    print b;
    "#,
    "true\nfalse\n"
)]
#[case(
    r#"
    // This program uses if conditions to get the stage
    // of a person's life based on their age, and then
    // prints if they are eligible for voting
    var stage = "unknown";
    var age = 40;
    if (age < 18) { stage = "child"; }
    if (age >= 18) { stage = "adult"; }
    print stage;

    var isAdult = age >= 18;
    if (isAdult) { print "eligible for voting"; }
    if (!isAdult) { print "not eligible for voting"; }
    "#,
    "adult\neligible for voting\n"
)]
fn test_logical_and_operator_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program uses a while loop to print the
    // numbers from 0 to N
    // The assignment operation returns the assigned
    // value
    var hello = 0;
    while (hello < 3) print hello = hello + 1;
    "#,
    "1\n2\n3\n"
)]
#[case(
    r#"
    // This program uses a while loop to print the
    // numbers from 0 to 3
    // The statement inside the block is executed
    // every time the loop condition is true
    var foo = 0;
    while (foo < 3) {
      print foo;
      foo = foo + 1;
    }
    "#,
    "0\n1\n2\n"
)]
#[case(
    r#"
    // This program uses a while loop to calculate the
    // factorial of 5
    // The first while loop never runs because the
    // condition is false
    while (false) { print "should not print"; }

    var product = 1;
    var i = 1;

    while (i <= 5) {
      product = product * i;
      i = i + 1;
    }

    print "Factorial of 5: "; print product;
    "#,
    "Factorial of 5: \n120\n"
)]
#[case(
    r#"
    // This program uses a while loop to generate and
    // print the first N Fibonacci numbers
    var n = 10;
    var fm = 0;
    var fn = 1;
    var index = 0;

    while (index < n) {
        print fm;
        var temp = fm;
        fm = fn;
        fn = temp + fn;
        index = index + 1;
    }
    "#,
    "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n"
)]
fn test_while_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program defines a simple function that
    // doesn't take any arguments
    // and then invokes the function
    fun quz() { print 40; }
    quz();
    "#,
    "40\n"
)]
#[case(
    r#"
    // This function, when invoked should not return
    // or print anything
    fun f() {}
    f();
    "#,
    ""
)]
#[case(
    r#"
    // This program should print <fn foo>
    fun foo() {}
    print foo;
    "#,
    "<fn foo>\n"
)]
#[case(
    r#"
    // This program calculates the cumulative sum of
    // numbers from 1 to n.
    fun cumulative_sum() {
        var n = 10;  // Fixed value
        var total = 0;
        var i = 1;
        while (i <= n) {
            total = total + i;
            i = i + 1;
        }
        print "The cumulative sum from 1 to 10 is: ";
        print total;
    }

    cumulative_sum();
    "#,
    "The cumulative sum from 1 to 10 is: \n55\n"
)]
fn test_functions_without_arguments_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This is a simple function that takes one
    // argument and prints it
    fun f1(a) { print a; }
    f1(52);
    "#,
    "52\n"
)]
#[case(
    r#"
    // This function takes three arguments and prints
    // their sum
    fun f3(a, b, c) { print a + b + c; }
    f3(49, 49, 49);
    "#,
    "147\n"
)]
#[case(
    r#"
    // This function takes eight arguments and prints
    // their sum
    fun f8(a, b, c, d, e, f, g, h) {
      print a - b + c * d + e - f + g - h;
    }
    f8(58, 58, 58, 58, 58, 58, 58, 58);
    "#,
    "3364\n"
)]
#[case(
    r#"
    // This function takes two arguments and prints
    // the grade based on the score and bonus
    fun calculateGrade(score, bonus) {
      var finalScore = score + bonus;

      if (finalScore >= 90) {
        print "A";
      } else if (finalScore >= 80) {
        print "B";
      } else if (finalScore >= 70) {
        print "C";
      } else if (finalScore >= 60) {
        print "D";
      } else {
        print "F";
      }
    }

    var score = 86;
    var bonus = 5;
    print "Grade for given score is: ";
    calculateGrade(score, bonus);
    "#,
    "Grade for given score is: \nA\n"
)]
fn test_functions_with_arguments_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program computes the 35th Fibonacci number
    fun fib(n) {
      if (n < 2) return n;
      return fib(n - 2) + fib(n - 1);
    }

    var start = clock();
    print fib(10) == 55;
    print (clock() - start) < 5; // 5 seconds
    "#,
    "true\ntrue\n"
)]
#[case(
    r#"
    // This program uses a return statement inside an
    // if statement
    // to return "ok" if the condition is false
    fun f() {
      if (false) return "no"; else return "ok";
    }

    print f();
    "#,
    "ok\n"
)]
#[case(
    r#"
    // This program uses a return statement inside a
    // while loop
    // to return "ok" if the condition is false
    fun f() {
      while (!true) return "ok";
    }

    print f();
    "#,
    "nil\n"
)]
#[case(
    r#"
    // This program relies on the return statement
    // returning nil by default
    fun f() {
      return;
      print "bad";
    }

    print f();
    "#,
    "nil\n"
)]
fn test_return_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"fun foo() {
  // Return statements are allowed within function
  // scope
  return "at function scope is ok";
}

// Return statements are not allowed at the
// top-level
return; // expect compile error
"#,
    "[line 9] Error at 'return': Can't return from top-level code."
)]
#[case(
    r#"fun foo() {
  if (true) {
    return "early return";
  }

  for (var i = 0; i < 10; i = i + 1) {
    return "loop return";
  }
}

if (true) {
  return "conditional return";
  // expect compile error
}
"#,
    "[line 12] Error at 'return': Can't return from top-level code."
)]
#[case(
    r#"{
  // Return statements are not allowed in
  // top-level blocks
  return "not allowed in a block either";
  // expect compile error
}

fun allowed() {
  if (true) {
    return "this is fine";
  }
  return;
}
"#,
    "[line 4] Error at 'return': Can't return from top-level code."
)]
#[case(
    r#"fun outer() {
  fun inner() {
    return "ok";
  }

  return "also ok";
}

if (true) {
  fun nested() {
    return;
  }

  // Return statements are not allowed outside of
  // functions
  return "not ok"; // expect compile error
}
"#,
    "[line 16] Error at 'return': Can't return from top-level code."
)]
fn test_invalid_return_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
}

#[rstest]
#[case(
    r#"
    // This program creates a function that returns
    // another function
    // and uses it to greet two different people with
    // two different greetings
    fun makeGreeter() {
      fun greet(name) {
        print "Hello " + name;
      }
      return greet;
    }

    var sayHello = makeGreeter();

    sayHello("Bob");
    sayHello("Alice");
    sayHello("Eve");
    "#,
    "Hello Bob\nHello Alice\nHello Eve\n"
)]
#[case(
    r#"
    // This program defines a function that takes in a
    // function and an argument
    // and returns the result of calling the function
    // with the argument
    fun returnArg(arg) {
      return arg;
    }

    fun returnFunCallWithArg(func, arg) {
      return returnArg(func)(arg);
    }

    fun printArg(arg) {
      print arg;
    }

    returnFunCallWithArg(printArg, "baz");
    "#,
    "baz\n"
)]
#[case(
    r#"
    fun square(x) {
      return x * x;
    }

    // This higher-order function applies a
    // function N times to a starting value x.
    fun applyTimesN(N, f, x) {
      var i = 0;
      while (i < N) {
        x = f(x);
        i = i + 1;
      }
      return x;
    }

    // 3 is squared once
    print applyTimesN(1, square, 3);
    // 3 is squared twice
    print applyTimesN(2, square, 3);
    // 3 is squared thrice
    print applyTimesN(3, square, 3);
    "#,
    "9\n81\n6561\n"
)]
#[case(
    r#"
    // This program creates a function that returns
    // another function
    // and uses it to filter a list of numbers
    fun makeFilter() {
      fun filter(n) {
        if (n < 70) {
          return false;
        }
        return true;
      }
      return filter;
    }

    // This function applies a function to a list of
    // numbers
    fun applyToNumbers(f, count) {
      var n = 0;
      while (n < count) {
        if (f(n)) {
          print n;
        }
        n = n + 1;
      }
    }

    var greaterThanX = makeFilter();

    print "Numbers >= 70:";
    applyToNumbers(greaterThanX, 70 + 3);
    "#,
    "Numbers >= 70:\n70\n71\n72\n"
)]
fn test_higher_order_functions_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program demonstrates the use of closures
    // to create a counter function.
    // The inner function count() needs access to the
    // outer function's local variable i.
    // This can be achieved using closures.
    fun makeCounter() {
      var i = 0;
      fun count() {
        i = i + 6;
        print i;
      }

      return count;
    }

    var counter = makeCounter();
    counter();
    counter();
    "#,
    "6\n12\n"
)]
#[case(
    r#"
    // This program uses mutual recursion to determine
    // if a number is even or odd.
    // It also uses a shared threshold variable that
    // is used to determine if a number is too large
    //to be processed.
    {
      var threshold = 50;

      fun isEven(n) {
        if (n == 0) return true;
        if (n > threshold) return false;
        return isOdd(n - 1);
      }

      fun isOdd(n) {
        if (n == 0) return false;
        if (n > threshold) return false;
        return isEven(n - 1);
      }

      print isEven(75);
    }
    "#,
    "false\n"
)]
#[case(
    r#"
    // This program demonstrates the use of closures
    // to create a logger function.
    // The inner function log() has access to the
    // outer function's local variable logCount.
    // This is an example of how closures can be used
    // to create private variables and methods.
    fun makeLogger(prefix) {
      var logCount = 0;

      fun log(message) {
        logCount = logCount + 1;
        print prefix + ": " + message;

        if (logCount > 3) {
          print prefix + ": Too many log lines!";
          logCount = 0;
        }
      }

      return log;
    }

    var debugLog = makeLogger("foo");
    var errorLog = makeLogger("hello");

    debugLog("Starting");
    debugLog("Processing");
    debugLog("Finishing");
    debugLog("Extra line");

    errorLog("Failed!");
    errorLog("Retrying...");
    "#,
    "foo: Starting\nfoo: Processing\nfoo: Finishing\nfoo: Extra line\nfoo: Too many log lines!\nhello: Failed!\nhello: Retrying...\n"
)]
#[case(
    r#"
    // This program demonstrates the use of closures
    // to create an accumulator function.
    // The inner function accumulate() has access to
    // the outer function's local variables sum and
    // count.
    // This is an example of how closures can be used
    // to create private variables and methods.
    fun makeAccumulator(label) {
      var sum = 0;
      var count = 0;

      fun accumulate(value) {
        sum = sum + value;
        count = count + 1;

        print label;
        print count;
        print sum;
        print sum;

        if (count > 3) {
          print "reset";
          sum = 0;
          count = 0;
        }

        return sum;
      }

      return accumulate;
    }

    var acc1 = makeAccumulator("First:");
    var acc2 = makeAccumulator("Second:");

    acc1(4);
    acc1(5);
    acc1(6);
    acc1(2);

    acc2(5);
    acc2(2);
    "#,
    "First:\n1\n4\n4\nFirst:\n2\n9\n9\nFirst:\n3\n15\n15\nFirst:\n4\n17\n17\nreset\nSecond:\n1\n5\n5\nSecond:\n2\n7\n7\n"
)]
fn test_closures_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This variable is used in the function `f` below.
    var variable = "global";

    {
      fun f() {
        print variable;
      }

      f(); // this should print "global"

      // This variable declaration shouldn't affect
      // the usage in `f` above.
      var variable = "local";

      f(); // this should still print "global"
    }
    "#,
    "global\nglobal\n"
)]
#[case(
    r#"
    // This function is used in the function `f` below.
    fun global() {
      print "global";
    }

    {
      fun f() {
        global();
      }

      f(); // this should print "global"

      // This function declaration shouldn't affect
      // the usage in `f` above.
      fun global() {
        print "local";
      }

      f(); // this should also print "global"
    }
    "#,
    "global\nglobal\n"
)]
#[case(
    r#"
    var x = "global";

    fun outer() {
      var x = "outer";

      fun middle() {
        // The `inner` function should capture the
        // variable from the closest outer
        // scope, which is the `outer` function's
        // scope.
        fun inner() {
          print x; // Should capture "outer"
        }

        inner(); // Should print "outer"

        // This variable declaration shouldn't affect
        // the usage in `inner` above.
        var x = "middle";

        inner(); // Should still print "outer"
      }

      middle();
    }

    outer();
    "#,
    "outer\nouter\n"
)]
#[case(
    r#"
    var count = 0;

    {
      // The `counter` function should use the `count`
      // variable from the
      // global scope.
      fun makeCounter() {
        fun counter() {
          // This should increment the `count`
          // variable from the global scope.
          count = count + 1;
          print count;
        }
        return counter;
      }

      var counter1 = makeCounter();
      counter1(); // Should print 1
      counter1(); // Should print 2

      // This variable declaration shouldn't affect
      // our counter.
      var count = 0;

      counter1(); // Should print 3
    }
    "#,
    "1\n2\n3\n"
)]
fn test_identifier_resolution_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program tries to execute an integer as a
    // function
    24();
    "#,
    "Can only call functions and classes.",
    "[line 4]"
)]
#[case(
    r#"
    // This program tries to call a function with too
    // many arguments
    fun f(a, b) {
      print a;
      print b;
    }

    f(1, 2, 3, 4);
    // expect runtime error: Expected 2 arguments
    "#,
    "Expected 2 arguments but got 4.",
    "[line 9]"
)]
#[case(
    r#"
    // This program tries to call a function with too
    // few arguments
    fun f(a, b) {}

    // expect runtime error: Expected 2 arguments
    f(1);
    "#,
    "Expected 2 arguments but got 1.",
    "[line 7]"
)]
#[case(
    r#"
    // This program tries to execute a boolean as a
    // function
    (true == true)();
    "#,
    "Can only call functions and classes.",
    "[line 4]"
)]
fn test_function_runtime_errors_report_output_and_exit_70(
    #[case] source: &str,
    #[case] expected_message: &str,
    #[case] expected_line: &str,
) {
    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains(expected_message));
    assert!(combined.contains(expected_line));
}

#[rstest]
#[case(
    r#"
    // This program demonstrates global and local
    // variable shadowing in Lox.
    var a = 62;

    fun printAndModify() {
      print a;
      var a = 70;
      print a;
    }

    print a;
    a = 75;
    printAndModify();
    "#,
    "62\n75\n70\n"
)]
#[case(
    r#"
    // This program uses a while loop to count down
    // from 5 to 1, printing each
    // number
    // and then decrementing the count until it
    // reaches 0, at which point it prints
    // "Blast off!"
    var count = 5;

    fun tick() {
      if (count > 0) {
        print count;
        count = count - 1;
        return false;
      }
      print "Blast off!";
      return true;
    }

    while (!tick()) {}
    "#,
    "5\n4\n3\n2\n1\nBlast off!\n"
)]
#[case(
    r#"
    // This program demonstrates variable shadowing in
    // Lox with functions.
    // The first counter is a global variable that is
    // modified by the inner block.
    // The second counter is a local variable that
    // shadows the global variable.
    var counter = 73;

    fun incrementCounter(amount) {
      counter = counter + amount;
      print counter;
    }

    {
      counter = 45;
      incrementCounter(2);
      print counter;
    }
    print counter;
    "#,
    "47\n47\n47\n"
)]
#[case(
    r#"
    // This program tests variable scoping and
    // shadowing in Lox. It demonstrates:
    // Global variable declarations
    // Function scope access to global variables
    // Block scoping with local variables shadowing
    // outer variables
    // Verification that global variables remain
    // unchanged after shadowing
    var x = 1;
    var y = 2;

    fun printBoth() {
      if (x < y) {
        print "x is less than y:";
        print x;
        print y;
      } else {
        print "x is not less than y:";
        print x;
        print y;
      }
    }

    {
      var x = 10;
      {
        var y = 20;

        var i = 0;
        while (i < 3) {
          x = x + 1;
          y = y - 1;
          print "Local x: ";
          print x;
          print "Local y: ";
          print y;
          i = i + 1;
        }

        if (x > y) {
          print "Local x > y";
        }

        printBoth();
      }
    }

    if (x == 1 and y == 2) {
      print "Globals unchanged:";
      printBoth();
    }
    "#,
    "Local x: \n11\nLocal y: \n19\nLocal x: \n12\nLocal y: \n18\nLocal x: \n13\nLocal y: \n17\nx is less than y:\n1\n2\nGlobals unchanged:\nx is less than y:\n1\n2\n"
)]
fn test_function_scope_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program is missing the closing parenthesis
    // for the function call
    // Hence the compiler error
    print clock(;
    "#,
    "[line 5] Error at ';': Expect expression"
)]
#[case(
    r#"
    // This program is missing the opening parenthesis
    // for the function call,
    // and has extra closing parentheses
    // Hence the compiler error
    print clock)));
    "#,
    "[line 6] Error at ')': Expect ';' after value."
)]
#[case(
    r#"
    // This function declaration is missing the
    // opening and closing braces
    // The body should always be inside a block
    // Hence the compiler error
    fun f() 79;
    print f();
    "#,
    "[line 6] Error at '79': Expect '{' before function body."
)]
#[case(
    r#"
    // This function declaration is missing a comma
    // between b and c
    // Hence the compiler error
    fun foo(a, b c, d, e, f) {}
    foo();
    "#,
    "[line 5] Error at 'c': Expect ')' after parameters."
)]
fn test_function_syntactic_errors_report_output_and_exit_65(
    #[case] source: &str,
    #[case] expected_output_fragment: &str,
) {
    let output = run_source(source);

    assert_eq!(Some(65), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains(expected_output_fragment));
}

#[rstest]
#[case(
    r#"// Class declaration with empty body
class Spaceship {}
print Spaceship;
"#,
    "Spaceship\n"
)]
#[case(
    r#"// Multiple class declarations with empty body
class Robot {}
class Wizard {}
print Robot;
print Wizard;
print "Both classes successfully printed";
"#,
    "Robot\nWizard\nBoth classes successfully printed\n"
)]
#[case(
    r#"// Class declaration inside function should work
fun foo() {
  class Superhero {}
  print "Class declared inside function";
  print Superhero;
}

foo();
print "Function called successfully";
"#,
    "Class declared inside function\nSuperhero\nFunction called successfully\n"
)]
fn test_class_declarations_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[test]
fn test_block_scoped_class_runtime_error_reports_stderr_and_exit_70() {
    let source = r#"{
  // Class declaration inside blocks should work
  class Dinosaur {}
  print "Inside block: Dinosaur exists";
  print Dinosaur;
}
print "Accessing out-of-scope class:";
print Dinosaur;  // expect runtime error
"#;

    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert_eq!(
        "Inside block: Dinosaur exists\nDinosaur\nAccessing out-of-scope class:\n",
        stdout
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Undefined variable 'Dinosaur'."));
    assert!(stderr.contains("[line 8]"));
}

#[rstest]
#[case(
    r#"// Class instantiation
class Spaceship {}
var falcon = Spaceship();
print falcon;
"#,
    "Spaceship instance\n"
)]
#[case(
    r#"// Instantiating multiple instances of a class
// should work
class Robot {}
var r1 = Robot();
var r2 = Robot();

print "Created multiple robots:";
print r1;
print r2;
"#,
    "Created multiple robots:\nRobot instance\nRobot instance\n"
)]
#[case(
    r#"class Wizard {}
class Dragon {}

// Instantiating classes in a function should work
fun createCharacters() {
  var merlin = Wizard();
  var smaug = Dragon();
  print "Characters created in fantasy world:";
  print merlin;
  print smaug;
  return merlin;
}

var mainCharacter = createCharacters();
// An instance of a class should be truthy
if (mainCharacter) {
  print "The main character is:";
  print mainCharacter;
} else {
  print "Failed to create a main character.";
}
"#,
    "Characters created in fantasy world:\nWizard instance\nDragon instance\nThe main character is:\nWizard instance\n"
)]
#[case(
    r#"class Superhero {}

var count = 0;
while (count < 3) {
  var hero = Superhero();
  print "Hero created:";
  print hero;
  count = count + 1;
}

print "All heroes created!";
"#,
    "Hero created:\nSuperhero instance\nHero created:\nSuperhero instance\nHero created:\nSuperhero instance\nAll heroes created!\n"
)]
fn test_class_instances_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class Doughnut {}

// BostonCream is a subclass of Doughnut
class BostonCream < Doughnut {}

print Doughnut();
print BostonCream();
"#,
    "Doughnut instance\nBostonCream instance\n"
)]
#[case(
    r#"{
  class A {}

  // B is a subclass of A
  class B < A {}

  // C is also a subclass of A
  class C < A {}

  print A();
  print B();
  print C();
}
"#,
    "A instance\nB instance\nC instance\n"
)]
#[case(
    r#"class A {}

fun f() {
  // B is a subclass of A
  class B < A {}
  return B;
}

print f();
"#,
    "B\n"
)]
#[case(
    r#"class Vehicle {}

// Car is a subclass of Vehicle
class Car < Vehicle {}

// Sedan is a subclass of Car
class Sedan < Car {}

print Vehicle();
print Car();
print Sedan();

{
  // Truck is a subclass of Vehicle
  class Truck < Vehicle {}
  print Truck();
}
"#,
    "Vehicle instance\nCar instance\nSedan instance\nTruck instance\n"
)]
fn test_class_hierarchy_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class Doughnut {
  cook() {
    print "Fry until golden brown.";
    }
  }

// BostonCream is a subclass of Doughnut
class BostonCream < Doughnut {}

// BostonCream class should inherit the cook
// method from Doughnut class
BostonCream().cook();
"#,
    "Fry until golden brown.\n"
)]
#[case(
    r#"class Root {
  getName() {
    print "Root class";
  }
}

class Parent < Root {
  parentMethod() {
    print "Method defined in Parent";
  }
}

class Child < Parent {
  childMethod() {
    print "Method defined in Child";
  }
}

var root = Root();
var parent = Parent();
var child = Child();

// Root methods are available to all
root.getName();
parent.getName();
child.getName();

// Parent methods are available to Parent and Child
parent.parentMethod();
child.parentMethod();

// Child methods are only available to Child
child.childMethod();
"#,
    "Root class\nRoot class\nRoot class\nMethod defined in Parent\nMethod defined in Parent\nMethod defined in Child\n"
)]
#[case(
    r#"class Foo {
  init() {
    this.secret = 42;
  }
}

// Bar is a subclass of Foo
class Bar < Foo {}

// Baz is a subclass of Bar
class Baz < Bar {}

var baz = Baz();

// Baz should inherit the constructor from Foo
// which should set the secret value to 42
print baz.secret;
"#,
    "42\n"
)]
#[case(
    r#"class hello {
  inhello() {
    print "from hello";
  }
}

class bar < hello {
  inbar() {
    print "from bar";
  }
}

class world < bar {
  inworld() {
    print "from world";
  }
}

// world should inherit the methods
// from both hello and bar
var world = world();
world.inhello();
world.inbar();
world.inworld();
"#,
    "from hello\nfrom bar\nfrom world\n"
)]
fn test_inherited_methods_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class A {
  method() {
    print "A method";
  }
}


// B inherits method `method` from A
// and overrides it with a new implementation
class B < A {
  method() {
    print "B method";
  }
}

var b = B();
b.method();  // expect: B method
"#,
    "B method\n"
)]
#[case(
    r#"class Base {
  init(a) {
    this.a = a;
  }
}


// Constructors can also be overridden
class Derived < Base {
  init(a, b) {
    this.a = a;
    this.b = b;
  }
}

var derived = Derived(20, 78);
print derived.a;
print derived.b;
"#,
    "20\n78\n"
)]
#[case(
    r#"class Base {
  init(a) {
    this.a = a;
  }

  cook() {
    return "Base cooking " + this.a;
  }
}

class Derived < Base {
  init(a, b) {
    this.a = a;
    this.b = b;
  }

  // Derived overrides the cook method of Base
  cook() {
    return "Derived cooking " + this.b + " with "
    + this.a + " and " + this.b;
  }

  makeFood() {
    return this.cook();
  }
}

var derived = Derived("onions", "shallots");
print derived.a;
print derived.b;

print Base("ingredient").cook();
print derived.cook();
"#,
    "onions\nshallots\nBase cooking ingredient\nDerived cooking shallots with onions and shallots\n"
)]
#[case(
    r#"class Animal {
  speak() {
    return "Animal speaks";
  }

  makeSound() {
    return "Generic sound";
  }

  communicate() {
    return this.speak() + " : " + this.makeSound();
  }
}

// Dog inherits the speak and makeSound methods
// from Animal and overrides them with new
// implementations specific to dogs
class Dog < Animal {
  speak() {
    return "Dog speaks";
  }

  makeSound() {
    return "Woof";
  }
}

// Puppy inherits the speak and makeSound methods
// from Dog and overrides them with new
// implementations specific to puppies
class Puppy < Dog {
  speak() {
    return "Puppy speaks";
  }
}

var animal = Animal();
var dog = Dog();
var puppy = Puppy();

print animal.communicate();
print dog.communicate();
print puppy.communicate();
"#,
    "Animal speaks : Generic sound\nDog speaks : Woof\nPuppy speaks : Woof\n"
)]
fn test_overridden_methods_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[test]
fn test_class_cannot_inherit_from_itself_reports_static_error_and_exit_65() {
    let source = r#"// A class can't inherit from itself.
class Foo < Foo {} // expect compile error
"#;

    assert_static_error(
        source,
        "[line 2] Error at 'Foo': A class can't inherit from itself.",
    );
}

#[rstest]
#[case(
    r#"fun A() {}

// A class can only inherit from a class.
class B < A {} // expect runtime error

print A();
print B();
"#,
    "[line 4]"
)]
#[case(
    r#"var A = "class";

// A class can only inherit from a class
class B < A {} // expect runtime error

print B();
"#,
    "[line 4]"
)]
#[case(
    r#"class A {
  method() {
    print "A";
  }
}

class B < A {}
class C < B {}
class D < A {}

// B is updated to a non-class value
B = "not a class";
// E inherits from B, which is not a class
class E < B {}  // expect runtime error
"#,
    "[line 14]"
)]
fn test_inheritance_runtime_errors_report_stderr_and_exit_70(
    #[case] source: &str,
    #[case] expected_line: &str,
) {
    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Superclass must be a class."));
    assert!(stderr.contains(expected_line));
}

#[rstest]
#[case(
    r#"class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

// Super can be used to call the overridden method
// of the parent class
class BostonCream < Doughnut {
  cook() {
    super.cook();
  }
}

BostonCream().cook();
"#,
    "Fry until golden brown.\n"
)]
#[case(
    r#"class A {
  say() {
    print "A";
  }
}

class B < A {
  // test calls say() from A
  test() {
    super.say();
  }

  say() {
    print "B";
  }
}

// C inherits test() from B
// But the super keyword used in test()
// should still have a binding to B
class C < B {
  say() {
    print "C";
  }
}

C().say();
C().test(); // expect: A
"#,
    "C\nA\n"
)]
#[case(
    r#"class A {
  say() {
    print "A";
  }
}

class B < A {
  getClosure() {
    fun closure() {
      super.say();
    }
    return closure;
  }

  say() {
    print "B";
  }
}

class C < B {
  say() {
    print "C";
  }
}

// C inherits getClosure() from B
// But the super keyword used in getClosure()
// should still have a binding to B
C().getClosure()(); // expect: A
"#,
    "A\n"
)]
#[case(
    r#"class Base {
  method() {
    print "Base.method()";
  }
}

// Parent inherits method from Base
class Parent < Base {
  method() {
    super.method();
  }
}

// Child inherits method from Parent
class Child < Parent {
  method() {
    super.method();
  }
}

var parent = Parent();
parent.method(); // expect: Base.method()
var child = Child();
child.method(); // expect: Base.method()
"#,
    "Base.method()\nBase.method()\n"
)]
fn test_super_keyword_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class Foo {
  cook() {
    // Foo is not a subclass
    super.cook(); // expect compile error
  }
}
"#,
    "[line 4] Error at 'super': Can't use 'super' in a class with no superclass."
)]
#[case(
    r#"// super can't be used outside of a class
super.notEvenInAClass(); // expect compile error
"#,
    "[line 2] Error at 'super': Can't use 'super' outside of a class."
)]
#[case(
    r#"class A {
  method() {}
}

class B < A {
  method() {
    // super must be followed by `.`
    // and an expression
    (super).method(); // expect compile error
  }
}
"#,
    "[line 9] Error at ')': Expect '.' after 'super'."
)]
#[case(
    r#"class A {}

class B < A {
  method() {
    // super must be followed by `.`
    // and an expression
    super; // expect compile error
  }
}
"#,
    "[line 7] Error at ';': Expect '.' after 'super'."
)]
fn test_invalid_super_static_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
}

#[rstest]
#[case(
    r#"class Spaceship {}
var falcon = Spaceship();

// Setting properties on an instance should work
falcon.name = "Millennium Falcon";
falcon.speed = 75.5;

// Getting properties on an instance should work
print "Ship details:";
print falcon.name;
print falcon.speed;
"#,
    "Ship details:\nMillennium Falcon\n75.5\n"
)]
#[case(
    r#"class Robot {}
var r2d2 = Robot();

// Setting properties on an instance should work
r2d2.model = "Astromech";
r2d2.operational = true;

// Getting properties on an instance should work
if (r2d2.operational) {
  print r2d2.model;
  r2d2.mission = "Navigate hyperspace";
  print r2d2.mission;
}
"#,
    "Astromech\nNavigate hyperspace\n"
)]
#[case(
    r#"class Superhero {}
var batman = Superhero();
var superman = Superhero();

// Setting properties on an instance should work
batman.name = "Batman";
batman.called = 59;

// Setting properties on an instance should work
superman.name = "Superman";
superman.called = 75;

// Getting properties on an instance should work
print "Times " + superman.name + " was called: ";
print superman.called;
print "Times " + batman.name + " was called: ";
print batman.called;
"#,
    "Times Superman was called: \n75\nTimes Batman was called: \n59\n"
)]
#[case(
    r#"class Wizard {}
var gandalf = Wizard();

gandalf.color = "Grey";
gandalf.power = nil;
print gandalf.color;

// functions should be able to accept class
// instances and get or set properties on them
fun promote(wizard) {
  wizard.color = "White";
  if (true) {
    wizard.power = 100;
  } else {
    wizard.power = 0;
  }
}

promote(gandalf);
print gandalf.color;
print gandalf.power;
"#,
    "Grey\nWhite\n100\n"
)]
fn test_class_getters_and_setters_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class Robot {
  beep() {
    print "Beep boop!";
  }
}

var r2d2 = Robot();
// Calling a method on an instance should work
r2d2.beep();

// Calling a method on a class instance should work
Robot().beep();
"#,
    "Beep boop!\nBeep boop!\n"
)]
#[case(
    r#"{
  class Foo {
    returnSelf() {
      // Should be able to return the class itself
      return Foo;
    }
  }

  // Calling a method on an instance should work
  print Foo().returnSelf();
}
"#,
    "Foo\n"
)]
#[case(
    r#"class Wizard {
  castSpell(spell) {
    // Methods should be able to accept a parameter
    print "Casting a magical spell: " + spell;
  }
}

class Dragon {
  // Methods should be able to accept multiple
  // parameters
  breatheFire(fire, intensity) {
    print "Breathing " + fire + " with intensity: "
    + intensity;
  }
}

var merlin = Wizard();
var smaug = Dragon();

if (false) {
  var action = merlin.castSpell;
  action("Fireball");
} else {
  var action = smaug.breatheFire;
  action("Fire", "100");
}
"#,
    "Breathing Fire with intensity: 100\n"
)]
#[case(
    r#"class Superhero {
  // Methods should be able to accept a parameter
  useSpecialPower(hero) {
    print "Using power: " + hero.specialPower;
  }

  // Methods should be able to accept a parameter
  // of any type
  hasSpecialPower(hero) {
    return hero.specialPower;
  }

  // Methods should be able to accept class
  // instances as parameters and then update their
  // properties
  giveSpecialPower(hero, power) {
    hero.specialPower = power;
  }
}

fun performHeroics(hero, superheroClass) {
  if (superheroClass.hasSpecialPower(hero)) {
    superheroClass.useSpecialPower(hero);
  } else {
    print "No special power available";
  }
}

var superman = Superhero();
var heroClass = Superhero();

if (false) {
  heroClass.giveSpecialPower(superman, "Flight");
} else {
  heroClass.giveSpecialPower(superman, "Strength");
}

performHeroics(superman, heroClass);
"#,
    "Using power: Strength\n"
)]
fn test_instance_methods_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"class Spaceship {
  identify() {
    // this should be bound to the instance
    print this;
  }
}

// Calling a method on a class instance should work
Spaceship().identify();
"#,
    "Spaceship instance\n"
)]
#[case(
    r#"class Calculator {
  add(a, b) {
    // this should be bound to the instance
    return a + b + this.memory;
  }
}

var calc = Calculator();
// Instance properties should be accessible using
// the this keyword
calc.memory = 28;
print calc.add(35, 1);
"#,
    "64\n"
)]
#[case(
    r#"class Animal {
  makeSound() {
    print this.sound;
  }

  identify() {
    print this.species;
  }
}

var dog = Animal();
dog.sound = "Woof";
dog.species = "Dog";

var cat = Animal();
cat.sound = "Meow";
cat.species = "Cat";

// The this keyword should be bound to the
// class instance that the method is called on
cat.makeSound = dog.makeSound;
dog.identify = cat.identify;

cat.makeSound(); // expect: Woof
dog.identify(); // expect: Cat
"#,
    "Woof\nCat\n"
)]
#[case(
    r#"class Wizard {
  getSpellCaster() {
    fun castSpell() {
      print this;
      print "Casting spell as " + this.name;
    }

    // Functions are first-class objects in Lox
    return castSpell;
  }
}

var wizard = Wizard();
wizard.name = "Merlin";

// Calling an instance method that returns a
// function should work
wizard.getSpellCaster()();
"#,
    "Wizard instance\nCasting spell as Merlin\n"
)]
fn test_this_keyword_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"// The this keyword used outside of a class
// should be a compile error
print this;
"#,
    "[line 3] Error at 'this': Can't use 'this' outside of a class."
)]
#[case(
    r#"// using this outside of a class shouldn't work
fun notAMethod() {
  print this; // expect compile error
}
"#,
    "[line 3] Error at 'this': Can't use 'this' outside of a class."
)]
fn test_invalid_this_static_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
}

#[rstest]
#[case(
    r#"class Person {
  sayName() {
    // this is not a callable object
    print this(); // expect runtime error
  }
}
Person().sayName();
"#,
    "Can only call functions and classes.",
    "[line 4]"
)]
#[case(
    r#"class Confused {
  method() {
    fun inner(instance) {
      // this is a local variable
      var feeling = "confused";
      // Unless explicitly set, feeling can't be
      // accessed using this keyword
      print this.feeling; // expect runtime error
    }
    return inner;
  }
}

var instance = Confused();
var m = instance.method();
// calling the function returned should work
m(instance);
"#,
    "Undefined property 'feeling'.",
    "[line 8]"
)]
fn test_invalid_this_runtime_errors_report_stderr_and_exit_70(
    #[case] source: &str,
    #[case] expected_message: &str,
    #[case] expected_line: &str,
) {
    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains(expected_message));
    assert!(stderr.contains(expected_line));
}

#[rstest]
#[case(
    r#"class Default {
  // this is the constructor
  init() {
    // it should be able to set
    // properties on the instance
    this.x = "quz";
    this.y = 49;
  }
}

// the constructor should be called
// automatically  when the class is being
// instantiated
print Default().x;
print Default().y;
"#,
    "quz\n49\n"
)]
#[case(
    r#"class Robot {
  // constructors should be able to accept
  // one or more parameters
  init(model, function) {
    this.model = model;
    this.function = function;
  }
}
print Robot("R2-D2", "Astromech").model;
"#,
    "R2-D2\n"
)]
#[case(
    r#"class Counter {
  init(startValue) {
    if (startValue < 0) {
      print "startValue can't be negative";
      this.count = 0;
    } else {
      this.count = startValue;
    }
  }
}

// constructor is called automatically here
var instance = Counter(-67);
print instance.count;

// it should be possible to call the constructor
// on a class instance as well
print instance.init(67).count;
"#,
    "startValue can't be negative\n0\n67\n"
)]
#[case(
    r#"class Vehicle {
  init(type) {
    this.type = type;
  }
}

class Car {
  init(make, model) {
    this.make = make;
    this.model = model;
    this.wheels = "four";
  }

  describe() {
    // expression across multiple lines should work
    print this.make + " " + this.model +
    " with " + this.wheels + " wheels";
  }
}

var vehicle = Vehicle("Generic");
print "Generic " + vehicle.type;

var myCar = Car("Toyota", "Corolla");
myCar.describe();
"#,
    "Generic Generic\nToyota Corolla with four wheels\n"
)]
fn test_constructor_calls_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[test]
fn test_empty_return_from_constructor_success() {
    let source = r#"class Person {
  init() {
    print "bar";
    // constructor should return nothing
    return;
  }
}

Person();
"#;

    assert_success_output(source, "bar\n");
}

#[rstest]
#[case(
    r#"class ThingDefault {
  init() {
    this.x = "foo";
    this.y = 42;
    // constructor should not return the instance
    return this; // expect compile error
  }
}
var out = ThingDefault();
print out;
"#,
    "[line 6] Error at 'return': Can't return a value from an initializer."
)]
#[case(
    r#"class Foo {
  init() {
    // constructor should not return anything
    return "something"; // expect compile error
  }
}

Foo();
"#,
    "[line 4] Error at 'return': Can't return a value from an initializer."
)]
#[case(
    r#"class Foo {
  init() {
    // just calling the callback should've worked
    // but returning it is not allowed
    return this.callback(); // expect compile error
  }

  callback() {
    return "callback";
  }
}

Foo();
"#,
    "[line 5] Error at 'return': Can't return a value from an initializer."
)]
fn test_constructor_return_value_errors_report_stderr_and_exit_65(
    #[case] source: &str,
    #[case] expected_stderr_fragment: &str,
) {
    assert_static_error(source, expected_stderr_fragment);
}

#[rstest]
#[case(
    r#"
    // This program uses a for loop to print the
    // numbers from 0 to 3
    // The assignment operation returns the assigned
    // value
    for (var foo = 0; foo < 3;) print foo = foo + 1;
    "#,
    "1\n2\n3\n"
)]
#[case(
    r#"
    // This program uses a for loop to print the
    // numbers from 0 to 3
    for (var foo = 0; foo < 3; foo = foo + 1) {
      print foo;
    }
    "#,
    "0\n1\n2\n"
)]
#[case(
    r#"
    // This program uses a for loop to print the
    // numbers from 0 to 2
    // The loop initializer is ignored in this loop
    var bar = 0;
    for (; bar < 2; bar = bar + 1) print bar;

    // This program uses a for loop to print the
    // numbers from 0 to 2
    // The loop increment clause is ignored in this
    // loop
    for (var foo = 0; foo < 2;) {
      print foo;
      foo = foo + 1;
    }
    "#,
    "0\n1\n0\n1\n"
)]
#[case(
    r#"
    // This program uses for loops and block scopes
    // to print the updates to the same variable
    var baz = "after";
    {
      var baz = "before";

      for (var baz = 0; baz < 1; baz = baz + 1) {
        print baz;
        var baz = -1;
        print baz;
      }
    }

    {
      for (var baz = 0; baz > 0; baz = baz + 1) {}

      var baz = "after";
      print baz;

      for (baz = 0; baz < 1; baz = baz + 1) {
        print baz;
      }
    }
    "#,
    "0\n-1\nafter\n0\n"
)]
fn test_for_statements_success(#[case] source: &str, #[case] expected_stdout: &str) {
    assert_success_output(source, expected_stdout);
}

#[rstest]
#[case(
    r#"
    // This program would give a compile error
    // because the variable declaration is not
    // inside a block
    for (;;) var foo;
    "#,
    "[line 5] Error at 'var': Expect expression"
)]
#[case(
    r#"
    // This program would give a compile error
    // because the condition is not valid
    for (var a = 1; {}; a = a + 1) {}
    "#,
    "[line 4] Error at '{': Expect expression"
)]
#[case(
    r#"
    // This program would give a compile error
    // because the increment clause is not valid
    for (var a = 1; a < 2; {}) {}
    "#,
    "[line 4] Error at '{': Expect expression"
)]
#[case(
    r#"
    // This program would give a compile error
    // because the initialization clause is not valid
    for ({}; a < 2; a = a + 1) {}
    "#,
    "[line 4] Error at '{': Expect expression"
)]
fn test_for_syntactic_errors_report_output_and_exit_65(
    #[case] source: &str,
    #[case] expected_output_fragment: &str,
) {
    let output = run_source(source);

    assert_eq!(Some(65), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains(expected_output_fragment));
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

#[test]
fn test_scope_runtime_error_reports_stderr_and_exit_70() {
    let source = r#"
    // Variables declared in an outer scope should be
    // accessible inside inner scopes, but not the
    // other way around
    {
      var world = "outer world";
      var quz = "outer quz";
      {
        world = "modified world";
        var quz = "inner quz";
        print world;
        print quz;
      }
      print world;
      print quz;
    }
    print world;
    "#;

    let output = run_source(source);

    assert_eq!(Some(70), output.status.code());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert_eq!(
        "modified world\ninner quz\nmodified world\nouter quz\n",
        stdout
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("Undefined variable 'world'."));
    assert!(stderr.contains("[line 17]"));
}
