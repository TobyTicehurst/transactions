use crate::util::Fixed;

#[derive(Debug, Default, Clone, Copy)]
pub struct Transaction {
    // id is unique, but does not specify ordering
    pub id: u64,
    // chronology may not be unique (can imagine it being a timestamp), but does specify ordering
    // in the case of disputed chronology (2 conflicting transactions happening at the same time), id will be used to order
    pub chronology: u64,
    pub amount: Fixed,
    pub disputed: bool,
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.chronology, self.id).partial_cmp(&(other.chronology, other.id))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        (self.chronology, self.id).eq(&(other.chronology, other.id))
    }
}

impl Transaction {
    pub fn new(id: u64, chronology: u64, amount: Fixed) -> Self {
        Self {
            id,
            chronology,
            amount,
            disputed: false,
        }
    }
}

#[derive(Debug)]
pub struct Claim {
    pub id: u64,
    pub chronology: u64,
    pub claim_type: ClaimType,
}

// TODO implement these as the same type
#[derive(Debug, Clone, Copy)]
pub struct Dispute {
    pub id: u64,
    pub chronology: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Resolve {
    pub id: u64,
    pub chronology: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Chargeback {
    pub id: u64,
    pub chronology: u64,
}

#[derive(Debug)]
pub struct TransactionMetadata {
    pub client_id: u64,
    pub transaction_id: u64,
    pub chronology: u64,
}

#[derive(Debug)]
pub enum ClaimType {
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub enum TransactionType {
    UpdateFunds(Fixed),
    Claim(ClaimType),
}

#[derive(Debug)]
pub struct UnprocessedTransaction {
    pub transaction_type: TransactionType,
    pub metadata: TransactionMetadata,
}

impl UnprocessedTransaction {
    pub fn new(
        transaction_type: TransactionType,
        client_id: u64,
        transaction_id: u64,
        chronology: u64,
    ) -> Self {
        Self {
            transaction_type,
            metadata: TransactionMetadata {
                client_id,
                transaction_id,
                chronology,
            },
        }
    }
}
