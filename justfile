git *ARGS:
    cargo run -- {{ ARGS }}

help *ARGS:
    @just git help {{ ARGS }}

test:
    cargo test

lint:
    cargo clippy -- -Dwarnings
