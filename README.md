# CKB Development Skill for AI

A comprehensive Vibe coding skill and best practices for Nervos CKB development (CKB2023 MIRANA, to date of March 2026).

## Overview

This skill provides a **Team Lead + 4 Specialist Agent** architecture for deep CKB development knowledge:

```
                    ┌─────────────────────────────┐
         You ──────▶│  🧠 Team Lead (ckb-dev-lead) │
                    │   Intent → Route → Review    │
                    └──────────┬──────────────────┘
                               │
            ┌──────────┬───────┴──────┬──────────────┐
            ▼          ▼              ▼              ▼
     ┌────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
     │ 🔗 Core    │ │ 📝 Contract│ │ 🌐 DApp  │ │ ⚡ Fiber │
     │ Agent      │ │ Agent     │ │ Agent    │ │ Agent    │
     │            │ │           │ │          │ │          │
     │ Cell Model │ │ Rust      │ │ CCC SDK  │ │ Payment  │
     │ Transaction│ │ Testing   │ │ React    │ │ Channels │
     │ CKB-VM     │ │ Deploy    │ │ Wallets  │ │ Swaps    │
     └────────────┘ └──────────┘ └──────────┘ └──────────┘
```

### Agent Specializations

- **🔗 Core Agent** — Cell Model, Script structure, Transaction structure, CKB-VM. For understanding CKB fundamentals.
- **📐 Contract Design Skill** — Interactive 4-phase design workflow (Cell modeling, permissions, state transitions, security review). Run before writing contract code.
- **📝 Contract Agent** — Rust environment, writing Scripts, testing, debugging, security, deployment, token standards. For Rust smart contract developers.
- **🌐 DApp Agent** — CCC SDK, React wallet integration, transaction building, frontend scaffolding. For TypeScript/frontend developers.
- **⚡ Fiber Agent** — Fiber node operations, payment channels, invoices, stablecoin payments, cross-chain interop. For payment/L2 developers.

### Coverage

- **Cell Model**: Generalized UTXO model — Cells, capacity, Live/Dead state
- **Scripts**: On-chain programs (Lock Script, Type Script) running on CKB-VM (RISC-V)
- **Script Language**: Rust with `ckb-std` (default), C with `ckb-c-stdlib`, JS via `ckb-js-vm`
- **DApp SDK**: CCC (`@ckb-ccc/*`) for transaction building, signing, wallet connection
- **Tokens**: sUDT, xUDT, Spore DOB, RGB++ protocol
- **Testing**: `ckb-testtool` (Rust) + `ckb-debugger` (CLI, GDB)
- **Deployment**: OffCKB (local Devnet), Type ID for upgradable Scripts
- **Security**: Capacity validation, Cell counting, reinitialization prevention
- **Fiber Network**: Payment channels, off-chain payments, cross-chain Lightning interop

## Installation

### One-Liner Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/gpBlockchain/ckb-dev-skills/main/install.sh | bash
```

This automatically clones the repository and installs the skill to `~/.claude/skills/ckb-dev`.

### Claude Code (via Plugin Marketplace)

In Claude Code, register the marketplace first:

```
/plugin marketplace add gpBlockchain/ckb-dev-skills
```

Then install the plugin:

```
/plugin install ckb-dev-skills@ckb-dev-skills-marketplace
```

### Cursor

In Cursor Agent chat:

```
/add-plugin ckb-dev-skills
```

### Codex

Tell Codex:

```
Fetch and follow instructions from https://raw.githubusercontent.com/gpBlockchain/ckb-dev-skills/main/.codex/INSTALL.md
```

### OpenCode

Tell OpenCode:

```
Fetch and follow instructions from https://raw.githubusercontent.com/gpBlockchain/ckb-dev-skills/main/.opencode/INSTALL.md
```

### Gemini CLI

```bash
gemini extensions install https://github.com/gpBlockchain/ckb-dev-skills
```

### Manual Install

```bash
git clone https://github.com/gpBlockchain/ckb-dev-skills
cd ckb-dev-skills
./install.sh
```

### Install to Project

```bash
./install.sh --project
```

### Update

```bash
# Re-run the one-liner, or:
./install.sh --update
```

### Uninstall

```bash
./install.sh --uninstall
```

## Skill Structure

```
agents/
├── ckb-dev-lead/
│   └── SKILL.md                      # 🧠 Team Lead — main entry point
│
├── ckb-core/
│   ├── SKILL.md                      # 🔗 Core Agent
│   ├── cell-model.md                 # Cell Model basics
│   ├── script.md                     # Script structure & types
│   ├── transaction.md                # Transaction structure
│   └── ckb-vm.md                     # CKB-VM, cycles, syscalls
│
├── ckb-contract/
│   ├── SKILL.md                      # 📝 Contract Agent
│   ├── rust-setup.md                 # Rust environment setup
│   ├── writing-scripts.md            # Writing Scripts (authoritative links)
│   ├── testing.md                    # Testing Scripts
│   ├── debugging.md                  # Debugging Scripts
│   ├── deployment.md                 # Deployment & tools
│   ├── security.md                   # Security checklist
│   ├── ecosystem-scripts.md          # System/ecosystem Scripts
│   ├── token-standards.md            # sUDT, xUDT, RGB++
│   ├── transaction-patterns.md       # Transaction composition patterns
│   └── design-patterns.md            # CKB contract design patterns
│
├── ckb-dapp/
│   ├── SKILL.md                      # 🌐 DApp Agent
│   ├── ccc-sdk.md                    # CCC SDK for DApp development
│   └── wallet-integration.md         # Wallet connection & ecosystem
│
└── ckb-fiber/
    ├── SKILL.md                      # ⚡ Fiber Agent
    └── fiber-network.md              # Fiber Network (payment channels)

skills/
├── brainstorming/
│   └── SKILL.md                      # 🧠 Interactive project brainstorming
└── contract-design/
    └── SKILL.md                      # 📐 Interactive contract design (4-phase)

.claude/
└── agents/
    ├── ckb-dev-lead.md               # @ckb-dev-lead — Talk to Team Lead
    ├── brainstorm.md                  # @brainstorm — Q&A project creation
    ├── contract-design.md             # @contract-design — 4-phase design session
    ├── ckb-core.md                    # @ckb-core — Talk to Core Agent
    ├── ckb-contract.md                # @ckb-contract — Talk to Contract Agent
    ├── ckb-dapp.md                    # @ckb-dapp — Talk to DApp Agent
    └── ckb-fiber.md                   # @ckb-fiber — Talk to Fiber Agent

shared/
└── resources.md                      # Curated reference links
```

## Usage

Once installed, Claude Code will automatically use this skill when you ask about CKB development.

### Custom Agents

Talk directly to specific agents or start a brainstorming session:

| Agent              | Description                                                        |
| ------------------ | ------------------------------------------------------------------ |
| `@ckb-dev-lead`    | Talk to the Team Lead (routes to the right specialist agent)       |
| `@brainstorm`      | Interactive Q&A to design a new CKB project                        |
| `@contract-design` | Interactive 4-phase contract design with step-by-step confirmation |
| `@ckb-core`        | Talk to the Core Agent (Cell Model, transactions)                  |
| `@ckb-contract`    | Talk to the Contract Agent (Rust Scripts, testing)                 |
| `@ckb-dapp`        | Talk to the DApp Agent (CCC SDK, React, wallets)                   |
| `@ckb-fiber`       | Talk to the Fiber Agent (payment channels, Lightning)              |

### Creating a New Project

Use `@brainstorm` to start an interactive session. The agent will ask you questions about:

1. **Project type** — Script, DApp, Fiber, or Full-Stack?
2. **Goal** — What are you building?
3. **Technical preferences** — Language, framework, network target
4. **Experience level** — How familiar are you with CKB?

Then it generates a design summary for your approval before writing any code.

### Designing a Contract

Before writing any contract code, use `@contract-design` to walk through the 4-phase design process:

1. **Phase 1 — State Modeling**: Define your Cell types, data fields, and capacity budgets
2. **Phase 2 — Roles & Permissions**: Choose Lock Scripts, define who can create/update/destroy each Cell
3. **Phase 3 — State Transitions**: List all operations and their validation rules
4. **Phase 4 — Security Pre-Review**: Check the design against the security checklist before writing code

Each phase pauses for your confirmation before proceeding. Once all 4 phases are approved, the skill produces a **Contract Design Document** and hands off to the Contract Agent for implementation.

### Automatic Skill Activation

The skill also activates automatically when you ask about:

- CKB on-chain Script (smart contract) development
- Cell Model and UTXO-style state management
- Transaction building, signing, and sending on CKB
- DApp development with CCC SDK (TypeScript/JavaScript)
- Token creation and management (sUDT, xUDT, Spore DOB, RGB++)
- Wallet integration for CKB
- Testing and debugging CKB Scripts
- Deploying Scripts to Devnet/Testnet/Mainnet
- CKB-VM, cycles, and performance optimization
- Molecule serialization format
- Fiber Network payment channels, invoices, and cross-chain swaps

### Example Prompts

```
"Help me set up a new CKB Script project in Rust"
"Create a Lock Script that requires 2-of-3 multisig"
"Build a transaction that transfers CKB using CCC SDK"
"Write tests for my Type Script using ckb-testtool"
"How do I deploy a Script to CKB Testnet?"
"Create a simple UDT token with mint and transfer"
"Debug my Script — it's returning error code 5"
"What's the minimum capacity for a Cell with 32 bytes of data?"
"Help me integrate CKB wallet connection in my React app"
"Review this Script for security issues"
"How do I run a Fiber node and open a payment channel?"
"Send a payment over Fiber Network using invoices"
"Set up a two-node Fiber testnet for local development"
```

## Stack Decisions

This skill encodes opinionated best practices:

| Layer               | Default Choice         | Alternative                            |
| ------------------- | ---------------------- | -------------------------------------- |
| Script Language     | Rust + `ckb-std`       | C (`ckb-c-stdlib`), JS (`ckb-js-vm`)   |
| DApp SDK            | CCC (`@ckb-ccc/shell`) | CCC React (`@ckb-ccc/connector-react`) |
| Project Scaffolding | `ckb-script-templates` | Manual setup                           |
| Unit Testing        | `ckb-testtool`         | `ckb-debugger` CLI                     |
| Debugging           | `ckb-debugger` + GDB   | Debug print via `ckb_debug!`           |
| Local Development   | OffCKB                 | Manual CKB node                        |
| Deployment          | Type ID (upgradable)   | Direct data deployment                 |
| Serialization       | Molecule               | —                                      |
| Payment Channels    | Fiber Network (fnn)    | —                                      |

## Content Sources

This skill incorporates best practices from:

- [Nervos CKB Documentation](https://docs.nervos.org/) — Official documentation
- [CCC SDK](https://github.com/ckb-devrel/ccc) — Primary JS/TS SDK
- [ckb-std](https://github.com/nervosnetwork/ckb-std) — Rust standard library for Scripts
- [CKB RFCs](https://github.com/nervosnetwork/rfcs) — Protocol specifications
- [CKB Academy](https://academy.ckb.dev/) — Learning platform
- [Fiber Network](https://github.com/nervosnetwork/fiber) — Payment channel network
- [Fiber Docs](https://docs.fiber.world) — Fiber Network documentation

## Progressive Disclosure

The skill uses Claude Code's progressive disclosure pattern. The main `SKILL.md` provides core guidance and operating procedures. Claude reads the specialized markdown files only when needed for specific tasks, keeping context usage efficient.

## Quality Checks

The repository includes a docs-focused CI workflow that checks:

- Markdown links with Lychee
- Spelling with CSpell
- Markdown formatting with Prettier

You can run the same checks locally with:

```bash
make docs-check
```

Useful shortcuts:

- `make docs-format` — rewrite Markdown files with Prettier
- `make docs-format-check` — verify Markdown formatting only
- `make docs-spell` — run spelling checks only
- `make docs-links` — run link checks only

## Contributing

Contributions are welcome! Please ensure any updates reflect current CKB ecosystem best practices.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT License
