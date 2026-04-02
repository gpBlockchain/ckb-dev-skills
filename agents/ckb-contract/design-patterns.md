# CKB Contract Design Patterns

A quick-reference guide for common CKB on-chain design patterns. Use this during the contract design phase to select the right pattern for your scenario.

## Pattern Overview

| Pattern               | Applicable Scenario                                       | Key Cell Structure Idea                                       |
| --------------------- | --------------------------------------------------------- | ------------------------------------------------------------- |
| Single-Cell State     | Global state (e.g., token info, registry)                 | One Cell holds all global state; Type ID ensures uniqueness   |
| Multi-Cell Collection | Per-user distributed state (e.g., UDT balances)           | One Cell per user; identical Type Script differentiates token |
| Type ID Singleton     | Uniqueness guarantee (governance, configuration center)   | Type ID pattern; creation validates first input outpoint      |
| Lock + Type Layering  | Separating authorization from business rules              | Lock controls "who can act"; Type controls "how to act"       |
| Since Time-Lock       | Delayed operations (DAO unlock, vesting, cliff)           | `since` field encodes block / epoch / timestamp condition     |
| Witness Data Pattern  | Off-chain proof verified on-chain (oracle, sig aggregate) | Payload in `witnesses`; Script validates it                   |
| Dep Cell Reference    | Shared read-only configuration                            | Config data in `cell_deps`; consumed by reference only        |

---

## Pattern 1: Single-Cell State

### Overview

Store the entire global state of a contract in a single Cell. The Type Script enforces that exactly one such Cell exists at all times (using the Type ID pattern) and that state transitions are valid.

### Applicable scenarios

- Token metadata / Info Cell (total supply, name, decimals)
- Global registry (list of authorized participants)
- Protocol configuration (fee rate, admin address)

### Cell structure

```
Info Cell
├── capacity:  [minimum CKB]
├── lock:      admin secp256k1 lock (or multisig)
├── type:      InfoTypeScript { args: type_id_hash }
└── data:      Molecule-encoded global state
               e.g., { name: Bytes, total_supply: u128, admin: Byte32 }
```

### Notes

- Use **Type ID** to guarantee uniqueness — only one Cell with this `type_hash` can exist.
- On creation, validate `first_input.out_point` to derive the Type ID.
- On update, verify exactly one input and one output in the group; state transition rules apply.
- Keep the Cell `data` as small as possible to minimize capacity cost.

### Reference

- [Type ID RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)
- [sUDT Info Cell](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md)

---

## Pattern 2: Multi-Cell Collection

### Overview

Each user (or entity) owns an independent Cell carrying their piece of state. All Cells in the collection share the same Type Script, which distinguishes the token or object type.

### Applicable scenarios

- UDT (fungible token) balance Cells — one per user
- NFT or Spore Cell collection — one per asset
- Order book entries — one Cell per open order

### Cell structure

```
Balance Cell (per user)
├── capacity:  [minimum CKB, typically 142 CKBytes for UDT]
├── lock:      owner's secp256k1 / Omnilock
├── type:      UDTTypeScript { args: issuer_lock_hash }
└── data:      u128 (little-endian amount, 16 bytes)
```

### Notes

- The Type Script's `args` field encodes the token identity (typically the issuer's lock hash).
- On transfer, verify `sum(input amounts) >= sum(output amounts)` — no unintended minting.
- Mint (input < output) is only allowed when the issuer's lock passes.
- Use `Source::GroupInput` / `Source::GroupOutput` to iterate only Cells belonging to this Type Script.

### Reference

- [sUDT specification](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md)
- [xUDT specification](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0052-extensible-udt/0052-extensible-udt.md)

---

## Pattern 3: Type ID Singleton

### Overview

Guarantee that exactly one Cell with a specific Type Script exists on-chain at any time. This is the foundation for upgradable Scripts, unique governance tokens, and singleton configuration stores.

### Applicable scenarios

- Upgradable Script deployment (store the binary in a unique Cell)
- Governance contract (one vote-tally Cell per proposal)
- Protocol configuration Cell that must be unique

### Cell structure

```
Singleton Cell
├── capacity:  [minimum CKB]
├── lock:      admin lock
├── type:      { code_hash: TYPE_ID_CODE_HASH, hash_type: "type", args: type_id }
└── data:      [contract state or script binary]
```

### Creation logic (Type Script must validate)

```rust
// On creation: no group inputs exist
// Derive type_id from first input outpoint + output index
let type_id = blake2b(first_input_out_point || output_index_as_u64);
// Verify Script args == type_id
```

### Notes

- The `TYPE_ID_CODE_HASH` is a well-known hash built into CKB consensus.
- Once created, the `args` (type_id) is fixed — it cannot change even on upgrade.
- Upgrade = consume the old Cell + produce a new Cell with the same `args` but updated `data`.
- Only one Cell with a given `type_hash` can exist in the live Cell set.

### Reference

- [Type ID RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)
- [ckb-std type_id helpers](https://docs.rs/ckb-std/latest/ckb_std/)

---

## Pattern 4: Lock + Type Layering

### Overview

Separate authorization ("who can act") from business logic ("how to act") by using two independent Scripts on the same Cell. The Lock Script handles identity and signatures; the Type Script enforces data rules and state transitions.

### Applicable scenarios

- Any Cell where ownership and data rules are independently replaceable
- Contracts where multiple lock types (secp256k1, multisig, time-lock) should all work with the same data rules
- Upgradable lock without changing the data logic, or vice versa

### Cell structure

```
Layered Cell
├── capacity:  [minimum CKB]
├── lock:      [any standard or custom Lock Script]   ← "who can act"
├── type:      [data-rule Type Script]                ← "how to act"
└── data:      [application state]
```

### Execution model

```
When consuming this Cell:
  1. Lock Script runs → must return 0 (authorization)
  2. Type Script runs → must return 0 (data validity)
  Both must pass for the transaction to succeed.
```

### Notes

- Keep Lock and Type Scripts loosely coupled — they should not depend on each other's `args`.
- This pattern enables hot-swapping the Lock (e.g., user rotates key) without invalidating the Type.
- The Type Script should validate data fields, not make assumptions about the Lock Script used.

---

## Pattern 5: Since Time-Lock

### Overview

Use the `since` field on transaction inputs to enforce that a Cell cannot be spent until a specific block number, block timestamp, or epoch condition is met.

### Applicable scenarios

- DAO withdrawal unlock period
- Token vesting / cliff schedule
- Escrow with time-based release
- Delayed governance execution

### Cell structure

```
Time-Locked Cell
├── capacity:  [minimum CKB]
├── lock:      time-lock Lock Script (checks since field) or standard lock
├── type:      [optional: escrow Type Script]
└── data:      [optional: unlock condition parameters]
```

### Since encoding

```
since field (8 bytes, u64):
  bits[63]:    metric flag (0 = block, 1 = time/epoch)
  bits[62]:    relative flag (0 = absolute, 1 = relative)
  bits[61-56]: sub-type (0x00 = block, 0x01 = epoch, 0x02 = timestamp)
  bits[55-0]:  value

Examples:
  absolute block 1000:        0x0000_0000_0000_03E8
  relative 100 blocks:        0x4000_0000_0000_0064
  absolute timestamp (ms):    0xA000_XXXX_XXXX_XXXX
```

### Notes

- Use `ckb-std`'s `Since` struct to parse and validate the `since` field safely.
- CKB consensus enforces `since` — you do not need to check it in your Type Script unless applying additional constraints.
- Verify `since` is set correctly in the Lock Script; an attacker can omit or weaken it if the Lock does not check.
- Test edge cases: epoch boundaries, timestamp precision (seconds vs milliseconds), relative vs absolute.

### Reference

- [Since RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md)
- [Nervos DAO](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0023-dao-deposit-withdraw/0023-dao-deposit-withdraw.md)

---

## Pattern 6: Witness Data Pattern

### Overview

Pass large or ephemeral data (proofs, aggregated signatures, oracle prices) in the transaction's `witnesses` array rather than in Cell `data`. The Script reads from `witnesses` and validates the payload without storing it on-chain.

### Applicable scenarios

- Oracle price feeds — price in witness, Script checks signature
- Zero-knowledge proof verification — proof in witness
- Batch signature aggregation — combined sig in witness
- Off-chain computed state proofs

### Transaction structure

```typescript
const tx = ccc.Transaction.from({
  inputs: [{ previousOutput: cellToAuthorize }],
  outputs: [{ lock: ownerLock, capacity: ... }],
  // No outputsData needed for the witness payload
});

// Place the payload in the witness at the same index as the input
tx.witnesses[0] = WitnessArgs.encode({
  lock: signatureBytes,      // for Lock Script
  inputType: proofBytes,     // for Type Script (input side)
  outputType: undefined,     // for Type Script (output side)
});
```

### Script usage

```rust
// In the Script, read from witnesses:
let witness_args = load_witness_args(0, Source::GroupInput)?;
let proof = witness_args.input_type().to_opt().ok_or(Error::MissingProof)?;
// Validate proof...
```

### Notes

- Witness data is **not** stored in Cell state — it is only available during transaction execution.
- The witness at index `i` corresponds to the input at index `i` in the Script group.
- `WitnessArgs` layout: `lock` (for Lock Script), `input_type` (for Type Script on input side), `output_type` (for Type Script on output side).
- Witness data can be large (e.g., ZK proofs) without affecting Cell storage costs.

### Reference

- [WitnessArgs Molecule schema](https://github.com/nervosnetwork/ckb/blob/develop/util/types/schemas/blockchain.mol)
- [ckb-std load_witness_args](https://docs.rs/ckb-std/latest/ckb_std/high_level/fn.load_witness_args.html)

---

## Pattern 7: Dep Cell Reference

### Overview

Store shared, read-only configuration or code in a Cell referenced via `cell_deps` rather than carrying it in every transaction's inputs or outputs. Scripts read from `cell_deps` without consuming (destroying) the referenced Cell.

### Applicable scenarios

- Shared Script code Cell (the canonical use of `cell_deps`)
- Global protocol parameters (fee rates, allowed assets, admin list)
- Oracle price snapshot — publish price in a Cell; contracts reference it as a dep
- Shared Molecule type definitions

### Transaction structure

```typescript
const tx = ccc.Transaction.from({
  inputs: [...],
  outputs: [...],
});

// Reference the config Cell as a dep — it is NOT consumed
tx.addCellDeps({
  outPoint: { txHash: configCellTxHash, index: 0 },
  depType: "code", // "code" for Script binary, "depGroup" for a group of deps
});
```

### Script usage

```rust
// Load config from cell_deps by searching for a known type_hash
// Use load_cell_data / load_cell_by_field to scan deps
let config_data = load_cell_data(0, Source::CellDep)?;
// Parse and use config...
```

### Notes

- `cell_deps` are read-only — the referenced Cells are not consumed or modified.
- Use `depType: "depGroup"` to bundle multiple deps into one reference (reduces transaction size).
- The oracle or config publisher can update the dep Cell by consuming and recreating it; consumers always reference the latest live version.
- Ensure your Script verifies the `code_hash` or `type_hash` of dep Cells it reads from — an attacker could substitute a malicious dep.

### Reference

- [Transaction structure — cell_deps](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)

---

## Choosing the Right Pattern

```
Your scenario                              → Recommended pattern
───────────────────────────────────────────────────────────────
One global state Cell (token info, config) → Pattern 1 (Single-Cell State)
Per-user balance or asset ownership        → Pattern 2 (Multi-Cell Collection)
Need to upgrade Script later               → Pattern 3 (Type ID Singleton)
Lock and data rules are independent        → Pattern 4 (Lock + Type Layering)
Time-delayed spending (vesting, DAO)       → Pattern 5 (Since Time-Lock)
Large proof / oracle data in transaction   → Pattern 6 (Witness Data Pattern)
Shared config read by many contracts       → Pattern 7 (Dep Cell Reference)
```

Most real-world contracts combine multiple patterns — for example, a token uses Pattern 2 (Multi-Cell Collection) + Pattern 4 (Lock + Type Layering) + Pattern 3 (Type ID) for the Info Cell.
