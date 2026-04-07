# Red Packet DApp Frontend

React frontend for the CKB Red Packet smart contracts. Built with Vite, TypeScript, and [CCC SDK](https://github.com/ckb-devrel/ccc).

## Setup

```bash
cd frontend-red-packet
npm install
```

Copy `.env.example` to `.env` and fill in contract deployment info:

```bash
cp .env.example .env
# Edit .env with actual values after deploying contracts
```

## Development

```bash
npm run dev
```

## Build

```bash
npm run build
```

## Configuration

The `.env` file contains contract deployment information:

| Variable | Description |
|----------|-------------|
| `VITE_CKB_NETWORK` | `testnet` or `mainnet` |
| `VITE_RED_PACKET_TYPE_CODE_HASH` | Type Script code_hash |
| `VITE_RED_PACKET_TYPE_HASH_TYPE` | Type Script hash_type |
| `VITE_RED_PACKET_LOCK_CODE_HASH` | Lock Script code_hash |
| `VITE_RED_PACKET_LOCK_HASH_TYPE` | Lock Script hash_type |
| `VITE_RED_PACKET_TYPE_TX_HASH` | Type Script cell dep tx_hash |
| `VITE_RED_PACKET_TYPE_TX_INDEX` | Type Script cell dep index |
| `VITE_RED_PACKET_LOCK_TX_HASH` | Lock Script cell dep tx_hash |
| `VITE_RED_PACKET_LOCK_TX_INDEX` | Lock Script cell dep index |

## Pages

- **Home** — Overview and how-it-works guide
- **Create** — Create a new red packet (set amount, shares, mode, expiry)
- **Claim** — Look up a red packet by Type ID and claim a share
- **My Packets** — View your created red packets and refund expired ones

## Stack

- [Vite](https://vite.dev/) + React 19 + TypeScript
- [CCC SDK](https://github.com/ckb-devrel/ccc) (`@ckb-ccc/connector-react`) for wallet connection and transaction building
- [React Router](https://reactrouter.com/) for client-side routing
