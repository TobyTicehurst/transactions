use anyhow::{Result, anyhow};
use csv::Error;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename(deserialize = "type"))]
    type_name: String, // String for now, will be enum
    #[serde(rename(deserialize = "client"))]
    client_id: u64,
    #[serde(rename(deserialize = "tx"))]
    transaction_id: u64,
    #[serde(rename(deserialize = "amount"))]
    amount: f64, // f64 for now, will be fixed point
}

fn read_transactions_from_csv_file(filepath: &str) -> Result<Vec<Transaction>> {
    let file = File::open(filepath)?;
    let buf_reader = BufReader::new(file);
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(buf_reader)
        .into_deserialize()
        .map(|result| result.map_err(|err| anyhow!("Failed to deserialize transaction: {err}")))
        .collect()
}

fn main() {
    let transactions = read_transactions_from_csv_file("tests/test_data.csv").unwrap();
    for transaction in transactions {
        // error here.
        println!("{:?}", transaction);
    }
}
