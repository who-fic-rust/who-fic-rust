//! Async client for the World Health Organization (WHO) ICD-API
//! (<https://id.who.int>).
//!
//! Unlike every other crate in this workspace, this crate makes network
//! calls — it is the one place credentials are required. See
//! `specs/who-fic-icd-api.md` in the repository for the full spec, verified
//! directly against WHO's own OpenAPI document at
//! <https://id.who.int/swagger/v2/swagger.json>.
//!
//! This crate does not cache, redistribute, or bundle WHO's classification
//! content; it is a thin wrapper around WHO's own REST API, which the user
//! authenticates against with credentials they register for themselves at
//! <https://icd.who.int/icdapi>.
//!
//! See [`IcdApiClient`] for the entry point and a usage example.

#![warn(missing_docs)]

mod client;
mod error;
mod types;

pub use client::{IcdApiClient, IcdApiClientBuilder};
pub use error::IcdApiError;
pub use types::{CodeInfo, Entity, Icd10Entity, LangString, SearchResultEntity, SearchResults};
