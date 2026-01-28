[private]
default:
    @just --list --unsorted

# run a lox program
run path="test.lox":
    @./your_program.sh tokenize {{ path }}

alias r := run

# test with codecrafters
test:
    @codecrafters test

alias t := test

# submit to codecrafters
submit:
    @codecrafters submit

alias s := submit
