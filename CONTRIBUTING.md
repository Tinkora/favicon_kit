# Contributing to Favicon Kit

Favicon Kit is currently v0.1.0. Contributions should keep the browser-local
asset-generation contract and the static editor deployable as documented.

## Development Environment

- Rust 1.95 or later (stable)
- wasm-pack 0.15 or later
- wasm32-unknown-unknown target (`rustup target add wasm32-unknown-unknown`)

## Project Structure

```text
favicon_kit/
├── crates/
│   ├── favicon_kit_core/         # Image resize, ICO encode, manifest generation, WASM bindings
│   └── favicon_kit_web/          # WASM cdylib + HTML editor
├── docs/                          # English-first product specifications
└── skills/                        # Non-runnable integration drafts and guidance
```

## Local Development

```bash
# Run tests
cargo test --workspace

# Format & lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Build Web WASM
wasm-pack build --target web crates/favicon_kit_web

# Copy the generated package beside the static editor
mkdir -p crates/favicon_kit_web/static/pkg
cp crates/favicon_kit_web/pkg/* crates/favicon_kit_web/static/pkg/

# Start the local editor
python3 -m http.server --directory crates/favicon_kit_web/static 8080
```

For Pages deployment, publish `crates/favicon_kit_web/static/` only after its
`pkg/` directory is populated. Do not add a second root-level marketing page.

## Documentation and Comments

- English is the default language for public documentation. Keep
  `README.zh-CN.md` complete when user-facing README content changes.
- Write new or modified code comments in English.
- Keep the documented ZIP member list synchronized across `README.md`,
  `AGENTS.md`, and `docs/product_spec.md`.

## Commit Convention

- Use English Conventional Commits, such as `feat:`, `fix:`, `docs:`,
  `refactor:`, `test:`, and `chore:`.
- Each commit must contain one logically complete change.

## Pull Request Process

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/your-feature`)
3. Commit your changes
4. Ensure `cargo fmt --all -- --check`, `cargo test --workspace --locked`, and
   `cargo clippy --workspace --all-targets --locked -- -D warnings` pass
5. Push to your fork (`git push origin feat/your-feature`)
6. Create a Pull Request

## Code of Conduct

Please read [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).
