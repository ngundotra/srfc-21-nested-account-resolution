# SRFC 21 - Nested Account Resolution

Helper library is available at `additional-accounts-request`.

Examples of how to implement and use `additional-accounts-request` in `programs`.

# Introduction

This specification presents a solution to account resolution when using unknown programs on Solana. It is crafted to make the Solana ecosystem more accessible, secure, and user-friendly.

Central to this specification are Minimal Set of Accounts (MSAs) which are the smallest set of accounts required to execute a program instruction. 
An instruction's MSA is defined by the accounts required by its `aar` (additional accounts request) instruction.
Additional accounts can be requested through iterative simulation of the `aar` instruction, which returns a list of additional accounts required to execute the instruction.

This spec enables users to interact with programs directly through a block explorer, regardless of their technical expertise, whenever program developers provide a `aar` instruction. This approach democratizes access and increases the demand for security and transparency of program interactions.

The goal is to make the next generation of smart contract development flourish on Solana. 

# Work that is outside the scope of this sRFC

### Automatic Lookup Table Creation

Example code is provided for constructing tables for large transactions, but there is no harnessing in place to tear down unused tables.

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
