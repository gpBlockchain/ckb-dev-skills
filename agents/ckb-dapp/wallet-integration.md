# Wallet Integration

## Overview

CKB has a rich wallet ecosystem. CCC SDK provides unified wallet connection through its connector, supporting multiple wallets via a single integration point.

## CCC Wallet Connector (React)

```tsx
import { ccc } from "@ckb-ccc/connector-react";

function App() {
  return (
    <ccc.Provider>
      <YourApp />
    </ccc.Provider>
  );
}

function WalletButton() {
  const { open, wallet, signer } = ccc.useCcc();

  return (
    <button onClick={open}>{wallet ? wallet.name : "Connect Wallet"}</button>
  );
}
```

## Quick Start: create-ccc-app

```bash
# Bootstrap a new CCC app with wallet connection
npx create-ccc-app@latest my-ccc-app

# Or with pnpm
pnpm create ccc-app my-ccc-app
```

## CKB Wallet Ecosystem

CCC supports connecting to multiple CKB wallets via its connector:

| Wallet        | Platform                 | Lock Type           | Notes                                                                     |
| ------------- | ------------------------ | ------------------- | ------------------------------------------------------------------------- |
| Neuron        | Desktop (Win/Mac/Linux)  | secp256k1           | Full node wallet                                                          |
| CKBull        | Mobile (Android/iOS)     | secp256k1           | Mobile wallet                                                             |
| JoyID         | Web                      | secp256r1 (PassKey) | Uses WebAuthn/Passkeys via [JoyID Lock](https://docs.joyid.dev/guide/sdk) |
| Portal Wallet | Web                      | Omnilock            | Web wallet                                                                |
| Rei Wallet    | Browser Extension        | secp256k1           | [Docs](https://docs.reiwallet.io), [Demo](https://demo-app.reiwallet.io/) |
| SafePal       | Hardware/Mobile          | secp256k1           | Hardware wallet                                                           |
| Ledger        | Hardware                 | secp256k1           | Hardware wallet                                                           |
| imToken       | Mobile (Android/iOS)     | secp256k1           | Multi-chain                                                               |
| OneKey        | Desktop/Mobile/Extension | secp256k1           | Multi-chain                                                               |

## Omnilock: Universal Lock for Wallet Interop

[Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md) is a universal Lock Script that supports multiple authentication methods in a single Script:

- secp256k1 (CKB native, Bitcoin, Ethereum)
- secp256r1 (WebAuthn, passkeys)
- Multi-sig
- Time-lock and supply-lock extensions

This enables CKB DApps to accept users from any wallet ecosystem. CCC handles Omnilock integration automatically when connecting to supported wallets.

## JoyID Integration

[JoyID](https://docs.joyid.dev/guide/sdk) provides passwordless authentication using WebAuthn/Passkeys (secp256r1):

- Uses on-device biometric authentication (fingerprint, Face ID)
- CCC SDK supports JoyID connection out of the box via `@ckb-ccc/connector-react`

**Practical example**: [Philosopher's Stone - JoyID integration](https://github.com/SpectreMercury/PhilosopherStone/blob/main/src/utils/joyid.ts)

## AI Dev Tips

- CCC's wallet connector automatically handles different Lock Script types — you don't need to implement each wallet's auth logic manually.
- For `tsconfig.json`, set `moduleResolution` to `node16`, `nodenext`, or `bundler` to enable CCC's package exports.
- For React Server Components, add `"use client"` at the top of files using `ccc.Provider`.
- Use Omnilock when you need to support users from multiple wallet ecosystems in a single DApp.

## References

- [CCC Connector React](https://github.com/ckb-devrel/ccc)
- [Omnilock RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md)
- [Omnilock Introduction (Blog)](https://blog.cryptape.com/omnilock-a-universal-lock-that-powers-interoperability-1)
- [JoyID SDK](https://docs.joyid.dev/guide/sdk)
- [Rei Wallet Docs](https://docs.reiwallet.io)
