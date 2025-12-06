pub mod clients;
pub mod io;
pub mod transactions;
pub mod util;

use crate::{clients::Client, transactions::UnprocessedTransaction};
use crate::io::serialized_client::*;
use io::transactions_csv::*;
use std::{
    collections::HashMap,
    sync::{Mutex, RwLock, Arc},
};
use tokio::task;
use util::Cli;
use tokio;

async fn work(clients: &mut Vec<Option<Client>>, unprocessed_transactions: &Vec<UnprocessedTransaction>) {
    eprintln!("Start");
    use std::time::Instant;
    let now = Instant::now();


    // A: Threaded Attempt - HashMap
    // let mut handles = vec![];
    // // assign transactions to clients
    // for transaction in unprocessed_transactions {
    //     let clients_clone = clients.clone();
    //     let client_id = transaction.metadata.client_id;

    //     clients_clone.write().unwrap().entry(client_id).or_insert(Mutex::new(Client::new(client_id as u64)));

    //     handles.push(task::spawn(async move {
    //         let client_read_lock = clients_clone.read().unwrap();
    //         let client = client_read_lock.get(&client_id).unwrap();
    //         client.lock().unwrap().handle_transaction(transaction).unwrap();
    //     }));
    // }

    // for handle in handles {
    //     handle.await.unwrap();
    // }

    // B: HashMap
    for transaction in unprocessed_transactions {
        let client_id = transaction.metadata.client_id as usize;

        // if client_id is off the end of the current list of clients
        if client_id >= clients.len() {
            clients.resize_with(client_id + 1, || None);
        }

        if let Some(client) = clients.get_mut(client_id) {
            let client = client.get_or_insert(Client::new(client_id as u64));
            client.handle_transaction(transaction).unwrap();
        } else {
            // unreachable error
        }
    }

    let elapsed = now.elapsed();
    eprintln!("Elapsed: {:.2?}", elapsed);

    eprintln!("Parse");

    // lock here to avoid needed to re-lock every loop
    std::thread::scope(|s| {
        // process each client
        // for client in clients.values_mut() {
        for client in clients.iter_mut().flatten() {
            s.spawn(move || {
                client.calculate_funds().unwrap();
            });
        }
    });

    let elapsed = now.elapsed();
    eprintln!("Elapsed: {:.2?}", elapsed);
}

#[tokio::main]
async fn main() {
    let csv_filepath = Cli::from_args().csv_filepath;

    // indexed by client_id (client_id 0 is allowed by this code)

    // A
    //let clients: Arc<RwLock<HashMap<u64, Mutex<Client>>>> = Arc::new(RwLock::new(HashMap::new()));

    // B
    //let mut clients: HashMap<u64, Client> = HashMap::new();

    let mut clients: Vec<Option<Client>> = vec![];

    eprintln!("Reading file");
    // read transactions from csv file
    let unprocessed_transactions = read_transactions_from_csv_file(csv_filepath.as_str()).unwrap();

    work(&mut clients, &unprocessed_transactions).await;

    // at this point we no longer care about performance
    let mut clients_clone = vec![];
    // for client in clients.values() {
    for client in clients.iter().flatten() {
        let client_clone = client.clone();
        clients_clone.push(client_clone);
    }
    write_clients_to_stdout(&clients_clone).unwrap();
}
