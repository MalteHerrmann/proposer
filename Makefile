format:
	cargo fmt

lint:
	cargo clippy

test:
	cargo test

build:
	cargo build --locked

install:
	cargo install --path . --locked
