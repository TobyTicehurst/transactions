use crate::clients::Client;
use crate::util::Fixed;
use anyhow::Result;
use csv::Writer;
use serde::{Serialize, Serializer};
use std::io;
use std::io::BufWriter;

#[derive(Debug, Serialize)]
struct CsvClient {
    #[serde(rename(serialize = "client"))]
    pub client_id: u64,
    #[serde(rename(serialize = "available"))]
    #[serde(serialize_with = "se_fixed")]
    pub available_funds: Fixed,
    #[serde(rename(serialize = "held"))]
    #[serde(serialize_with = "se_fixed")]
    pub held_funds: Fixed,
    #[serde(rename(serialize = "total"))]
    #[serde(serialize_with = "se_fixed")]
    pub total_funds: Fixed,
    pub locked: bool,
}

fn se_fixed<S>(fixed: &Fixed, se: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    String::serialize(&fixed.to_string(), se)
}

impl From<Client> for CsvClient {
    fn from(client: Client) -> Self {
        Self {
            client_id: 0,
            available_funds: client.available_funds(),
            held_funds: client.held_funds(),
            total_funds: client.total_funds(),
            locked: client.is_locked(),
        }
    }
}

pub fn write_clients_to_stdout(clients: &[Option<Client>]) -> Result<()> {
    let buf_writer = BufWriter::new(io::stdout());
    let mut writer = Writer::from_writer(buf_writer);
    for client in clients.iter().flatten() {
        let csv_client: CsvClient = client.clone().into();
        writer.serialize(csv_client)?;
    }

    Ok(())
}
