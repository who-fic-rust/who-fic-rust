# Testing conventions

What each kind of crate in this workspace tests, and how, so a new
addition matches the existing pattern instead of inventing a new one.

## Pure code-parsing crates (`who-fic-icd`, `who-fic-icf`, `who-fic-ichi`)

Per module (roughly one per code type):

- **Accept-list**: real or realistic-shaped valid codes, asserting the
  parsed fields (`category()`, `component()`, `chapter()`, etc.).
- **Reject-list**: malformed input, asserting the *specific* expected
  error variant (not just "is an error") — e.g. `Icd10ParseError::Empty`
  vs. `InvalidCharacter { position, found }` vs. `InvalidLength { found }`.
  Cross-classification confusion belongs here too: an ICD-10 code fed to
  the ICD-11 parser, etc.
- **Property tests** (`proptest`, already a dev-dependency everywhere it's
  needed): parse→format→parse round trip on generated valid-shaped
  strings; arbitrary/garbage strings never panic the parser (this is the
  test that actually matters most for a hand-written grammar parser — a
  panic on malformed untrusted input is the failure mode to rule out).
- **`serde` round-trip** (behind `#[cfg(feature = "serde")]`, via
  `serde_json` as a dev-dependency): canonical-string serialize, parse
  back, compare.
- **Rustdoc doctests** on every public item — not optional decoration,
  these run under `cargo test` and are this workspace's primary
  usage-example documentation. Every method should have at least one.

## Format-parser crates (`who-fic-linearization`, `who-fic-claml`)

Same shape, plus: **hand-written fixtures matching the real export
format**, never vendored real WHO export files (see the licensing
constraint in `AGENTS.md`/`specs/architecture.md` — a WHO export file,
even used only as a test fixture, is WHO content). When adding a fixture,
base its shape on what's documented in the matching `specs/*.md` file
(which itself was verified against a real download, then deleted) — a
handful of representative rows/elements is enough, not a full file.

## Data-loading `*Index` adapters (the `claml`/`linearization` feature
modules in `who-fic-icd`/`who-fic-icf`/`who-fic-ichi`)

Beyond the round-trip/property style above: test the *lenient-skip*
behavior specifically (a row/class that doesn't parse as that
classification's code type must be silently excluded, not fail the whole
build) versus the *fatal* behavior (a malformed underlying file/reader
error must propagate). Both are meaningfully different code paths and
both need their own test — every existing adapter already does this
correctly (see e.g. `skips_row_with_code_that_fails_to_parse_as_icd11` /
`propagates_reader_errors` in `who-fic-icd/src/icd11.rs`), and new ones
should match. What Phase 11 actually found wrong in the existing four was
structural (map type, `iter()` shape — see `AGENTS/lessons.md`), not this
distinction; it's called out here because it's easy to get wrong, not
because it was.

## The network-calling crate (`who-fic-icd-api`)

No live WHO credentials exist for this workspace, and none should be
sought — the test suite runs entirely against a local `wiremock` mock
server (`wiremock` is a dev-dependency already):

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

let mock_server = MockServer::start().await;
Mock::given(method("POST")).and(path("/connect/token"))
    .respond_with(ResponseTemplate::new(200).set_body_json(/* ... */))
    .mount(&mock_server).await;

let client = IcdApiClient::builder("id", "secret")
    .token_url(format!("{}/connect/token", mock_server.uri()))
    .api_base_url(mock_server.uri())
    .build();
```

`IcdApiClientBuilder::token_url`/`::api_base_url` exist specifically to
make this possible — any new client method needs to be reachable through
a client built this way, not hardcoded against the real WHO URLs.

Cover: token fetch on first call, token *reuse* on a second call (assert
the mock token endpoint was hit exactly once across two API calls), token
*refresh* after simulated expiry, a successful response parsing into the
expected type, a non-2xx response mapping to the right error variant, and
a malformed-JSON response mapping to the right error variant. Use
`#[tokio::test]` (the `tokio` dev-dependency has `rt-multi-thread` +
`macros` already).

The one doctest in this crate that's allowed to skip actually running is
the top-level client usage example — it's marked ` ```no_run ` because it
genuinely needs live credentials and network access. Every other public
item's doctest should be a real, executing example (e.g. deserializing a
small hand-written fixture JSON string with `serde_json::from_str` needs
no network and should be a normal runnable doctest).

## Cross-cutting: full workspace verification

Before considering any change done, run the exact commands in
`AGENTS.md`'s "Build, test, lint" section against the *whole workspace*,
not just the crate(s) you touched — several real bugs in this project's
history (see `AGENTS/lessons.md`) were changes that looked correct when
checked crate-by-crate but broke under `cargo test --workspace
--no-default-features` or `--all-features` specifically because of
cross-crate interactions (feature-gating, example target name collisions).
