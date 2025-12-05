pub mod clients;
pub mod io;
pub mod transactions;
pub mod util;

use crate::clients::Client;
use crate::io::serialized_client::*;
use io::transactions_csv::*;
use util::Cli;

fn main() {
    let csv_filepath = Cli::from_args().csv_filepath;

    // indexed by client_id (client_id 0 is allowed by this code)
    let mut clients: Vec<Option<Client>> = vec![];

    // read transactions from csv file
    let unprocessed_transactions = read_transactions_from_csv_file(csv_filepath.as_str()).unwrap();
    //let processed_transactions = vec![];

    for transaction in unprocessed_transactions {
        let client_id = transaction.metadata.client_id as usize;

        // if client_id is off the end of the current list of clients
        if client_id >= clients.len() {
            clients.resize_with(client_id + 1, || None);
        }

        if let Some(client) = clients.get_mut(client_id) {
            let client = client.get_or_insert_default();
            client.handle_transaction(transaction).unwrap();
        } else {
            // unreachable error
        }
    }

    for client in clients.iter_mut().flatten() {
        client.calculate_funds().unwrap();
    }

    write_clients_to_stdout(&clients).unwrap();
}
