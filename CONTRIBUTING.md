# Contributing

## AI usage declaration

This project records AI assistance in [aidecl.yaml](./aidecl.yaml). 
Contributors are responsible for reviewing and verifying AI-assisted contributions.

## Development

Run these checks from the crate directory:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps
```
