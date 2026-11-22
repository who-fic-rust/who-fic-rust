# WHO FIC Rust

World Health Organization (WHO) Family of International Classifications (FIC) Rust programming.

Main crate:

- `who-fic`: any common code, and crate feature options for including subcrates

Subcrates:

- `who-fic-icd`: International Classification of Diseases (ICD)
- `who-fic-icf`: International Classification of Functioning, Disability and Health (ICF)
- `who-fic-ichi`: International Classification of Health Interventions (ICHI)

## Contents

| Section | Covers |
| ------- | ------ |
| [§1 Rust checked arithmetic](rust-checked-arithmetic/index.md) | Production arithmetic never panics, wraps, truncates or invents a number: checked operations, recovery at the source with a declared `unwrap_or` fallback, typed errors, lossless conversions, aggregation, floats, time, lengths, the lints and `clippy.toml` bans that enforce it, and the tests that prove it |
