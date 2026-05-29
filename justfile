test:
    cargo test

lint:
    cargo check
    cargo clippy

fmt:
    cargo fmt --check

watch *args:
    cargo watch -x test

doc:
    cargo doc --open
