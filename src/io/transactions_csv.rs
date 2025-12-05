use crate::io::SerializedTransactionType;
use crate::transactions::transaction::ClaimType;
use crate::transactions::{TransactionType, UnprocessedTransaction};
use crate::util::Fixed;
use anyhow::{Result, anyhow};
use num::Signed;
use serde::{Deserialize, Deserializer, de};
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct CsvTransaction {
    #[serde(rename(deserialize = "type"))]
    pub type_name: SerializedTransactionType,
    #[serde(rename(deserialize = "client"))]
    pub client_id: u64,
    #[serde(rename(deserialize = "tx"))]
    pub transaction_id: u64,
    #[serde(rename(deserialize = "amount"))]
    #[serde(deserialize_with = "de_fixed")]
    #[serde(default)]
    pub amount: Option<Fixed>,
    #[serde(skip)]
    pub chronology: u64,
}

fn de_fixed<'de, D>(de: D) -> Result<Option<Fixed>, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(de)?;
    if str.is_empty() {
        return Ok(None);
    }

    let fixed =
        Fixed::from_str(str.as_str()).map_err(|err| de::Error::custom(err.to_string().as_str()))?;

    if fixed.is_positive() {
        Ok(Some(fixed))
    } else {
        Err(de::Error::custom("Amount must be positive"))
    }
}

impl fmt::Display for CsvTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CsvTransaction {{ type_name: {:?}, client_id: {}, id: {}, amount: {:?} }}",
            self.type_name, self.client_id, self.transaction_id, self.amount
        )
    }
}

impl TryFrom<CsvTransaction> for UnprocessedTransaction {
    type Error = anyhow::Error;

    fn try_from(csv_transaction: CsvTransaction) -> Result<Self> {
        let transaction_type = match csv_transaction.type_name {
            SerializedTransactionType::Deposit | SerializedTransactionType::Withdrawal => {
                TransactionType::UpdateFunds(
                    csv_transaction
                        .amount
                        .ok_or(anyhow!("Deposits and Withdrawals must specify amounts"))?,
                )
            }
            SerializedTransactionType::Dispute => TransactionType::Claim(ClaimType::Dispute),
            SerializedTransactionType::Resolve => TransactionType::Claim(ClaimType::Resolve),
            SerializedTransactionType::Chargeback => TransactionType::Claim(ClaimType::Chargeback),
        };

        Ok(UnprocessedTransaction::new(
            transaction_type,
            csv_transaction.client_id,
            csv_transaction.transaction_id,
            csv_transaction.chronology,
        ))
    }
}

// reads transactions from a file
// csv row -> CsvTransaction -> UnprocessedTransaction
pub fn read_transactions_from_csv_file(filepath: &str) -> Result<Vec<UnprocessedTransaction>> {
    let file = File::open(filepath)?;
    let buf_reader = BufReader::new(file);
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // allow for whitespace between fields and delimiters
        .from_reader(buf_reader)
        .into_deserialize()
        .enumerate()
        .map(|(i, result)| {
            result
                .map(|mut t: CsvTransaction| {
                    t.chronology = i as u64;
                    t
                })
                .map_err(|err| anyhow!("Failed to deserialize transaction: {err}"))?
                .try_into()
        })
        .collect()
}
