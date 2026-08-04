# who-fic-icd-api

An async client for the World Health Organization (WHO)
[ICD-API](https://icd.who.int/icdapi) (`id.who.int`) — the live web
service WHO runs for looking up and searching ICD-10 and ICD-11 entities.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. Unlike every other crate in that workspace, this one makes
network calls: it's the live-data companion to
[`who-fic-icd`](https://crates.io/crates/who-fic-icd), for when you need
an entity's actual title or want to resolve a postcoordinated code's axes
— information that requires asking WHO, not something this crate (or any
crate in this workspace) bundles or vendors.

You supply your own WHO ICD-API credentials, registered for free at
<https://icd.who.int/icdapi>, subject to WHO's own terms and rate limits.

Endpoint paths, headers, and the OAuth2 flow are verified directly against
WHO's own OpenAPI spec (`https://id.who.int/swagger/v2/swagger.json`); see
`specs/who-fic-icd-api.md` in the repository for the full reference.

## Example

```rust,no_run
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
use who_fic_icd_api::IcdApiClient;

let client = IcdApiClient::builder("my-client-id", "my-client-secret").build();

// Look up a Foundation entity by its numeric ID.
let entity = client.entity("257068234").await?;
println!("{}", entity.title.value);

// Resolve an ICD-11 MMS code to its entity and postcoordination axes.
let info = client.code_info("2024-01", "mms", "1A00").await?;
println!("{}", info.id);
# Ok(())
# }
```

(This example needs real WHO ICD-API credentials and network access, so
it isn't run as part of this README's tests.)

## What this crate implements

- OAuth2 client-credentials authentication, with automatic token caching
  and refresh.
- `entity` / `entity_search` — WHO's ICD-11 Foundation.
- `linearization_entity` / `search` — a specific linearization (e.g. MMS).
- `code_info` / `code_info_typed` — resolve a classification code
  (accepts a raw `&str` or a `who_fic_icd::icd11::Icd11Code`) to its
  entity and postcoordination axis breakdown.
- `icd10_category` / `icd10_category_typed` — ICD-10 category lookup by
  code.

See `specs/who-fic-icd-api.md` for what's deliberately out of scope in
this first release (autocode, describe, lookup, POST search, and a few
other endpoints WHO's API exposes).

## Testing without WHO credentials

`IcdApiClientBuilder::token_url` / `::api_base_url` let you point the
client at a local mock server instead of the real WHO endpoints — this
crate's own test suite uses that with `wiremock`, with no live network
access or credentials required.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
