[private]
default:
    @just --list --unsorted

# run a lox program
run path="test.lox":
    @./your_program.sh parse {{ path }}

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
submit message:
    @jj describe --message "{{ message }}"
    @jj bookmark move master
    @jj git push --bookmark master --remote me
    @jj git push --bookmark master --remote origin

alias s := submit
