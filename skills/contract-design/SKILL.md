---
name: ckb-contract-design
description: Interactive 5-phase contract design skill for CKB. Walks through Cell state modeling, roles & permissions, state transitions, transaction templates, and security pre-review — all confirmed step by step before any code is written.
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

**Never write contract code without a confirmed design.** Walk through all 5 phases with the user, pause after each phase for explicit confirmation, and only hand off to the Contract Agent once every phase is approved.

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

**Ask these questions one at a time.**

**Question 4 — Roles**

> Who are the actors in your system? For each Cell type, who can:
>
> - **Create** it (produce it as an output)?
> - **Update** it (consume and recreate with modified data)?
> - **Destroy** it (consume without recreating)?
>
> Example roles: "the token issuer", "any user with a valid signature", "a DAO admin multisig", "anyone"

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
> - Any arithmetic invariants? (e.g., "sum of input amounts = sum of output amounts")
>
> Example for a token transfer:
> Pre-condition: sender owns input Balance Cell
> Constraint: sum(input amounts) == sum(output amounts)
> Signature: sender's Lock Script passes

**Phase 3 output — State Transition Diagram:**

```
State Transition Diagram
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Operation: create]
  Inputs:  (none, or: fee Cell)
  Outputs: [CellType1] with initial state
  Auth:    [who signs]
  Rules:   [validation constraints]

[Operation: update]
  Inputs:  [CellType1] (current state)
  Outputs: [CellType1] (new state)
  Auth:    [who signs]
  Rules:   [validation constraints]

[Operation: destroy]
  Inputs:  [CellType1]
  Outputs: (none — capacity returned to owner)
  Auth:    [who signs]
  Rules:   [validation constraints]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Cell Lifecycle:
  (nonexistent) ──create──▶ [CellType1] ──update──▶ [CellType1]
                                         ◀──update──
                                         ──destroy──▶ (capacity released)
```

⏸ **Pause here.** Ask: "Does this state transition model cover all the operations you need? Are the validation rules complete?"

---

### Phase 4: Transaction Templates

For each operation identified in Phase 3, generate a concrete transaction template. Reference the style from `agents/ckb-contract/transaction-patterns.md`.

**Phase 4 output — Transaction Templates:**

For each operation, produce:

````
## Transaction: [operation name]

**Purpose**: [one-sentence description]

**Structure**:

```typescript
const tx = ccc.Transaction.from({
  inputs: [
    // [description of each input Cell]
    { previousOutput: { txHash: "0x...", index: 0 } }, // [CellType]
  ],
  outputs: [
    // [description of each output Cell]
    {
      lock: [lockScript],      // [who owns it]
      type: [typeScript],      // [rules enforced, or omit if none]
      capacity: ccc.fixedPointFrom("[X]"), // [minimum CKB required]
    },
  ],
  outputsData: [
    // [encoded data for each output]
    ccc.bytesFrom("[encoded fields]"),
  ],
});

// cell_deps required:
tx.addCellDeps({ outPoint: [lockScriptDep], depType: "code" });
tx.addCellDeps({ outPoint: [typeScriptDep], depType: "code" });

// witnesses:
// index 0: [signature or witness data]
```

**Validation (Type Script checks)**:
- [rule 1]
- [rule 2]
````

⏸ **Pause here.** Ask: "Do these transaction templates match what you expect? Should I adjust any inputs, outputs, cell_deps, or validation rules?"

---

### Phase 5: Security Pre-Review

Apply the checklist from `agents/ckb-contract/security.md` to the confirmed design. Flag any risk points **before** any code is written.

**Phase 5 output — Security Pre-Review Report:**

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

Once all 5 phases are confirmed, output the **Contract Design Document**:

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
║ STATE TRANSITIONS                                             ║
║ [Phase 3 diagram]                                             ║
╠══════════════════════════════════════════════════════════════╣
║ TRANSACTION TEMPLATES                                         ║
║ [Phase 4 templates]                                           ║
╠══════════════════════════════════════════════════════════════╣
║ SECURITY NOTES                                                ║
║ [Phase 5 flagged items and mitigations]                       ║
╚══════════════════════════════════════════════════════════════╝
```

Then hand off to the Contract Agent:

> "Your contract design is complete and confirmed. I will now activate the CKB Contract Agent to implement this design. Load `agents/ckb-contract/SKILL.md` and proceed from Step 1 (Environment check) using the design document above."

## Design patterns reference

When answering questions in any phase, refer to `agents/ckb-contract/design-patterns.md` for established CKB patterns that match the user's scenario.
