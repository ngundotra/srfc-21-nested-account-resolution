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


Iterative:
Linked List (CPI)
{ num: 3, computeUnits: 24637 }