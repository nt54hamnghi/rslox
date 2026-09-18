[private]
default:
    @just --list --unsorted

# format code
fmt:
    cargo +nightly fmt
alias f := fmt

# lint code with clippy and rustfmt
lint:
    cargo clippy --all-targets -- -D clippy::all -W clippy::pedantic
    cargo +nightly fmt --check

alias l := lint

# run a lox program
# run path="test.lox":
#     @./your_program.sh run {{ path }}
# alias r := run
