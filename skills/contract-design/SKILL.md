---
name: ckb-contract-design
description: Interactive 4-phase contract design skill for CKB. Walks through Cell state modeling, roles & permissions, state transitions, and security pre-review — all confirmed step by step before any code is written.
user-invocable: true
---

# CKB Contract Design

## When to activate

Activate this skill when the user wants to:

- Design a new CKB on-chain Script (Lock Script or Type Script)
- Plan the Cell structure and state model for a contract
- Think through permissions, state transitions, and security before coding
- Get a structured contract design document to hand off to the Contract Agent

## Core principle

**Never write contract code without a confirmed design.** Walk through all 4 phases with the user, pause after each phase for explicit confirmation, and only hand off to the Contract Agent once every phase is approved.

## Design workflow

### Phase 1: State Modeling

Guide the user to model their on-chain state using CKB's Cell primitives. Every Cell has four fields — help the user think through each:

| Field      | Description                                      |
| ---------- | ------------------------------------------------ |
| `capacity` | Storage rent in shannons (1 CKB = 10^8 shannons) |
| `lock`     | Who can spend this Cell (authorization script)   |
| `type`     | What rules govern this Cell's data (optional)    |
| `data`     | Application-specific binary payload              |

**Ask these questions one at a time. Wait for the user's answer before moving on.**

**Question 1 — Cell types**

> How many types of Cells does your application need?
> For each Cell type, describe:
>
> - What it represents (e.g., "a user balance Cell", "a global config Cell")
> - What data it holds
>
> Example: "I need two Cell types: an Info Cell that holds the token name and total supply, and a Balance Cell that holds each user's amount."

**Question 2 — Data format**

For each Cell type identified:

> What is the exact format of the `data` field?
>
> - List each field: name, type (u8 / u32 / u64 / u128 / bytes / string), size in bytes
> - Example: `{ amount: u128 (16 bytes), owner_lock_hash: Byte32 (32 bytes) }`
>
> Consider using [Molecule](https://github.com/nervosnetwork/molecule) for structured encoding — it is the standard serialization format on CKB.

**Question 3 — Capacity budget**

Based on their data format:

> Let's calculate the minimum `capacity` for each Cell type.
>
> CKB capacity formula:
>
> ```
> minimum_capacity = (lock_script_size + type_script_size + data_size + 8) bytes × 10^8 shannons
> ```
>
> - `8` bytes is the overhead for the capacity field itself
> - A typical secp256k1 lock adds ~53 bytes
> - A typical type script hash adds ~33 bytes
>
> Fill in your data sizes and we will compute the minimum CKB required per Cell.

**Phase 1 output — Cell Schema Table:**

After the user answers, present this table:

```
┌──────────────────────────────────────────────────────────────┐
│ Cell Schema                                                    │
├──────────────┬────────────────────────────┬───────────────────┤
│ Cell Type    │ Data Fields                │ Min Capacity      │
├──────────────┼────────────────────────────┼───────────────────┤
│ [CellType1]  │ [field: type (N bytes)]    │ [X CKB]           │
│ [CellType2]  │ [field: type (N bytes)]    │ [Y CKB]           │
└──────────────┴────────────────────────────┴───────────────────┘
```

⏸ **Pause here.** Ask: "Does this Cell schema look right? Should I adjust any field names, types, or sizes before we move on to permissions?"

---

### Phase 2: Roles & Permissions

> ⚠️ **CRITICAL CHECK**: Lock Script determines WHO can spend a Cell.
> Type Script only validates HOW a Cell can be spent. Type can NEVER override Lock.
> Always verify that your permission design is feasible under CKB's Lock/Type separation.

**Ask these questions one at a time.**

**Question 4 — Roles**

> Who are the actors in your system? For each Cell type, who can:
>
> - **Create** it (produce it as an output)?
> - **Update** it (consume and recreate with modified data)?
> - **Destroy** it (consume without recreating)?
>
> Example roles: "the token issuer", "any user with a valid signature", "a DAO admin multisig", "anyone"

**Question 4a — Lock Permission Boundary** [NEW]

> Before choosing Lock Scripts, verify that each actor's permissions are feasible under CKB's architecture:
>
> | Check          | Question                                                            |
> | -------------- | ------------------------------------------------------------------- |
> | Lock args      | Whose pubkey hash / script hash is in Lock args?                    |
> | Since          | Is a time-lock (`since` field) needed for conditional spending?     |
> | Multisig       | Are multiple parties needed to sign?                                |
> | Non-owner flow | Does your design require "anyone" or a "non-owner" to spend a Cell? |
>
> If your design requires a non-owner to spend a Cell, you **must** use one of the following — a Type Script alone is NOT sufficient:
>
> - `since` time-lock — allow spending after a time/block/epoch condition
> - `anyone_can_pay` — allow anyone to add capacity or UDT to the Cell
> - Omnilock with special conditions — flexible multi-auth support
> - Custom Lock Script — fully custom authorization logic
>
> Verify: does your permission model align with CKB's Lock/Type separation? If any role claims "anyone can trigger" an action on a Cell protected by a single-key Lock, the design is **invalid** — that Cell can only be spent by the key holder.

**Question 5 — Lock Script selection**

For each role:

> Which Lock Script should protect these Cells?
>
> | Lock Script            | Use case                                               |
> | ---------------------- | ------------------------------------------------------ |
> | `secp256k1_blake160`   | Standard single-key ownership (default for most Cells) |
> | `anyone_can_pay` (ACP) | Accept CKB/UDT from anyone without a signature         |
> | Omnilock               | Flexible: ETH keys, multisig, DAO lock, admin lock     |
> | Custom Lock Script     | Fully custom authorization logic                       |
> | Time-lock via `since`  | Delay spending until a block/time/epoch condition      |
>
> Pick the Lock Script for each Cell type, or describe your custom authorization requirement.

**Question 6 — Type Script responsibilities**

> What should the Type Script enforce?
>
> Common Type Script roles:
>
> - **Token issuance rules** — control minting and total supply
> - **Data integrity** — enforce Molecule schema, field constraints, valid ranges
> - **State machine** — allow only specific state transitions
> - **Unique instance** — Type ID pattern to guarantee only one Cell exists
> - **None needed** — the Lock Script alone is sufficient
>
> For each Cell type, does it need a Type Script? If yes, what should it validate?

**Phase 2 output — Permission Matrix:**

```
┌──────────────────────────────────────────────────────────────┐
│ Permission Matrix                                              │
├──────────────┬──────────────┬──────────────┬──────────────────┤
│ Cell Type    │ Create       │ Update       │ Destroy          │
├──────────────┼──────────────┼──────────────┼──────────────────┤
│ [CellType1]  │ [Role/Lock]  │ [Role/Lock]  │ [Role/Lock]      │
│ [CellType2]  │ [Role/Lock]  │ [Role/Lock]  │ [Role/Lock]      │
└──────────────┴──────────────┴──────────────┴──────────────────┘

Lock Scripts:
- [CellType1]: [Lock Script name + brief rationale]
- [CellType2]: [Lock Script name + brief rationale]

Type Scripts:
- [CellType1]: [Type Script responsibilities, or "none"]
- [CellType2]: [Type Script responsibilities, or "none"]
```

⏸ **Pause here.** Ask: "Does this permission model look right? Should I adjust any roles, Lock Script choices, or Type Script responsibilities?"

---

### Phase 2.5: Data Placement Design

Before defining state transitions, guide the user to decide **where each data field lives**. This is a critical CKB design decision that affects Cell size, on-chain query support, and capacity cost.

**Placement options:**

| Placement   | Characteristics                                                     | Best for                                                      |
| ----------- | ------------------------------------------------------------------- | ------------------------------------------------------------- |
| Cell Data   | Persisted on-chain, stored by all nodes, consumes capacity          | State that must survive across transactions (balance, owner)  |
| Witness     | Transaction-scoped, not stored long-term, no capacity cost          | Temporary verification data (signatures, proofs, parameters)  |
| Lock args   | Part of the Lock Script, defines Cell identity and spending control | Identity and authorization data (owner pubkey hash, multisig) |
| `cell_deps` | Read-only reference to another live Cell                            | Shared configuration, oracle data, Script code                |

**Question 6.5 — Data placement**

For each data field identified in Phase 1:

> Where should this field be stored?
>
> **Place in Cell Data if:**
>
> - It must persist across transactions (e.g., token balance, rental rate)
> - Other contracts need to read it from the Cell
> - You need to query it on-chain (e.g., "find all Cells where status = active")
>
> **Place in Witness if:**
>
> - It is only needed for single-transaction validation (e.g., a signature, a proof)
> - It can be derived from other data (e.g., total cost = rate × duration)
> - It is a one-time parameter (e.g., a nonce, an operation code)
>
> **Place in Lock args if:**
>
> - It defines who controls the Cell (e.g., owner public key hash)
> - It is part of the authorization identity
>
> ⚠️ **Warning**: Witness data is NOT stored on-chain after the transaction. You cannot query or retrieve it later. If you need the data after the transaction completes, it must go in Cell Data.
>
> Example for a billboard rental contract:
>
> | Field       | Candidates       | Choice    | Rationale                                         |
> | ----------- | ---------------- | --------- | ------------------------------------------------- |
> | owner       | Data / Lock args | Lock args | Identity belongs in Lock, controls spending       |
> | rate        | Data / Witness   | Data      | Persists across transactions, modifiable by owner |
> | lease_start | Data / Witness   | Data      | Must be queryable to check lease status           |
> | duration    | Data / Witness   | Witness   | Only needed during rent validation                |
> | deposit     | Data / Witness   | Data      | Must persist for refund calculation               |
> | renter      | Data / Lock args | Data      | Not in Lock args because Lock is owner-controlled |

**Phase 2.5 output — Data Placement Table:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Data Placement                                                            │
├──────────────┬──────────────────┬──────────────┬─────────────────────────┤
│ Field        │ Candidates       │ Placement    │ Rationale               │
├──────────────┼──────────────────┼──────────────┼─────────────────────────┤
│ [field1]     │ Data / Lock args │ [chosen]     │ [why]                   │
│ [field2]     │ Data / Witness   │ [chosen]     │ [why]                   │
│ [field3]     │ Data / Witness   │ [chosen]     │ [why]                   │
└──────────────┴──────────────────┴──────────────┴─────────────────────────┘

Common Patterns:
  Pattern A — All state in Data (simple, easy to query, higher capacity cost)
  Pattern B — Minimal Data + Witness (smaller Cells, harder to query history)
  Pattern C — Data + Lock args (separate identity from application state)

See "Data placement patterns" section at the end of this document for details.
```

⏸ **Pause here.** Ask: "Does this data placement look right? For each field in Witness, confirm you will not need to query it after the transaction. For each field in Data, confirm the capacity cost is acceptable."

---

### Phase 3: State Transitions

**Ask these questions one at a time.**

**Question 7 — Operations**

> List all the operations your contract needs to support.
>
> Common operations: `create`, `update`, `transfer`, `burn`, `mint`, `lock`, `unlock`, `claim`, `merge`, `split`
>
> For each operation:
>
> - What is its name?
> - Which Cell types are consumed (inputs)?
> - Which Cell types are produced (outputs)?

**Question 8 — Validation rules**

For each operation:

> What are the pre-conditions and validation rules?
>
> - What must be true about the input Cells?
> - What constraints must the output Cells satisfy?
> - Who must sign (which Lock Scripts run)?
> - What data goes in the Witness for this operation? (Refer to your Data Placement table from Phase 2.5)
> - Any arithmetic invariants? (e.g., "sum of input amounts = sum of output amounts")
>
> Example for a token transfer:
> Pre-condition: sender owns input Balance Cell
> Constraint: sum(input amounts) == sum(output amounts)
> Signature: sender's Lock Script passes
> Witness: sender signature in `WitnessArgs.lock`

**Phase 3 output — State Transition Diagram:**

```
State Transition Diagram
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Operation: create]
  Data Placement:
    "[field] in Data because [reason]"
    "[field] in Witness because [reason]"

  Inputs:
    [0] [fee Cell or specific Cell]
        Lock: [Lock Script name] args: [Lock args]
        Type: [Type Script name, or none]

  Witnesses:
    [0] lock: [signature or authorization data]
        input_type: [Type Script verification data, if any]

  Outputs:
    [0] [CellType1] with initial state
        Lock: [Lock Script name] args: [Lock args]
        Type: [Type Script name]
        Data: {[field: type], [field: type]} — from Phase 2.5
        Capacity: [X CKB]

  Auth:    [who signs — which Lock Scripts run]
  Rules:   [validation constraints]

[Operation: update]
  Data Placement:
    "[field] updated in Data because [reason]"
    "[field] in Witness because [reason]"

  Inputs:
    [0] [CellType1] (current state)
        Lock: [Lock Script name] args: [Lock args]
        Type: [Type Script name]
        Data: {[current fields]}

  Witnesses:
    [0] lock: [signature]
        input_type: {[verification data from Phase 2.5 Witness fields]}

  Outputs:
    [0] [CellType1] (new state)
        Lock: [Lock Script name] args: [Lock args]
        Type: [Type Script name]
        Data: {[updated fields]}
        Capacity: [X CKB]

  Auth:    [who signs]
  Rules:   [validation constraints]

[Operation: destroy]
  Inputs:
    [0] [CellType1]
        Lock: [Lock Script name] args: [Lock args]
        Type: [Type Script name]
        Data: {[current fields]}

  Witnesses:
    [0] lock: [signature]

  Outputs: (none — capacity returned to owner)

  Auth:    [who signs]
  Rules:   [validation constraints]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Cell Lifecycle:
  (nonexistent) ──create──▶ [CellType1] ──update──▶ [CellType1]
                                         ◀──update──
                                         ──destroy──▶ (capacity released)
```

**Phase 3 checklist — verify before moving on:**

- [ ] Every input and output Cell has Lock Script, Type Script, and Data explicitly labeled
- [ ] Every field's placement (Data / Witness / Lock args) matches the Phase 2.5 table
- [ ] Witness structure is fully specified for each operation
- [ ] Fields placed in Witness are confirmed not needed for post-transaction queries
- [ ] Fields placed in Data have acceptable capacity cost
- [ ] Validation rules reference the correct data source (Data vs Witness)

⏸ **Pause here.** Ask: "Does this state transition model cover all the operations you need? Are the validation rules complete?"

---

### Phase 4: Security Pre-Review

Apply the checklist from `agents/ckb-contract/security.md` to the confirmed design. Flag any risk points **before** any code is written.

**Phase 4 output — Security Pre-Review Report:**

```
Security Pre-Review
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Based on your design, here are the security considerations:

✅ / ⚠️  Capacity validation
  [Assessment: does the Type Script enforce minimum capacity on outputs?]

✅ / ⚠️  Type Script creation vs update paths
  [Assessment: does the Type Script distinguish creation (no group inputs) from update?]

✅ / ⚠️  Cell counting
  [Assessment: does the Type Script verify the number of inputs/outputs in the group?]

✅ / ⚠️  Token amount balance
  [Assessment: if tokens are involved, is sum(inputs) >= sum(outputs) enforced?]

✅ / ⚠️  Overflow-safe arithmetic
  [Assessment: are u128 amounts handled with checked arithmetic?]

✅ / ⚠️  Lock Script presence
  [Assessment: does the Type Script verify that required Lock Scripts are present?]

✅ / ⚠️  Witness authentication
  [Assessment: is signature verification delegated to the Lock Script correctly?]

✅ / ⚠️  Since time-lock (if applicable)
  [Assessment: are time-lock conditions on since fields verified?]

✅ / ⚠️  Reinitialization attack
  [Assessment: can an attacker create a Cell with arbitrary data and bypass the Type Script?]

⚠️  Items requiring attention:
  - [List each flagged risk with a recommended mitigation]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

⏸ **Pause here.** Ask: "Do you want to revise the design to address any of these security points before implementation begins?"

---

## Design complete — Handoff to Contract Agent

Once all 4 phases are confirmed, output the **Contract Design Document**:

```
╔══════════════════════════════════════════════════════════════╗
║               CONTRACT DESIGN DOCUMENT                        ║
╠══════════════════════════════════════════════════════════════╣
║ Project:       [name]                                         ║
║ Date:          [date]                                         ║
╠══════════════════════════════════════════════════════════════╣
║ CELL SCHEMA                                                   ║
║ [Phase 1 table]                                               ║
╠══════════════════════════════════════════════════════════════╣
║ PERMISSIONS                                                   ║
║ [Phase 2 matrix]                                              ║
╠══════════════════════════════════════════════════════════════╣
║ DATA PLACEMENT                                                ║
║ [Phase 2.5 table -- field to Data / Witness / Lock args]       ║
╠══════════════════════════════════════════════════════════════╣
║ STATE TRANSITIONS                                             ║
║ [Phase 3 diagram with Lock/Type/Witness per operation]        ║
╠══════════════════════════════════════════════════════════════╣
║ SECURITY NOTES                                                ║
║ [Phase 4 flagged items and mitigations]                       ║
╚══════════════════════════════════════════════════════════════╝
```

### Save design artifacts to `doc/`

After generating the design document, save both design artifacts to the project's `doc/` directory so they are version-controlled alongside the code:

1. **Create the `doc/` directory** if it does not already exist.
2. **Save the State Transition Diagram** to `doc/state-transition-diagram.md` — include the full Phase 3 output (the diagram with all operations, inputs, outputs, witnesses, auth, rules, and the Cell lifecycle).
3. **Save the CONTRACT DESIGN DOCUMENT** to `doc/contract-design-document.md` — include the complete document with all 5 sections (Cell Schema, Permissions, Data Placement, State Transitions, Security Notes).

Use Markdown format for both files. Preserve the ASCII tables and diagrams as fenced code blocks so they render correctly. Include a YAML front-matter header with the project name and date in each file.

Example file structure after saving:

```
doc/
├── state-transition-diagram.md
└── contract-design-document.md
```

Then hand off to the Contract Agent:

> "Your contract design is complete and confirmed. The design artifacts have been saved to `doc/state-transition-diagram.md` and `doc/contract-design-document.md`. I will now activate the CKB Contract Agent to implement this design. Load `agents/ckb-contract/SKILL.md` and proceed from Step 1 (Environment check) using the design document above."

## Design patterns reference

When answering questions in any phase, refer to `agents/ckb-contract/design-patterns.md` for established CKB patterns that match the user's scenario.

## Data placement patterns

Reference these common patterns when guiding the user through Phase 2.5 (Data Placement Design).

### Pattern A: All state in Cell Data

Store every field in the Cell's `data` field.

```
Cell
├── lock:  owner lock
├── type:  app Type Script
└── data:  {field1, field2, field3, ...}    ← everything here
Witness: signature only
```

- **Pros**: Easy to query on-chain, simple to validate, complete state in one place
- **Cons**: Larger Cell, higher capacity cost, all fields consume on-chain storage
- **Example**: sUDT Balance Cell (amount in data), Info Cell (name + total supply in data)
- **Best for**: Simple contracts where all state must be queryable

### Pattern B: Minimal Data + Witness supplement

Keep only essential persistent state in Cell Data; pass temporary or derivable data in Witness.

```
Cell
├── lock:  owner lock
├── type:  app Type Script
└── data:  {core_field1, core_field2}       ← minimal persistent state
Witness:
├── lock:  signature
└── input_type: {param1, param2, proof}     ← operation parameters
```

- **Pros**: Smaller Cell, lower capacity cost, flexible operation parameters
- **Cons**: Cannot query Witness data after transaction, must reconstruct history from transactions
- **Example**: Payment channel (balance in data, settlement proof in witness)
- **Best for**: Contracts with operation-specific parameters or large verification data

### Pattern C: Lock args + Data separation

Store identity and authorization data in Lock Script `args`; store application state in Cell Data.

```
Cell
├── lock:  CustomLock args: {owner_pubkey_hash}   ← identity here
├── type:  app Type Script
└── data:  {app_state_field1, app_state_field2}    ← app state here
Witness: signature + operation data
```

- **Pros**: Clean separation of identity and application logic, Lock handles authorization naturally
- **Cons**: Requires custom Lock Script, Lock args changes require Cell recreation
- **Example**: Custom lock contracts where the owner identity is embedded in the Lock
- **Best for**: Contracts where authorization logic is complex or multi-party
