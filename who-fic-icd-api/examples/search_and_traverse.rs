//! Two more `IcdApiClient` operations beyond `examples/lookup_code.rs`:
//! searching a linearization by free text, and following an entity's
//! `parent`/`child` URI lists to its neighbors in the hierarchy.
//!
//! Needs real WHO ICD-API credentials (register for free at
//! <https://icd.who.int/icdapi>):
//!
//! ```sh
//! WHO_ICD_API_CLIENT_ID=... WHO_ICD_API_CLIENT_SECRET=... \
//!     cargo run --example search_and_traverse -p who-fic-icd-api -- cholera
//! ```

use std::env;
use who_fic_icd_api::IcdApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = env::args().nth(1).unwrap_or_else(|| "cholera".to_string());

    let client_id = env::var("WHO_ICD_API_CLIENT_ID")
        .map_err(|_| "set WHO_ICD_API_CLIENT_ID (register at https://icd.who.int/icdapi)")?;
    let client_secret = env::var("WHO_ICD_API_CLIENT_SECRET")
        .map_err(|_| "set WHO_ICD_API_CLIENT_SECRET (register at https://icd.who.int/icdapi)")?;
    let client = IcdApiClient::builder(client_id, client_secret).build();

    // Search the MMS linearization for the query text.
    let results = client.search("2024-01", "mms", &query).await?;
    println!(
        "{} match(es) for {query:?}:",
        results.destination_entities.len()
    );
    for hit in &results.destination_entities {
        println!(
            "  {} — {}",
            hit.chapter.as_deref().unwrap_or("?"),
            hit.title.as_deref().unwrap_or("(no title)")
        );
    }

    // Follow the first hit's parent entities, one level up.
    if let Some(first) = results.destination_entities.first() {
        let entity = client.entity(&first.id).await?;
        println!(
            "\n{} has {} parent(s):",
            entity.title.value,
            entity.parent.len()
        );
        for parent_uri in &entity.parent {
            // Foundation URIs are full entity() calls, e.g.
            // "http://id.who.int/icd/entity/257068234" -> the trailing
            // numeric ID is what entity() expects.
            if let Some(id) = parent_uri.rsplit('/').next() {
                let parent = client.entity(id).await?;
                println!("  {}", parent.title.value);
            }
        }
    }

    Ok(())
}
