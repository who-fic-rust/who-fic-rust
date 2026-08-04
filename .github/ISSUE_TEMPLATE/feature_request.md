---
name: Feature request
about: Propose new functionality
title: ""
labels: enhancement
assignees: ""
---

**Which crate(s)**
e.g. `who-fic-icd`, or a new crate.

**What's missing**
A clear description of the gap.

**Proposed API (if you have one in mind)**

```rust
// Sketch of the type/function signature you'd expect.
```

**Licensing check**
Does this require vendoring WHO classification *content* (titles,
descriptions, full tabular lists)? If so, see
[specs/architecture.md](../../specs/architecture.md) — this project
implements code syntax/structure only, and content-requiring features
belong in a user-supplied-data loader (like `who-fic-linearization`/
`who-fic-claml`) or a live API client (like `who-fic-icd-api`), not
bundled directly.

**Alternatives considered**
Any other approaches you thought about.
