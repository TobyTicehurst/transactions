# How to run this code

`cargo run -- <input-filepath> > <output-filepath>`

Tests can act as examples of input and output, see tests/

# Code behaviour

The code initially reads from the input csv and at the end writes to the output csv but the meat of the program is parsing the transactions. Since this code is designed to work in an async way, transactions can't be immediately processed as each transaction relies on the previous transactions. For example, a "Chargeback" transaction prevents any further transactions from being processed. Transactions will first be categorised, sent to the required client and added to a list for later processing. At a later "sync" point, these lists of transactions will be processed in chronological order. Disputes, Resolves and Chargebacks are handled first, then Deposits and Withdrawals. The following behaviour occurs for each transaction type during that later pr:

Dispute - if client is not locked, look for a Deposit or Withdrawal with the same id. If found look for a Resolve or Chargeback with the same id. If a Deposit or Withdrawal is not found, move on (could remove the Dispute at this point). If a Resolve or Chargeback is found, process it now. If a Resolve or Chargeback is not found, mark the Deposit or Withdrawal as disputed. A disputed deposit removes the amount from available funds and adds it to held funds. A disputed Withdrawal adds the amount to held funds. A disputed deposit is allowed to make available funds negative.
Resolve - Remove both the Dispute and Resolve. Mark the transaction (Deposit or Withdrawal) as undisputed.
Chargeback - Remove the Dispute and Chargeback, and the transaction (Deposit or Withdrawal). Mark the client as locked and specify the chronology it was locked at
Deposit - if client is not locked, client's available funds increases by given amount
Withdrawal - if client is not locked, and if the withdrawal won't reduce available funds below 0, client's available funds decreases by given amount

# Chronology and transaction ordering

The problem statement described the need for transactions to be ordered but didn't specify a way to order them. Transaction id wouldn't be appropriate due to the possibility of re-ordering the csv file. I use chronology (think timestamp) to order all transactions. Given that in the future chronology could be represented as a timestamp (rather than a unique u64 as it currently is), chronology isn't necessarily unique. To preserve a unique and deterministic ordering, I use transaction id to order transactions with the same chronology, since the problem statement stated that transaction ids are unique.

# Current performance

Running: `python tests/performance_test.py`

Gives: Elapsed: 955.23ms

This tests 10 million random deposits and withdrawals and doesn't check for correctness. I am assuming that Disputes, Resolves and Chargebacks are rare.

# Thoughts on how to improve the code

- async - current the code is single threaded but build to be async. I would use a Hashmap of Clients where the Hashmap was protected by a RwLock and each Client was protected with a standard Mutex. 

# Initial thoughts (braindump)

Potential for multiple input sources

- csv file for machine testing (read file, then parse)
- csv file streaming (read file in one IO-bound thread)
- internals should be able to handle streaming from multiple tcp sockets

Precision issues with 4 decimal place decimal input to floating point binary

- could potentially use a fixed-point number
- when deserializing the number, take the integer part, multiply by 10,000, add the fractional part as a 4 digit integer

Overflow issues with u32 and u16 mentioned in brief

- u16_max 65,535
- u32_max 4,294,967,296 (4 billion)

Given this is being built like an ATM and there are ~17 billion credit and debit cards in the world, u64 is more appropriate for client ids
I see the problem statement says I am safe to use u16 but I do feel u64 is better


Input and Output can appear in any order and I would like the solution to be as async as possible. My current understanding:

- Clients have the properties: id, available funds, held funds, total funds, locked account
- id is a unique u64
- All funds are a 4 decimal point fixed-point number
- locked account is a boolean
- total funds = available funds + held funds. (I assume I don't need to store total funds separately?)
- Deposits and Withdrawals have transaction ids `tx`
- Disputes, Resolves and Chargebacks reference a single Deposit or Withdrawal via their transaction id
- A Dispute marks a Deposit or Withdrawal as "disputed"
- A Resolve is an "undo" of a Dispute
- A Chargeback is an "undo" of the Deposit or Withdrawal

Assumptions I am making

- Locked accounts should ignore all further transactions
- Available funds can be negative
- A Disputed Withdrawal is handled by adding the disputed amount to held funds, resolved by removing that amount from held funds, and a chargeback would move that amount from held funds to available funds
- Internal state can be in an incorrect state so long as the final output is correct. Can imagine a strategy where all deposits and withdrawals are done without checks until an issue like insufficient funds occurs. Since history may need to be edited via chargebacks locking accounts
- A data race can occur with a chargeback since transactions are not guaranteed to be ordered. The input file can be considered to be chronological so I may add an additional counter to the data to resolve these races. Any transactions which occurred chronologically after this point would then need to be reversed. Any transactions which happen chronologically before this point will still need to be parsed (therefore locking an account should store the time index it was locked at)
- Disputes are much rarer than Deposits and Withdrawals
- Client and Transaction IDs count up from 1 (or ideally 0, but they don't in the examples :( )). This isn't a required assumption, a collision checked hashmap would still be fine, but this is code I am creating, I'd like to think I would have control over this. I also don't think random client IDs would be a good security idea here (the threat being someone taking their own ID and adding 1 to get a valid ID) as it falls under security through obscurity (someone could just as easily try a million IDs).

Minimum Viable Product

- Parse command line input in the form: `cargo run -- transactions.csv > accounts.csv`
- Parse each transaction from the csv into a vector, adding chronology data
- Handle each transaction linearly, no async yet
- Output Client data to csv
