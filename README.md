# SRFC 21 - Nested Account Resolution

Examples in `tests/`


# Resolution Strategies

### 1 - Intra VM Preflight

Program X needs remaining accounts in a specified order
You can deliver it to them by asking X for the ordering of accounts that it needs

Pros: 
- easier to integrate with existing programs


Cons: 
- this doubles the deserialization work done by X, so CU expensive
- potentially unsafe to be passing around signatures more than necessary

### 2 - Iterative expansion

Same idea as 1, except iteratively deepen the preflight, starting with bare mininum of accounts.

Pros:
- mimicks off-chain logic exactly

Cons:
- Multiplies effort of runtime to do same action, which is derive accounts
- not anymore secure than other interactions

### 3 - Send all accounts

Since the security model is that we're basically giving program X full reign & control over our accounts,
might as well just give them everything, and let them decide what to do with it.

Pro:
- simple to write
- cheaper on CU
- just as secure as other methods

Cons:
- potentially unsafe to build marketplaces because the bag of unknown accounts can grow dramatically


# Note to Self

- `git stash pop` to get iterative stuff back
- Right now I'm in the middle of switching all `execute` paths to just give all remaining accounts to each program

# Patterns & Concepts

### Automatic Account Resolution for TransactionInstructions

Programs can adhere to interfaces, even with different account derivation schemes,
 by defining instructions in their IDL that conform to Minimal Instruction Account
 Meta Interfaces.

 MIAMI instructions require a minimal set of accounts that have semantic definitions, like `owner`, `destination`, or `authority`, and may optionally define a longer list of additional accounts that it needs to complete successfully.

 These additional accounts must be derived through iteratively simulating a separate `preflight` instruction that defines these additional accounts through its return data. 

 Each MIAMI instruction must have its own valid preflight instruction that defines its additional accounts. The corresponding `preflight` instruction can be found by prepending `preflight_` to MIAMI instruction's name.

 Off-chain clients must compose a Transaction Instruction against a MIAMI instruction by appending the list of account metas as defined by the return data of its `preflight` instruction.


Two concepts here:
- Minimal Instruction Account Metas Interfaces (MIAMI)
- Preflight instructions

### MIAMI Account Separation

When executing multiple CPIs to MIAMI instructions, 
it can become difficult to know which of your additional accounts
belong to which CPI call.

An example application would be a p2p marketplace that has to execute a
two MIAMI `transfer` instructions between parties.

We propose a method for solving this that uses `delimiter` pubkeys 
in the preflight payload.

# Hosted Features

### Automatic Lookup Table Creation

Coming soon

### Indexing (see `ngundotra/crud-indexing`)

See separate repo. Out of scope for this account resolution patterns.

# Considered Features That Were Removed

### Paging

The return data of the `preflight` instruction must be defined as follows (see `AdditionalAccounts` struct in `additional-accounts-request` crate).

Preflight instructions also take an additional byte of instruction data used to determine the requested account page. 

We imagine that for 99% of use cases the page will be 0. But to future-proof ourselves, we are going to allow accounts to be paged in quantites of 30.

Preflight instructions must return valid data for any requested page. 

Preflight instructions must be iteratively simulated with previously 
requested accounts appended to the instruction, until the return data says that
there are no more accounts to be returned.

#### Reason for Removal

This is impossible to support when you are making multiple MIAMI calls. 
The whole point of this framework is to enable composability on Solana, which we define
as the making calls to unknown programs with unknown execution paths with known accounts.

We cannot reasonably support paging in preflight instructions that for instructions that make 
multiple MIAMI calls because we do not have a way of easily delimiting which page we
should request for any preflight instruction after the first.



# Benchmark Results

Transferring a Linked List:
- Base: { num: 2, computeUnits: 16173 }
- CPI: { num: 2, computeUnits: 63378 }
- CPI-CPI: { num: 2, computeUnits: 116856 }

| Program | Number of Accounts | Compute Units |
| --- | ---- | ---- |
| Base | 1 | 14833 |
| Base | 2 | 16173 |
| Base | 3 | 17442 |
| Base | 10 | 26325 |
| CPI | 1 | 25470 |
| CPI | 2 | 27518 |
| CPI | 3 | 29491 |
| CPI | 10 | 44901 |
| CPI-CPI | 1 | 35948 |
| CPI-CPI | 2 | 39058 |
| CPI-CPI | 3 | 41767 |
| CPI-CPI | 10 | 64046 |

Transferring a Ownership List

| Program | Number of Accounts | Compute Units |
| --- | ---- | ---- |
| Base | 1 | 2600 |
| Base | 2 | 3040 |
| Base | 3 | 3849 |
| Base | 31 | 15800 |
| CPI | 1 | 14042 |
| CPI | 2 | 15188 |
| CPI | 3 | 30747 |
| CPI | 31 | 66756 |
| CPI-CPI | 1 | 25561 |
| CPI-CPI | 2 | 27494 |
| CPI-CPI | 3 | 66301 |
| CPI-CPI | 31 | 119991 |


Swapping Ownership Lists
| Program | Number of Accounts in Ownership List (per side) | Compute Units |
| ---       | ---                                   | ---------      |
| CPI       | 3                                     | 29479          |
| CPI       | 5                                     | 37681          |
| CPI       | 10                                    | 52649          |
| CPI       | 25                                    | 108666         |

Swapping Linked Lists for Linked Lists

| Program | Number of Accounts in Linked List (per side) | Compute Units  |
| ---       | ---                                        | ---------      |
| CPI       | 1                                          | 49984          |
| CPI       | 2                                          | 54107          |
| CPI       | 9                                          | 85240          |