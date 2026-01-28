[private]
default:
    @just --list --unsorted

# run a lox program
run path="test.lox":
    @./your_program.sh tokenize {{ path }}

alias r := run

# test locally with cargo
test-local:
    @cargo test

alias tl := test-local

# test remotely with codecrafters
test-remote:
    @codecrafters test

alias tr := test-remote

# submit to codecrafters
submit:
    @codecrafters submit

alias s := submit
