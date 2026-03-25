# xilem_extras build automation

# Build the library
build:
    cargo build

# Run the widget gallery example
run:
    cargo run --example gallery

# Generate documentation
doc:
    cargo doc --no-deps --open

# Run all tests
test:
    cargo test

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy -- -W clippy::all
