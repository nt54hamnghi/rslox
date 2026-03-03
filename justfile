[private]
default:
    @just --list --unsorted

# run a lox program
run path="test.lox":
    @./your_program.sh run {{ path }}

alias r := run

# format code
fmt:
    cargo +nightly fmt

alias f := fmt

# test locally with cargo
test-local:
    @cargo test

alias tl := test-local

# test remotely with codecrafters
test-remote *args:
    @codecrafters test {{ args }}

alias tr := test-remote

# submit to codecrafters
submit message:
    @cargo +nightly fmt
    @jj describe --message "{{ message }}"
    @jj bookmark move master
    @jj git push --bookmark master --remote me
    @jj git push --bookmark master --remote origin

alias s := submit
