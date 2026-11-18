Here are **prioritized, actionable quality improvement tasks** for the WHO-FIC-Rust project, organized by category. Each task is designed for AI execution, with clear acceptance criteria and estimated effort.

---

---

# 🎯 **NEW VISITOR EXPERIENCE & PROMOTION**

## High-Priority: First Impressions

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add project website/branch** | Create `gh-pages` branch with compiled docs + landing page. Use `cargo doc --no-deps` output as foundation, add narrative landing page linking to all crates, tutorial, and FAQ | GitHub Pages deployed at `who-fic-rust.github.io/who-fic-rust/`, links work, docs render | Medium (2-4h) |
| **Create "Quick Start" guide** | New file `QUICKSTART.md` with copy-paste snippets for most common use cases (parse ICD-11 code, validate, get title from file, live API lookup). Link from README | File exists, all snippets compile, linked from README's top section | Low (1-2h) |
| **Add repository topics** | Add GitHub topics: `who`, `fic`, `icd`, `icf`, `ichi`, `classification`, `health-informatics`, `rust`, `parsers` | Repository has 8-10 relevant topics visible on GitHub page | Low (15m) |
| **Add discussion forum setup** | Create GitHub Discussions with categories: Q&A, Ideas, Show & Tell, Announcements. Seed with 3-5 example questions from FAQ | Discussions enabled with 4 categories, pinned "Welcome" post | Low (30m) |
| **Create social media presence** | Set up project Twitter/X account (@who_fic_rust) and Mastodon. Cross-link from README | Accounts exist, bios describe project, links in README | Medium (1h) |
| **Add project logo/banner** | Create simple logo (WHO-FIC-Rust text with WHO colors). Add to README, use as repo social preview image | SVG logo in `docs/logo.svg`, referenced in README, used as GitHub social card | Medium (2h) |

## Medium-Priority: Documentation Depth

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Create architecture decision records (ADRs)** | Document key decisions in `docs/adr/` directory. Start with: Why no vendored WHO content, Why separate parser crates, Why umbrella crate pattern | ADR-001 through ADR-005 exist, follow standard ADR template, linked from README | Medium (3-5h) |
| **Add comparison table** | Create `COMPARISON.md` comparing this project to other ICD/WHO code libraries (Python, Java, JS). Highlight Rust advantages (type safety, zero-cost) | File exists, compares 5+ alternatives, honest about tradeoffs | Medium (2h) |
| **Create use case recipes** | New `RECIPES.md` with common patterns: validate a code list, build a code index from file, search ICD-11 API, convert between ICD-10 and ICD-11 formats | 5-7 recipes, each with complete compilable example | Medium (2-3h) |
| **Add video walkthrough** | Record 10-15 min screencast: clone repo, build, run examples, explain architecture. Host on YouTube, embed in README | Video linked from README, covers all 7 crates | High (4h) |
| **Create interactive playground** | Set up Rust Playground links for each major example. Users can edit/compile without local setup | 5+ playground links in docs, all working | Medium (2h) |
| **Add translation support** | Create `docs/translations/` directory. Add CONTRIBUTING section on how to translate docs | Directory exists, CONTRIBUTING has translation guide, issue template for translations | Low (1h) |

## Low-Priority: Community Building

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Create good-first-issue labels** | Tag issues suitable for newcomers. Add `good first issue`, `help wanted`, `documentation` labels | 10+ issues tagged, label descriptions defined | Low (30m) |
| **Add project roadmap** | Create `ROADMAP.md` with 6-month vision, planned features, release schedule | File exists, updated quarterly, linked from README | Low (1h) |
| **Create community health file** | Add `COMMUNITY.md` with contributor statistics, recognition, and community metrics | File exists with contributor list, metrics | Low (1h) |
| **Add newsletter signup** | Create simple mailing list for project updates using free tier of Mailchimp/TinyLetter | Signup form linked from README | Medium (2h) |
| **Create blog** | Start project blog in `docs/blog/` with posts: project origins, architecture deep dive, release announcements | 3+ blog posts, RSS feed generated | Medium (3-5h) |

---

---

# 🛠️ **ENGINEERING & CODE QUALITY**

## High-Priority: Critical Gaps

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Implement no_std support** | Add `no_std` feature to core crates (`who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`). Remove `std` dependencies where possible, add `alloc` where needed | All 4 crates compile with `--no-default-features --features no_std`, tests pass | High (8-12h) |
| **Fix TUTORIAL.md version references** | Update all version pins from `0.4` to current version. Verify all snippets compile against published crates | All version references current, all snippets compile | Low (1h) |
| **Add MSRV test matrix** | Extend CI to test against multiple Rust versions (1.98, stable, beta, nightly) | CI workflow includes rust-toolchain matrix, all versions pass | Medium (2h) |
| **Implement Cluster serde** | Add `serde` implementations for `Cluster` and `ClusterStem` in `who-fic-icd` (identified in backlog) | Serialization/deserialization works, round-trip tests pass | Medium (2-3h) |
| **Add rustdoc examples for Icd11ClassEntry** | Add doc examples for all 6 accessors (identified in backlog as gap) | All 6 methods have rustdoc `#[example]`, doc tests pass | Low (1-2h) |
| **Fix version drift in specs** | Audit all `specs/*.md` files for version references. Replace with pointers to `Cargo.toml` or remove | No version numbers in specs/, all reference source of truth | Low (1h) |

## Medium-Priority: Code Health

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add property-based tests** | Expand `proptest` usage for all parsers. Generate random valid/invalid codes, verify round-trips | All 4 classification crates have property tests, coverage >80% | Medium (4-6h) |
| **Implement fuzzing** | Add `cargo fuzz` targets for all code parsers. Use afl/libfuzzer | Fuzz targets exist for all 4 classification crates, CI runs fuzz for 60s | High (6-8h) |
| **Add benchmark suite** | Create `benches/` directory with Criterion.rs benchmarks for parsing, validation, formatting | Benchmarks for all code types, CI tracks performance | Medium (3-4h) |
| **Improve error messages** | Enhance parse error messages with context (which character failed, what was expected). Add `source()` chains | All parse errors have helpful messages, `source()` implemented where applicable | Medium (3-5h) |
| **Add Display for all error types** | Ensure all error types have human-readable `Display` impls, not just `Debug` | All error types have `Display`, error messages are user-friendly | Low (1-2h) |
| **Add TryFrom<&[u8]> for code types** | Implement `TryFrom<&[u8]>` for all code types to avoid UTF-8 validation overhead | All code types implement `TryFrom<&[u8]>`, tests added | Low (1h) |
| **Optimize parsing performance** | Profile and optimize hot paths in parsers. Consider using `lexgen` or `nom` for zero-copy parsing | 10-20% improvement in parsing benchmarks | High (6-8h) |
| **Add const constructors** | Add `const fn` constructors for code types where possible (ICD-10, ICD-11, ICF, ICHI) | All code types have const `new_unchecked()` for FFI, safe constructors where feasible | Medium (3-4h) |

## Low-Priority: Polish

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add From/Into for code strings** | Implement `From<CodeType> for String`, `From<CodeType> for Cow<str>` | All code types have these impls | Low (1h) |
| **Add AsRef<str> for code types** | Implement `AsRef<str>` for all code types | All code types implement `AsRef<str>` | Low (30m) |
| **Add PartialEq<&str> for code types** | Allow `code == "A00"` without explicit parsing | All code types can compare to `&str` | Low (1h) |
| **Add Hash for all code types** | Ensure all code types are `Hash` (they should be, verify) | All code types derive or implement `Hash` | Low (30m) |
| **Add Ord for all code types** | Implement `Ord` for all code types with semantic ordering | All code types implement `Ord`, ordering is documented | Medium (2h) |
| **Add Serialize/Deserialize for error types** | Add `serde` support for all error types | All error types serialize/deserialize | Medium (2h) |
| **Add more derives** | Add `Clone`, `Copy`, `Eq`, `PartialEq`, `Debug`, `Hash` where missing | All public types have appropriate derives | Low (1h) |

---

---

# 🧪 **TESTING IMPROVEMENTS**

## High-Priority: Coverage Gaps

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add integration tests** | Create `tests/integration/` with cross-crate tests. Test umbrella re-exports, feature flag combinations | Integration tests exist for all major feature combinations, pass | Medium (3-4h) |
| **Test no-default-features** | Ensure all crates compile and tests pass with `--no-default-features` | CI job added, all crates pass | Medium (2h) |
| **Add negative tests** | Expand test suites with explicit rejection tests for malformed inputs. Use `trybuild` for compile-time tests | All crates have negative test modules, 100+ rejection test cases | Medium (3-5h) |
| **Test serde round-trips** | Add comprehensive serde round-trip tests for all serializable types | All serializable types have round-trip tests | Medium (2-3h) |
| **Test feature flag combinations** | Add `cargo hack check --feature-powerset` for all crates, not just `who-fic` | All crates have powerset CI jobs | Medium (2h) |
| **Add docsrs tests** | Ensure all rustdoc examples compile and pass as doc tests | `cargo test --doc` passes for all crates | Low (1h) |

## Medium-Priority: Test Infrastructure

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add test coverage reporting** | Integrate `cargo tarpaulin` or `grcov` into CI for coverage reports | CI generates coverage report, uploads as artifact | Medium (2h) |
| **Create test fixtures** | Extract test data into shared fixtures directory. Use for data-loading tests | Shared fixtures exist, used by multiple crates | Medium (2-3h) |
| **Add snapshot testing** | Use `insta` for snapshot testing of serialization, error messages | Snapshot tests for serialization, error formatting | Medium (3h) |
| **Add mutable test data** | Create `tests/fixtures/` with real (but minimal) WHO export samples for testing parsers | Fixtures exist, parsers tested against real data formats | Medium (2-4h) |
| **Improve test organization** | Reorganize tests: unit tests in module, integration tests in `tests/`, examples in `examples/` | Clear separation, no duplication | Low (1-2h) |
| **Add test matrix for MSRV** | Test against MSRV (1.98) explicitly in CI | CI job tests against 1.98 | Low (1h) |

## Low-Priority: Advanced Testing

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add contract tests** | Create contract tests for API client (`who-fic-icd-api`) using recorded HTTP interactions | Contract tests exist, use VCR or similar | Medium (3h) |
| **Add chaos testing** | Add tests that simulate network failures, timeouts for API client | Chaos tests exist, handle error cases | Medium (2h) |
| **Add performance regression tests** | Add tests that fail if performance degrades beyond threshold | Performance regression CI job | Medium (2h) |
| **Add memory safety tests** | Add tests using `loom` for concurrent access patterns | Loom tests for shared state | High (4h) |
| **Test panic safety** | Add tests that verify no panics on invalid input (use `catch_unwind`) | Panic safety tests exist for all public APIs | Medium (2h) |

---

---

# 🔍 **AUDITING & SECURITY**

## High-Priority: Security Hardening

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Run comprehensive audit** | Run `cargo audit`, `cargo deny`, `cargo outdated`, `cargo crev` | All audits pass, no vulnerabilities | Medium (2h) |
| **Add security policy** | Create `SECURITY.md` with vulnerability reporting process, response SLA | File exists, linked from GitHub security settings | Low (1h) |
| **Add supply chain security** | Add `Cargo.lock` to repository, enable dependency verification | `Cargo.lock` committed, CI verifies lockfile | Low (30m) |
| **Add SBOM generation** | Generate Software Bill of Materials (SBOM) in CI using `cargo-bom` or similar | SBOM generated in CI, uploaded as artifact | Medium (2h) |
| **Audit unsafe code** | Review all `unsafe` blocks in codebase. Document safety invariants | All unsafe blocks have `// SAFETY:` comments, invariants documented | Medium (3-5h) |
| **Add dependency pinning** | Pin all dev-dependencies to specific versions in CI | CI uses pinned versions | Low (1h) |

## Medium-Priority: Code Review

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add linting for clippy** | Add `clippy.toml` with project-specific lint configuration | Configuration file exists, CI uses it | Low (1h) |
| **Add custom lints** | Create custom `clippy` lints for project-specific patterns (e.g., ensure all code types implement trait set) | Custom lints defined, CI enforces | High (4-6h) |
| **Audit error handling** | Review all error handling for consistency. Ensure all errors are `#[non_exhaustive]` where appropriate | Error handling consistent, documented | Medium (3h) |
| **Audit documentation** | Verify all public items have rustdoc, all examples compile | `cargo doc --document-private-items` passes, doc tests pass | Medium (2h) |
| **Review license compliance** | Verify all dependencies have compatible licenses. Update `deny.toml` if needed | All dependencies have MIT/Apache-2.0 compatible licenses | Medium (1-2h) |
| **Add code review checklist** | Create `REVIEW_CHECKLIST.md` for PR reviewers | Checklist exists, used in PR template | Low (1h) |

## Low-Priority: Advanced Auditing

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add formal verification** | Use `prusti` or similar for formal verification of parsers | Formal verification for at least one parser | High (8-12h) |
| **Add static analysis** | Integrate `cargo-semver-checks` into CI for breaking change detection | CI job runs semver checks on PRs | Medium (2h) |
| **Add dependency graph analysis** | Generate and analyze dependency graph for potential issues | Dependency graph generated, analyzed | Medium (2h) |
| **Review macro usage** | Audit all procedural macros for safety and hygiene | Macros reviewed, documented | Medium (2h) |
| **Add memory safety audit** | Use `cargo-geiger` to detect unsafe usage | Geiger report generated, reviewed | Medium (1-2h) |
| **Add semantic versioning checks** | Verify all breaking changes are properly version-bumped | All releases follow semver, changelog accurate | Low (1h) |

---

---

# 📊 **METRICS & OBSERVABILITY**

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Add CI metrics dashboard** | Create dashboard showing CI status, test results, coverage trends | Dashboard deployed (GitHub, external), updates automatically | Medium (3-4h) |
| **Add download metrics** | Track crates.io download statistics. Add badge to README | Download badges from crates.io in all READMEs | Low (1h) |
| **Add contributor statistics** | Track and display contributor metrics (lines of code, commits, issues) | Metrics generated, displayed somewhere | Low (1h) |
| **Add performance metrics** | Track and display performance benchmarks over time | Benchmark history maintained | Medium (2h) |
| **Add code quality metrics** | Track code quality metrics (complexity, duplication, maintainability) | Metrics generated, tracked | Medium (2h) |

---

---

# 🚀 **PROMOTION & OUTREACH**

## High-Priority: Visibility

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Publish on crates.io categories** | Ensure all crates have correct categories on crates.io | All crates properly categorized | Low (30m) |
| **Add to awesome-rust** | Submit PR to `awesome-rust` list under appropriate categories | PR merged to awesome-rust | Low (30m) |
| **Create Reddit post** | Post project announcement on r/rust, r/healthinformatics | Reddit posts created, linked from README | Low (1h) |
| **Add to Rust users forum** | Post on users.rust-lang.org | Forum post created | Low (30m) |
| **Submit to this week in Rust** | Submit for inclusion in TWiR | Submitted, ideally included | Low (30m) |
| **Add Hacktoberfest support** | Add `hacktoberfest` topic, create Hacktoberfest-ready issues | Repository tagged, 10+ Hacktoberfest issues | Low (1h) |

## Medium-Priority: Content Marketing

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Write Medium blog post** | Publish "Building WHO-FIC-Rust: Lessons in Type Safety" on Medium/Dev.to | Article published, shared on social media | Medium (3-4h) |
| **Create conference talk** | Prepare 20-30 min talk: "Type-Safe Health Classification Codes in Rust" | Talk outline, slides, demo ready | High (8-10h) |
| **Submit to RustConf** | Submit talk proposal to RustConf or similar | Proposal submitted | Low (1h) |
| **Create tutorial video series** | 3-part YouTube series: Intro, Deep Dive, Advanced Usage | Videos published, playlist created | High (10-15h) |
| **Write guest posts** | Write guest posts for Rust blogs, health tech publications | 2-3 guest posts published | Medium (4-6h) |
| **Create Twitter thread** | 10-tweet thread explaining project value proposition | Thread published, pinned | Low (1h) |

## Low-Priority: Partnerships

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Reach out to WHO** | Contact WHO about potential collaboration or recognition | Email sent, response documented | Low (1h) |
| **Partner with health tech companies** | Reach out to companies using WHO classifications | 3-5 companies contacted | Low (1h) |
| **Create case studies** | Document real-world usage of the library | 2-3 case studies published | Medium (3-4h) |
| **Join Rust Foundation** | Explore Rust Foundation membership | Application submitted or criteria documented | Low (1h) |

---

---

# 🎓 **UPSKILLING & LEARNING**

## For Contributors

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Create contributor learning path** | Add `LEARNING_PATH.md` with curated resources for new contributors | File exists, covers Rust basics to advanced patterns | Medium (2-3h) |
| **Add Rust mentorship** | Create mentorship program for new Rust contributors | Mentorship guide exists, mentors identified | Medium (2h) |
| **Add pair programming sessions** | Schedule regular pair programming sessions for contributors | Sessions scheduled, recorded, shared | Medium (2-3h prep + time) |
| **Create code review workshops** | Host workshops on effective code review for this project | Workshop materials created, sessions held | Medium (3-4h) |
| **Add architecture workshops** | Host workshops explaining project architecture | Workshop materials created | Medium (3-4h) |

## For Users

| Task | Description | Acceptance Criteria | Effort |
|------|-------------|---------------------|--------|
| **Create learning resources** | Add `LEARN.md` with curated list of Rust resources for health informatics | File exists, covers relevant topics | Low (1-2h) |
| **Add WHO classification guides** | Create guides on WHO classifications for developers | Guides exist for ICD-10, ICD-11, ICF, ICHI | Medium (4-6h) |
| **Create health informatics primer** | Write primer on health classification codes for software developers | Primer exists, covers basics | Medium (2-3h) |
| **Add domain glossary** | Create glossary of health informatics terms used in project | Glossary exists, linked from docs | Low (1h) |

---

---

# 📈 **SUMMARY & RECOMMENDATIONS**

## Quick Wins (Low Effort, High Impact)

1. **Add repository topics** (15m)
2. **Create good-first-issue labels** (30m)
3. **Add project logo** (2h)
4. **Fix TUTORIAL.md version references** (1h)
5. **Add rustdoc examples for Icd11ClassEntry** (1-2h)
6. **Add SECURITY.md** (1h)

## High Value (Medium Effort, High Impact)

1. **Add project website/gh-pages** (2-4h)
2. **Create QUICKSTART.md** (1-2h)
3. **Add no_std support** (8-12h) - *Major feature*
4. **Implement Cluster serde** (2-3h)
5. **Add integration tests** (3-4h)
6. **Run comprehensive audit** (2h)
7. **Add test coverage reporting** (2h)

## Strategic (High Effort, Long-term Impact)

1. **Add video walkthrough** (4h)
2. **Create tutorial video series** (10-15h)
3. **Implement fuzzing** (6-8h)
4. **Add formal verification** (8-12h) - *Research project*
5. **Create conference talk** (8-10h)

## Immediate Next Steps

Start with **Quick Wins** to improve the project's visibility and polish, then tackle **High Value** items. The `no_std` support and comprehensive audit are particularly valuable for a library targeting production health informatics use.

---

**Would you like me to elaborate on any specific category or create a detailed implementation plan for a particular task?**
