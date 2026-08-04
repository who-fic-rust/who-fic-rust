# Tutorial: WHO-FIC-Rust

A guided walkthrough of all seven crates, in the order you'd actually
reach for them. Each per-crate `README.md` has a terser "here's the API"
snippet; this ties them together with the reasoning in between. For the
authoritative details behind every example here, see the matching file in
[specs/](specs/index.md).

## 1. Pick your dependency

If you only need one classification, depend on its crate directly:

```toml
[dependencies]
who-fic-icd = "0.3"   # or who-fic-icf, who-fic-ichi
```

If you need more than one, or want the shared `Classification`/`FicError`
vocabulary, depend on the umbrella crate instead and reach the same types
through its re-exports:

```toml
[dependencies]
who-fic = "0.3"   # icd + icf + ichi all enabled by default
```

```rust
use std::str::FromStr;
use who_fic::icd::icd11::Icd11Code;   // == who_fic_icd::icd11::Icd11Code
```

The rest of this tutorial uses each subcrate's own path
(`who_fic_icd::...`) for clarity; swap in `who_fic::icd::...` etc. if
you're using the umbrella crate.

## 2. Parse and validate a code

Every classification code type in this workspace follows the same shape:
construct only via `FromStr`/`TryFrom<&str>` (never a bare constructor —
an `Icd11Code` that exists is guaranteed syntactically valid), format back
via `Display`/`as_str()`.

```rust
use std::str::FromStr;
use who_fic_icd::icd11::Icd11Code;

let code = Icd11Code::from_str("8b20").unwrap();  // case-insensitive input
assert_eq!(code.as_str(), "8B20");                // canonical uppercase
assert_eq!(code.to_string(), "8B20");

// Malformed input is a typed error, not a panic:
let err = Icd11Code::from_str("8I20").unwrap_err();  // 'I' never appears in ICD-11
```

This is true for `who_fic_icd::icd10::Icd10Code`, `who_fic_icf::IcfCode`,
`who_fic_ichi::IchiCode`, and the smaller types each crate builds on top
of (`who_fic_icd::icd11::ExtensionCode`, `who_fic_ichi::{Target, Action,
Means}`, ...). See `specs/architecture.md`'s "Code-type conventions" for
the full, exact list of traits every code type implements.

## 3. Navigate hierarchy

ICF codes form an explicit chapter → fourth-level tree; navigate it
without any data file, since the hierarchy is encoded in the code's own
digit structure:

```rust
use std::str::FromStr;
use who_fic_icf::{Component, IcfCode};

let code = IcfCode::from_str("b28010").unwrap();   // fourth level
assert_eq!(code.component(), Component::BodyFunctions);
assert_eq!(code.parent().unwrap().as_str(), "b2801");  // third level
assert_eq!(code.chapter().as_str(), "b2");             // straight to chapter
assert!(code.chapter().is_ancestor_of(&code));
```

ICD-10 and ICD-11 codes carry chapter membership too, though the mapping
is a lookup table (WHO's chapter-to-category-range table) rather than
something derivable from the code's digits alone:

```rust
use std::str::FromStr;
use who_fic_icd::icd10::Icd10Code;

let code = Icd10Code::from_str("I63.9").unwrap();
let chapter = code.chapter();   // Option: some codes are in reserved,
                                 // unassigned ranges — see specs/who-fic-icd.md
```

## 4. Build richer codes: qualifiers and axes

ICF codes are usually paired with a qualifier — and the qualifier's
*shape* depends on which of the four components the code belongs to
(body functions get one digit; environmental factors get a barrier-or-
facilitator distinction via `.`/`+`). The type system enforces this, so a
malformed combination is a parse error, not a runtime surprise:

```rust
use std::str::FromStr;
use who_fic_icf::QualifiedIcfCode;

assert!(QualifiedIcfCode::from_str("b280.2").is_ok());   // one digit: fine for 'b'
assert!(QualifiedIcfCode::from_str("e150.2").is_ok());   // barrier
assert!(QualifiedIcfCode::from_str("e150+2").is_ok());   // facilitator
assert!(QualifiedIcfCode::from_str("b280+2").is_err());  // '+' invalid outside 'e'
```

ICHI codes are always three axes (Target, Action, Means) composed with
dots; build one from parts or parse the dotted form directly:

```rust
use std::str::FromStr;
use who_fic_ichi::{Action, IchiCode, Means, Target};

let code = IchiCode::from_str("KAB.DB.AD").unwrap();
assert_eq!(code.target().as_str(), "KAB");

let same_code = IchiCode::from_parts(
    Target::from_str("KAB").unwrap(),
    Action::from_str("DB").unwrap(),
    Means::from_str("AD").unwrap(),
);
assert_eq!(code, same_code);
```

## 5. Add titles: the data-loading features (optional)

Everything above works with zero data files — it's pure syntax. To answer
"what is `1A00` actually *called*," you need WHO's own data, which this
project deliberately does not bundle (see `specs/architecture.md`'s
licensing section). Instead, enable a crate's `linearization` or `claml`
feature, download the matching export yourself from WHO, and build an
in-memory index from it:

```toml
[dependencies]
who-fic-icd = { version = "0.3", features = ["linearization"] }
who-fic-linearization = "0.1"
```

```rust,no_run
use std::fs::File;
use who_fic_icd::icd11::linearization::Icd11LinearizationIndex;
use who_fic_linearization::LinearizationReader;

// Downloaded yourself from https://icd.who.int/dev11/downloads
// (LinearizationMiniOutput-MMS-en.zip -> the .txt inside it).
let file = File::open("LinearizationMiniOutput-MMS-en.txt").unwrap();
let reader = LinearizationReader::from_reader(file);
let index = Icd11LinearizationIndex::from_rows(reader).unwrap();

let code = "1A00".parse().unwrap();
println!("{:?}", index.title(&code));   // Some("Cholera")
```

`who-fic-icd`'s `icd10::claml` module works the same way against a ClaML
XML file instead. All four data-loading index types across the three
classification crates share one shape — see `specs/architecture.md`'s
"Data-loading index conventions" for the full method list (`title`,
`get`, `iter`, `IntoIterator`, `len`, `is_empty`) and iteration-order
guarantee.

## 6. Ask WHO directly: the live API client (optional)

For anything the export-file approach can't answer — a code's current
status, a live search, WHO's postcoordination axis breakdown for a
cluster code — `who-fic-icd-api` talks to WHO's own ICD-API. This is the
one crate here that makes network calls and needs credentials (free
registration at <https://icd.who.int/icdapi>):

```rust,no_run
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
use std::str::FromStr;
use who_fic_icd::icd11::Icd11Code;
use who_fic_icd_api::IcdApiClient;

let code = Icd11Code::from_str("1A00").unwrap();   // validated locally, offline

let client = IcdApiClient::builder("my-client-id", "my-client-secret").build();
let info = client.code_info_typed("2024-01", "mms", &code).await?;
println!("{}", info.id);
# Ok(())
# }
```

This is exactly what
[`who-fic-icd-api/examples/lookup_code.rs`](who-fic-icd-api/examples/lookup_code.rs)
does, runnably:

```sh
WHO_ICD_API_CLIENT_ID=... WHO_ICD_API_CLIENT_SECRET=... \
    cargo run --example lookup_code -p who-fic-icd-api -- 1A00
```

## Where to go next

- [specs/index.md](specs/index.md) — the full specification, one file per
  crate.
- Each crate's own `README.md` — a shorter reference for that crate
  alone.
- [`who-fic-icd-api/examples/lookup_code.rs`](who-fic-icd-api/examples/lookup_code.rs)
  and every crate's `examples/readme.rs` — runnable code, not just
  snippets in a doc.
- [CONTRIBUTING.md](CONTRIBUTING.md) — if you want to change one of these
  crates rather than just use it.
