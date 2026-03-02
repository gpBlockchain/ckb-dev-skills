# Writing CKB Scripts in Rust

## Overview

This skill covers writing CKB on-chain Scripts (smart contracts) using Rust with the `ckb-std` library.

## Hello World Script

The simplest CKB Script:

```rust
#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

pub fn program_entry() -> i8 {
    ckb_std::debug!("Hello World!");
    0
}
```

Run with `ckb-debugger`:
```bash
ckb-debugger --bin build/release/hello-world
# Output:
# Script log: Hello World!
# Run result: 0
# All cycles: 7366(7.2K)
```

## Reading Script Args

Scripts commonly need to read their own `args` field:

```rust
pub fn program_entry() -> i8 {
    let script = ckb_std::high_level::load_script().unwrap();
    let args = script.args().raw_data().to_vec();
    ckb_std::debug!("Args Len: {}", args.len());
    ckb_std::debug!("Args Data: {:02x?}", args);
    0
}
```

## Common ckb-std APIs

### High-Level APIs (`ckb_std::high_level`)

```rust
use ckb_std::high_level::*;

// Load the current executing Script
let script = load_script()?;

// Load transaction data
let tx = load_transaction()?;

// Load Cell data by index and source
let data = load_cell_data(index, Source::Input)?;

// Load Cell capacity
let capacity = load_cell_capacity(index, Source::Output)?;

// Load Cell lock script
let lock = load_cell_lock(index, Source::Input)?;

// Load Cell type script
let type_script = load_cell_type(index, Source::Input)?;

// Load witness
let witness = load_witness_args(index, Source::Input)?;

// Load input since field
let since = load_input_since(index, Source::Input)?;

// Load Cell field
let lock_hash = load_cell_lock_hash(index, Source::Input)?;

// Query Cells by lock/type script
let indices = QueryIter::new(load_cell, Source::Input)
    .enumerate()
    .filter(|(_, cell)| /* condition */)
    .collect::<Vec<_>>();
```

### Source Enum

```rust
pub enum Source {
    Input,      // Input Cells
    Output,     // Output Cells
    CellDep,    // Cell dependencies
    HeaderDep,  // Header dependencies
    GroupInput,  // Cells in the same Script group (inputs)
    GroupOutput, // Cells in the same Script group (outputs)
}
```

`GroupInput` and `GroupOutput` are particularly useful -- they filter only Cells that share the same Script as the currently executing one.

## Common Script Patterns

### Lock Script: Signature Verification

```rust
pub fn program_entry() -> i8 {
    let script = load_script().unwrap();
    let args: Bytes = script.args().unpack();

    // Load witness for signature
    let witness_args = load_witness_args(0, Source::GroupInput).unwrap();
    let signature = witness_args.lock().to_opt().unwrap().raw_data();

    // Verify signature against args (public key hash)
    // ... signature verification logic ...

    0
}
```

### Type Script: Token (UDT) Validation

```rust
pub fn program_entry() -> i8 {
    // Sum input token amounts
    let input_amount: u128 = QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|data| u128::from_le_bytes(data[0..16].try_into().unwrap()))
        .sum();

    // Sum output token amounts
    let output_amount: u128 = QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|data| u128::from_le_bytes(data[0..16].try_into().unwrap()))
        .sum();

    // Ensure tokens are not created from nothing
    if output_amount > input_amount {
        return -1; // Error: token inflation
    }

    0
}
```

### Check if Script is Lock or Type

```rust
use ckb_std::high_level::load_script_hash;
use ckb_std::high_level::load_cell_lock_hash;
use ckb_std::high_level::load_cell_type_hash;

let script_hash = load_script_hash()?;

// If our hash matches an input's lock hash, we're running as a Lock Script
let is_lock = load_cell_lock_hash(0, Source::GroupInput)
    .map(|h| h == script_hash)
    .unwrap_or(false);
```

## Error Handling

Define custom error codes:

```rust
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    // Custom errors
    InvalidSignature = 5,
    AmountOverflow = 6,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Error::IndexOutOfBound,
            SysError::ItemMissing => Error::ItemMissing,
            SysError::LengthNotEnough(_) => Error::LengthNotEnough,
            SysError::Encoding => Error::Encoding,
            _ => Error::Encoding,
        }
    }
}
```

## AI Dev Tips

- Use `Source::GroupInput` / `Source::GroupOutput` to iterate only Cells sharing the current Script -- this is the most common pattern for both Lock and Type Scripts.
- Return `0` for success, negative `i8` for errors. Each error code should be documented.
- Always handle `SysError::IndexOutOfBound` when iterating Cells -- it signals the end of the list.
- `ckb_std::debug!()` output only appears in debugger/test, never in production.
- Keep Scripts minimal: smaller binary = fewer cycles = lower cost.

## References

- [Rust API Introduction](https://docs.nervos.org/docs/script/rust/rust-api-introduction)
- [Rust API Syscalls](https://docs.nervos.org/docs/script/rust/rust-api-syscalls)
- [Rust API Error](https://docs.nervos.org/docs/script/rust/rust-api-error)
- [Rust Examples](https://docs.nervos.org/docs/script/rust/rust-example-minimal-script)
