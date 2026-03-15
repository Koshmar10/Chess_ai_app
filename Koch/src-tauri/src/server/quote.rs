use rand::seq::IndexedRandom;
use std::{fs, path::Path};

use serde::Deserialize;
use serde_json;

use crate::server::server::ServerState;

#[derive(Deserialize)]
pub struct Quote {
    pub id: usize,
    pub name: String,
    pub quote: String,
}

impl ServerState {
    pub fn get_quote(&self) -> String {
        let default = String::from("black");
        let full_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../src/assets/quotes/quotes.json"
        ));
        let json_string = match fs::read_to_string(full_path) {
            Ok(json_string) => json_string,
            Err(e) => {
                eprintln!("Failed to read quotes file: {:?}", e);
                return default;
            }
        };
        let quote_vec: Vec<Quote> = match serde_json::from_str(&json_string) {
            Ok(vector) => vector,
            Err(e) => {
                eprintln!("Failed to parse quotes JSON: {:?}", e);
                return default;
            }
        };

        return match quote_vec.choose(&mut rand::rng()) {
            Some(quote) => String::from(format!("- {} -", quote.quote)),
            None => default,
        };
    }
}
