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
