//! Parse an ICD-11 code with `who-fic-icd`, then resolve it against the
//! live WHO ICD-API with `who-fic-icd-api` — the two crates' intended
//! division of labor: one validates syntax offline, the other answers
//! "what does WHO's server actually say about this code."
//!
//! Needs real WHO ICD-API credentials (register for free at
//! <https://icd.who.int/icdapi>), read from the `WHO_ICD_API_CLIENT_ID`
//! and `WHO_ICD_API_CLIENT_SECRET` environment variables:
//!
//! ```sh
//! WHO_ICD_API_CLIENT_ID=... WHO_ICD_API_CLIENT_SECRET=... \
//!     cargo run --example lookup_code -p who-fic-icd-api -- 1A00
//! ```

use std::env;
use std::str::FromStr;
use who_fic_icd::icd11::Icd11Code;
use who_fic_icd_api::IcdApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_code = env::args().nth(1).unwrap_or_else(|| "1A00".to_string());
    let code = Icd11Code::from_str(&raw_code).map_err(|error| {
        format!("{raw_code:?} is not a syntactically valid ICD-11 code: {error}")
    })?;
    println!("Parsed locally: {code} (chapter {:?})", code.chapter());

    let client_id = env::var("WHO_ICD_API_CLIENT_ID")
        .map_err(|_| "set WHO_ICD_API_CLIENT_ID (register at https://icd.who.int/icdapi)")?;
    let client_secret = env::var("WHO_ICD_API_CLIENT_SECRET")
        .map_err(|_| "set WHO_ICD_API_CLIENT_SECRET (register at https://icd.who.int/icdapi)")?;
    let client = IcdApiClient::builder(client_id, client_secret).build();

    let info = client.code_info_typed("2024-01", "mms", &code).await?;
    println!("WHO says: {} (stem code: {:?})", info.id, info.stem_code);

    Ok(())
}
