# Script

## Overview

A Script in CKB is a binary executable that runs on-chain in the CKB-VM (a RISC-V based virtual machine). Scripts are CKB's equivalent of smart contracts. They are Turing-complete and can perform arbitrary logic to guard and protect on-chain assets.

## Script Structure

```rust
pub struct Script {
    pub code_hash: H256,         // Identifies the Script code
    pub hash_type: ScriptHashType, // How to locate the Script code
    pub args: JsonBytes,          // Parameters passed to the Script
}
```

### Fields Explained

| Field       | Description                                                   |
| ----------- | ------------------------------------------------------------- |
| `code_hash` | Identifies which Script code to load and execute              |
| `hash_type` | Defines how to interpret `code_hash` when locating code       |
| `args`      | Custom arguments passed to the Script (e.g., public key hash) |

### hash_type Values

| Value                    | Name      | Description                                  |
| ------------------------ | --------- | -------------------------------------------- |
| `data`, `data1`, `data2` | Data Hash | Match the hash of the Script binary directly |
| `type`                   | Type Hash | Match the hash of a Cell's Type Script       |

## Two Types of Scripts

### Lock Script (Required)

- Controls **ownership and access** to a Cell.
- Executes only on **input** Cells (not output Cells).
- If the Lock Script returns non-zero, the Cell cannot be consumed.
- Common use: signature verification (e.g., `secp256k1_blake160_sighash_all`).

### Type Script (Optional)

- Controls **how a Cell can be used** in a transaction.
- Executes on both **input** and **output** Cells.
- Common use: enforcing token rules (e.g., UDT issuance/transfer logic).

## Script Execution Rules

In a transaction:

- Input Cells' **Lock Scripts** are executed.
- Input Cells' **Type Scripts** are executed (if present).
- Output Cells' **Type Scripts** are executed (if present).
- Output Cells' **Lock Scripts** are **NOT** executed.

Return code `0` = success. Any non-zero = failure (transaction rejected).

## Default Lock Script: secp256k1_blake160_sighash_all

The most common Lock Script on CKB:

1. Extracts the public key from the transaction witness.
2. Hashes the public key with Blake2b to get a Blake160 hash.
3. Compares with the hash stored in `args`.
4. Verifies the secp256k1 signature.

```
code_hash: 0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8
hash_type: type
args: <20-byte blake160 hash of public key>
```

## AI Dev Tips

- Scripts share code via `code_hash` + `hash_type`; they differ by `args`. Multiple users share the same Lock Script code but each has a unique `args` (their public key hash).
- Use `type` hash_type for upgradable scripts (via Type ID pattern); use `data`/`data1`/`data2` for immutable scripts.
- When building transactions, always include the Cell containing the Script code in `cell_deps`.

## References

- [Intro to Script](https://docs.nervos.org/docs/script/intro-to-script)
- [Script Structure](https://docs.nervos.org/docs/tech-explanation/script)
- [Lock Script vs Type Script](https://docs.nervos.org/docs/tech-explanation/lock-type-diff)
- [Data Hash vs Type Hash](https://docs.nervos.org/docs/tech-explanation/data-type-diff)
