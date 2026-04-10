# Contributing to Sword

Hey! Want to help out with Sword? Awesome! Here’s how you can jump in:

## How to contribute

- Fork this repo and make a new branch for your changes.
- When you’re ready, open a Pull Request to the `main` branch. Tell what you changed and why.
- Found a bug or have an idea? Open an Issue and let us know :)

## Code style & tests

- Use `rustfmt` to keep code style.
- Run the full local quality gate before opening a PR:
  - `cargo fmt --all -- --check`
  - `cargo clippy -p sword --all-targets -- -D warnings`
  - `cargo clippy -p sword-web --all-targets --all-features -- -D warnings`
  - `cargo clippy -p sword-socketio --all-targets --all-features -- -D warnings`
  - `cargo clippy -p sword-grpc --all-targets --all-features -- -D warnings`
  - `cargo test -p sword-web --all-features`
  - `cargo test -p sword-socketio --all-features`
  - `cargo test -p sword-grpc --all-features`
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p sword --no-deps`
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p sword-web --no-deps --all-features`
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p sword-socketio --no-deps --all-features`
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p sword-grpc --no-deps --all-features`
- Try to write clear code and add comments if something’s tricky.

## Docs

- If you add something new, add rustdocs or README.

## Setup

1. Install Rust: https://rustup.rs
2. Clone: `git clone https://github.com/sword-web/sword`
3. Go to the folder: `cd sword`
4. Build: `cargo build`
5. Test: `cargo test`

Thanks for helping make Sword better!
