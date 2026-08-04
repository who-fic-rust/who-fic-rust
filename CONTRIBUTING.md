# Contributing to WHO-FIC-Rust

Thanks for your interest in contributing. This is a Cargo workspace of
seven crates implementing the WHO Family of International Classifications
for Rust — see [plan.md](plan.md) for the project's goals and design
principles, and [specs/](specs/index.md) for the detailed specification
each crate follows.

## Before you start

- **Read [specs/architecture.md](specs/architecture.md) first.** It covers
  the conventions every crate follows (code-type trait set, error shape,
  `serde` feature convention, testing standards) and, importantly, the
  **licensing constraint**: WHO owns the copyright on classification
  *content* (titles, definitions, full tabular lists). This repository
  implements code syntax/structure only and must never vendor that
  content — see that document for what's in bounds and what isn't.
- For anything beyond a small fix, consider opening an issue first to
  discuss the approach before writing code.

## Development setup

```sh
git clone https://github.com/who-fic-rust/who-fic-rust
cd who-fic-rust
cargo build --workspace --all-features
cargo test --workspace --all-features
```

Rust edition 2024, MSRV 1.85 (pinned in `[workspace.package]` in the root
`Cargo.toml`).

## Before opening a PR

Run the same checks CI runs, for the crate(s) you touched (or the whole
workspace):

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

If you touched `who-fic`'s feature flags, also check the feature
powerset (`cargo install cargo-hack` if you don't have it):

```sh
cargo hack check -p who-fic --feature-powerset
```

## Conventions

- **No vendored WHO content.** Individual example codes in docs/tests are
  fine (that's how every crate here already documents itself); full
  tabular lists, titles, or descriptions are not.
- **Code-type conventions.** If you're adding or changing a classification
  code type, follow the trait set and parsing conventions in
  `specs/architecture.md` (`FromStr`/`Display`/`TryFrom`/`as_str`,
  syntax-only validation, canonical-string `serde`).
- **Dependencies stay minimal.** Core crates aim for zero required
  dependencies; `who-fic-claml` (`quick-xml`) and `who-fic-icd-api`
  (`reqwest`, `tokio`) are the two documented exceptions, because their
  entire job requires them. New dependencies elsewhere need a good reason.
- **Tests.** Accept-lists and reject-lists for parsers, property tests
  (`proptest`) for round-tripping and panic-safety, rustdoc examples on
  public items. Match the existing style in the crate you're touching.
- **Specs stay in sync.** If your change alters documented behavior,
  update the matching file in `specs/` in the same PR.

## Reporting issues

Use the issue templates. For classification-specific questions (is a code
valid, what does WHO's real data say), please check WHO's own reference
guides first — this project implements syntax/structure, not a copy of
WHO's content, so some questions are genuinely out of scope here.

## License

By contributing, you agree your contributions are licensed under the same
dual [MIT](LICENSE-MIT) / [Apache-2.0](LICENSE-APACHE) terms as the rest
of the project.
