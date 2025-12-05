use anyhow::{Result, anyhow};
use num::{CheckedAdd, CheckedSub, Signed, Zero};

use crate::transactions::TransactionType::*;
use crate::transactions::transaction::{Chargeback, ClaimType, Dispute, Resolve};
use crate::util::merge_in_place;
use crate::{
    transactions::{Transaction, UnprocessedTransaction},
    util::Fixed,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct ClientState {
    available_funds: Fixed,
    held_funds: Fixed,
    locked: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Client {
    state: ClientState,
    // what chronology was the account locked at
    locked: Option<u64>,
    sorted_transactions: Vec<Transaction>,
    unsorted_transactions: Vec<Transaction>,
    disputes: Vec<Dispute>,
    resolves: Vec<Resolve>,
    chargebacks: Vec<Chargeback>,
}

impl Client {
    pub fn handle_transaction(&mut self, transaction: UnprocessedTransaction) -> Result<()> {
        // TODO combine id and chronology and implement Ord, Cmp
        let id = transaction.metadata.transaction_id;
        let chronology = transaction.metadata.chronology;

        // if we are locked, don't process any future transactions
        if let Some(locked_chronology) = self.locked
            && (locked_chronology, id) < (chronology, id)
        {
            // this is not a system error - in a more robust system I would return an error to the caller here
            return Ok(());
        }

        match transaction.transaction_type {
            UpdateFunds(amount) => self.handle_update_funds(id, chronology, amount),
            Claim(claim_type) => self.handle_claim(id, chronology, claim_type),
        };

        Ok(())
    }

    fn handle_update_funds(&mut self, id: u64, chronology: u64, amount: Fixed) {
        // add transaction in case of either:
        //  1. later dispute
        //  2. this transaction being an insufficient funds withdrawal and later processing an earlier deposit giving enough funds for withdrawal
        self.unsorted_transactions
            .push(Transaction::new(id, chronology, amount));
    }

    fn handle_claim(&mut self, id: u64, chronology: u64, claim_type: ClaimType) {
        match claim_type {
            ClaimType::Dispute => {
                self.disputes.push(Dispute { id, chronology });
            }
            ClaimType::Resolve => {
                self.resolves.push(Resolve { id, chronology });
            }
            ClaimType::Chargeback => {
                self.chargebacks.push(Chargeback { id, chronology });
            }
        }
    }

    //
    fn resolve_disputes(&mut self) {
        for (dispute_index, dispute) in self.disputes.clone().iter().enumerate() {
            // first check if transaction has been processed yet
            if let Some(transaction_index) = self.find_transaction_by_id(dispute.id) {
                // check if we have a matching resolution
                if let Some(resolve_index) = self.find_resolve_by_id(dispute.id) {
                    // just remove the dispute and resolve
                    self.disputes.remove(dispute_index);
                    self.resolves.remove(resolve_index);
                } else if let Some(chargeback_index) = self.find_chargeback_by_id(dispute.id) {
                    // lock
                    self.locked = Some(self.chargebacks[chargeback_index].chronology);
                    // remove the dispute and chargeback, and remove the transaction
                    self.disputes.remove(dispute_index);
                    self.chargebacks.remove(chargeback_index);
                    self.sorted_transactions.remove(transaction_index);
                }
                // if we don't have a matching resolution
                else {
                    // mark the transaction as disputed
                    self.sorted_transactions[transaction_index].disputed = true;
                }
            }
        }
    }

    fn find_resolve_by_id(&mut self, id: u64) -> Option<usize> {
        self.resolves
            .iter_mut()
            .enumerate()
            .find(|(_, resolution)| resolution.id == id)
            .map(|(index, _)| index)
    }

    fn find_chargeback_by_id(&mut self, id: u64) -> Option<usize> {
        self.chargebacks
            .iter_mut()
            .enumerate()
            .find(|(_, resolution)| resolution.id == id)
            .map(|(index, _)| index)
    }

    fn find_transaction_by_id(&mut self, id: u64) -> Option<usize> {
        self.sorted_transactions
            .iter_mut()
            .enumerate()
            .rev() // slight optimisation assuming we are more likely to dispute a more recent transaction
            .find(|(_, transaction)| transaction.id == id)
            .map(|(index, _)| index)
    }

    pub fn calculate_funds(&mut self) -> Result<()> {
        // TODO pass in max chronology

        // combine 2 sorted lists
        self.unsorted_transactions
            .sort_by_key(|t| (t.chronology, t.id));
        merge_in_place(&mut self.sorted_transactions, &self.unsorted_transactions);

        // resolve any disputes we can
        self.resolve_disputes();

        let mut state = ClientState {
            available_funds: Fixed::zero(),
            held_funds: Fixed::zero(),
            locked: self.locked.is_some(),
        };

        let max_chronology = self.locked.unwrap_or(u64::MAX);

        // then update client state with transactions
        for transaction in &self.sorted_transactions {
            // account should be locked beyond any chargebacks
            // may still have transactions beyond this point due to the async nature of the processing
            // TODO this doesn't handle equal chronology properly
            if transaction.chronology > max_chronology {
                break;
            }

            if transaction.disputed {
                // if withdrawal
                if transaction.amount.is_negative() {
                    // a disputed withdrawal isn't described in the brief so I am making assumptions
                    // assume that available funds shouldn't change as we don't want to prematurely add funds
                    // held funds should increase by the positive transaction amount so that if the transaction was indeed fraudulent,
                    //  those funds can later be added to available funds
                    state
                        .held_funds
                        .checked_add(&transaction.amount.abs())
                        .ok_or(anyhow!("Overflow caused by disputed transaction"))?;
                }
                // if deposit
                else {
                    // this is allowed to result in negative available funds
                    state
                        .available_funds
                        .checked_sub(&transaction.amount)
                        .ok_or(anyhow!("Overflow caused by disputed transaction"))?;

                    state
                        .held_funds
                        .checked_add(&transaction.amount)
                        .ok_or(anyhow!("Overflow caused by disputed transaction"))?;
                }
            } else {
                let available_funds = state
                    .available_funds
                    .checked_add(&transaction.amount)
                    .ok_or(anyhow!("Overflow caused by transaction"))?;

                // available_funds only allowed to be 0 or above
                if !available_funds.is_negative() {
                    state.available_funds = available_funds;
                }
            }
        }

        self.state = state;

        Ok(())
    }

    pub fn available_funds(&self) -> Fixed {
        self.state.available_funds
    }

    pub fn held_funds(&self) -> Fixed {
        self.state.held_funds
    }

    pub fn total_funds(&self) -> Fixed {
        self.available_funds() + self.held_funds()
    }

    pub fn is_locked(&self) -> bool {
        self.state.locked
    }
}
