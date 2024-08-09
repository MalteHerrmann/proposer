format:
	cargo fmt

lint:
	cargo clippy

test:
	cargo test

# on CI the keyring and OpenAI key are not set up so we're skipping those tests
test-remote:
	cargo test --features remote

update:
	cargo update

build:
	cargo build --locked

install:
	cargo install --path . --locked

.PHONY: format lint test test-remote update build install