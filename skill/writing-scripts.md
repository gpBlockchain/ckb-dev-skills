# Writing CKB Scripts

## Positioning (opinionated)

CKB-VM runs RISC-V binaries, so many languages are possible. For on-chain Scripts that protect real assets:

- Prefer **Rust** first (best ergonomics + `ckb-std` ecosystem).
- Use **C** when you are extremely sensitive to binary size/cycle cost.
- Other languages exist (JS/Lua/interpreters, etc), but are generally **not recommended** for production on-chain validation.

## Ecosystem scripts (reuse first)

Before writing a new lock/type script from scratch, check what already exists in the ecosystem.

- Official overview: https://docs.nervos.org/docs/ecosystem-scripts/introduction
- This repo's curated notes: [ecosystem-scripts.md](ecosystem-scripts.md)

Guidance:

- Prefer reusing well-known scripts (or composing via proxy locks) over inventing new security-critical primitives.
- Treat third-party scripts as security-sensitive; many are not audited. Do not use in production unless you understand and test them.
- When reusing scripts, be explicit about `cell_deps` (code/data cells) and the exact `code_hash` + `hash_type` expected on the target network.

## Rust (recommended)

Write CKB on-chain Scripts (smart contracts) in Rust with `ckb-std`.

### Project scaffolding (recommended)

For real projects, start from [ckb-script-templates](https://github.com/cryptape/ckb-script-templates) via `cargo-generate`.

```bash
cargo install cargo-generate

# Create a workspace (recommended for multiple contracts)
cargo generate gh:cryptape/ckb-script-templates workspace --name my-ckb-contracts
cd my-ckb-contracts

# Generate a new contract crate inside the workspace
make generate CRATE=my-lock

# Build + test
make build
make test
```

Notes:

- The templates expect you to have Rust + the `riscv64imac-unknown-none-elf` target and a recent clang installed.
- If you only need a single contract crate, the repo also provides a `standalone-contract` template.

### Testing (recommended)

Write contract tests in Rust using `ckb-testtool` (local simulation + `ckb-vm` verification). The official guide is: https://docs.nervos.org/docs/script/rust/rust-test

Practical notes:

- Build the tx, then call `context.complete_tx(tx)` to fill required `cell_deps` before signing/verifying.
- If your contract needs extra deps (e.g. signature data cells), `deploy_cell` does not auto-add them; add as `cell_deps` explicitly.
- Use `context.dump_tx()` to export a deterministic JSON tx for `ckb-debugger`.

### Hello World

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

### Reading Script Args

```rust
pub fn program_entry() -> i8 {
    let script = ckb_std::high_level::load_script().unwrap();
    let args = script.args().raw_data().to_vec();
    ckb_std::debug!("Args Len: {}", args.len());
    ckb_std::debug!("Args Data: {:02x?}", args);
    0
}
```

### Common `ckb-std` APIs

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

`Source::GroupInput` / `Source::GroupOutput` are especially common; they iterate only cells in the current script group.

### Common patterns

Lock script signature verification (sketch):

```rust
use ckb_std::high_level::{load_script, load_witness_args, Source};
use ckb_std::ckb_types::prelude::*;

pub fn program_entry() -> i8 {
    let script = load_script().unwrap();
    let _args = script.args().raw_data();

    let witness_args = load_witness_args(0, Source::GroupInput).unwrap();
    let _signature = witness_args.lock().to_opt().unwrap().raw_data();

    // Verify signature against args (public key hash)
    0
}
```

Type script token non-inflation check:

```rust
use ckb_std::high_level::{load_cell_data, QueryIter, Source};

pub fn program_entry() -> i8 {
    let input_amount: u128 = QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|data| u128::from_le_bytes(data[0..16].try_into().unwrap()))
        .sum();

    let output_amount: u128 = QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|data| u128::from_le_bytes(data[0..16].try_into().unwrap()))
        .sum();

    if output_amount > input_amount {
        return -1;
    }

    0
}
```

### Error handling

```rust
use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidSignature = 5,
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

### Tips

- Use `Source::GroupInput` / `Source::GroupOutput` for most validations.
- Handle `SysError::IndexOutOfBound` when iterating cells; it indicates end-of-list.
- Keep binaries small; fewer bytes usually means fewer cycles.
- `ckb_std::debug!()` output appears in debugger/tests, not on mainnet.

### References

- [Rust API Introduction](https://docs.nervos.org/docs/script/rust/rust-api-introduction)
- [Rust API Syscalls](https://docs.nervos.org/docs/script/rust/rust-api-syscalls)
- [Rust API Error](https://docs.nervos.org/docs/script/rust/rust-api-error)
- [Rust Examples](https://docs.nervos.org/docs/script/rust/rust-example-minimal-script)

## C (recommended for size/cycles)

C is widely used in production scripts (e.g. sUDT/xUDT). Prefer C when you need the smallest binaries and tightest cycle control.

### Toolchain

- GCC with RISC-V target or Clang with RISC-V backend
- Standard library: [ckb-c-stdlib](https://github.com/nervosnetwork/ckb-c-stdlib)

### Minimal script

```c
#include "ckb_syscalls.h"

int main() {
    unsigned char script[1024];
    uint64_t len = 1024;
    int ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS) {
        return ret;
    }
    return CKB_SUCCESS;
}
```

### Reference projects

- [ckb-production-scripts (C)](https://github.com/nervosnetwork/ckb-production-scripts/tree/master/c)
- [ckb-c-stdlib](https://github.com/nervosnetwork/ckb-c-stdlib)

## Other languages (supported, generally not recommended)

CKB-VM can run almost anything that can ultimately execute as RISC-V, but the practical approach is usually "VM on VM" (deploy an interpreter as a cell dep, store your code in cell data). This is great for prototyping and education, but typically costs more cycles and adds operational complexity.

### JavaScript (interpreter)

JavaScript can run inside an on-chain JS interpreter: [ckb-js-vm](https://github.com/nervosnetwork/ckb-js-vm) (QuickJS).

```bash
npm install @ckb-js-std/bindings @ckb-js-std/core
```

- Docs: [JS Quick Start](https://docs.nervos.org/docs/script/js/js-quick-start)
- Docs: [JS VM](https://docs.nervos.org/docs/script/js/js-vm)

### Lua (interpreter)

Lightweight scripting via [ckb-lua-vm](https://github.com/nervosnetwork/ckb-lua-vm).

### More options

Examples of what people have built:

| Language/VM | Method | Reference |
|------------|--------|-----------|
| Go | Compile to RISC-V | TinyGo can work for some use cases |
| Ruby | mruby interpreter on-chain | [mruby](https://github.com/mruby/mruby) |
| Python | interpreter on-chain | MicroPython-style approach |
| EVM | EVM runtime on-chain | [Godwoken Polyjuice](https://github.com/godwokenrises/godwoken/tree/develop/gwos-evm) |
| Bitcoin Script | Bitcoin VM on-chain | [ckb-bitcoin-vm](https://github.com/xxuejie/ckb-bitcoin-vm) |
| Cell-Script | dedicated CKB language | [Cell-Script](https://github.com/cell-labs/cell-script) |

### Practical guidance

- If a script guards value, default to Rust/C unless you have a strong reason.
- If you do use interpreters, measure cycle consumption early and often.
- Keep the deployment model simple (clear `cell_deps`, deterministic code cells).

### References

- [Program Languages for Script](https://docs.nervos.org/docs/script/program-language-for-script)
