# Script Development in Other Languages

## Overview

While Rust is recommended, CKB-VM supports any language that can target RISC-V. This skill covers alternative language options.

## C Language

C is used in many core CKB production Scripts (sUDT, xUDT).

### Toolchain
- GCC with RISC-V target or Clang with RISC-V backend.
- Standard library: [ckb-c-stdlib](https://github.com/nervosnetwork/ckb-c-stdlib)

### Example: Minimal C Script

```c
#include "ckb_syscalls.h"

int main() {
    // Load current script
    unsigned char script[1024];
    uint64_t len = 1024;
    int ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS) {
        return ret;
    }
    return CKB_SUCCESS;
}
```

### Reference Projects
- [ckb-production-scripts](https://github.com/nervosnetwork/ckb-production-scripts/tree/master/c) (sUDT, xUDT implementations)
- [ckb-c-stdlib](https://github.com/nervosnetwork/ckb-c-stdlib)

## JavaScript

JavaScript scripts run inside an on-chain JS interpreter ([ckb-js-vm](https://github.com/nervosnetwork/ckb-js-vm)) built on QuickJS.

### Quick Start
```bash
# Install dependencies
npm install @ckb-js-std/bindings @ckb-js-std/core
```

### How It Works
1. `ckb-js-vm` (the QuickJS interpreter) is deployed as a Cell on-chain.
2. Your JS script is stored in another Cell's data.
3. The Script's `code_hash` points to `ckb-js-vm`, and `args` references your JS code Cell.

### Built-in Crypto
`ckb-js-vm` includes C implementations of:
- Secp256K1
- Blake2b
- Sparse Merkle Tree (SMT)

### Trade-offs
- **Pro**: Familiar language, fast prototyping, large ecosystem.
- **Con**: Higher cycle consumption due to interpretation overhead.

### Packages
- [@ckb-js-std/bindings](https://www.npmjs.com/package/@ckb-js-std/bindings)
- [@ckb-js-std/core](https://www.npmjs.com/package/@ckb-js-std/core)

### Reference
- [JS Quick Start](https://docs.nervos.org/docs/script/js/js-quick-start)
- [JS VM](https://docs.nervos.org/docs/script/js/js-vm)

## Lua

Lightweight scripting via [ckb-lua-vm](https://github.com/nervosnetwork/ckb-lua-vm).

Same deployment model as JavaScript -- interpreter on-chain, Lua code in Cell data.

- **Pro**: Minimal, lightweight, suitable for constrained environments.
- **Con**: Smaller ecosystem, interpretation overhead.

## Other Languages

Since CKB-VM is essentially a RISC-V computer, you can run virtually any language:

| Language | Method | Reference |
|----------|--------|-----------|
| Go | Compile to RISC-V | Possible with TinyGo |
| Ruby | Deploy mruby interpreter on-chain | [mruby](https://github.com/mruby/mruby) |
| Python | Deploy interpreter on-chain | Possible with MicroPython |
| Solidity/EVM | Deploy EVM on-chain | [Godwoken-Polyjuice](https://github.com/godwokenrises/godwoken/tree/develop/gwos-evm) |
| Bitcoin Script | Deploy Bitcoin VM on-chain | [ckb-bitcoin-vm](https://github.com/xxuejie/ckb-bitcoin-vm) |
| Cell-Script | Dedicated CKB language | [Cell-Script](https://github.com/cell-labs/cell-script) (early stage) |

## AI Dev Tips

- For production Scripts handling real assets, prefer Rust or C for performance and safety.
- JavaScript is excellent for prototyping, educational demos, and quick tooling.
- When using interpreted languages, factor in the cycle cost of the interpreter itself.
- The "VM on VM" approach works but measure cycle consumption for your specific use case.

## References

- [Program Languages for Script](https://docs.nervos.org/docs/script/program-language-for-script)
- [JS Quick Start](https://docs.nervos.org/docs/script/js/js-quick-start)
- [ckb-js-vm](https://github.com/nervosnetwork/ckb-js-vm)
- [ckb-lua-vm](https://github.com/nervosnetwork/ckb-lua-vm)
