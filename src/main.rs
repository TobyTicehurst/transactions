use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    csv_filepath: String,
}


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
    let cli = Cli::parse();

    let transactions = read_transactions_from_csv_file(cli.csv_filepath.as_str()).unwrap();
    for transaction in transactions {
        // error here.
        println!("{:?}", transaction);
    }
}
