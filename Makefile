.PHONY: clean fmt lint machete nursery test

clean:
	cargo clean --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo clippy -p sword --all-targets -- -D warnings
	cargo clippy -p sword-web --all-targets --all-features -- -D warnings
	cargo clippy -p sword-socketio --all-targets --all-features -- -D warnings
	cargo clippy -p sword-grpc --all-targets --all-features -- -D warnings

	cargo doc -p sword-core --no-deps
	cargo doc -p sword --no-deps
	cargo doc -p sword-web --no-deps --all-features
	cargo doc -p sword-socketio --no-deps --all-features
	cargo doc -p sword-grpc --no-deps --all-features

machete:
	cargo machete

nursery:
	cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery

test:
	cargo test -p sword-web-tests
	cargo test -p sword-socketio-tests
	cargo test -p sword-grpc-tests -- --test-threads=1

test-log:
	cargo test -p sword-web-tests -- --nocapture
	cargo test -p sword-socketio-tests -- --nocapture
	cargo test -p sword-grpc-tests -- --test-threads=1 --nocapture

build:
	cargo build --workspace --all-features

release:
	cargo build --workspace --all-features --release
