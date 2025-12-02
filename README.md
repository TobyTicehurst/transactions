# Initial thoughts

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
