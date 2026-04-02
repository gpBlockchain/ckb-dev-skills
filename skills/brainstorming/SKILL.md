---
name: ckb-brainstorming
description: Interactive brainstorming and Q&A skill for CKB project creation. Asks questions to understand user intent before generating any code.
user-invocable: true
---

# CKB Project Brainstorming

## When to activate

Activate this skill when the user wants to:

- Create a new CKB project
- Start building something on CKB without clear requirements
- Explore what they can build on CKB
- Design a new CKB Script, DApp, or Fiber integration

## Core principle

**Never jump into writing code.** First understand what the user is really trying to build through a structured Q&A conversation.

## Brainstorming workflow

### Phase 1: Project Discovery

Ask the user these questions one at a time. Wait for their response before asking the next question. Adapt based on their answers.

**Question 1 — Project Type**

> What kind of CKB project do you want to build?
>
> 1. 🔗 **On-chain Script (Smart Contract)** — A Lock Script or Type Script running on CKB-VM
> 2. 🌐 **DApp (Decentralized Application)** — A frontend application interacting with CKB
> 3. ⚡ **Fiber Network Integration** — Payment channels, invoices, off-chain payments
> 4. 🔄 **Full-Stack** — Contract + Frontend together
> 5. 🤔 **Not sure yet** — Let me describe what I want to achieve

**Question 2 — Goal Clarification**

Based on their answer, ask a follow-up:

- If Script: "What should this Script control? (e.g., multisig, token, custom lock logic, data validation)"
- If DApp: "What should the DApp do? (e.g., token transfer, NFT minting, wallet dashboard, DEX)"
- If Fiber: "What payment scenario? (e.g., micropayments, cross-chain, stablecoin, invoicing)"
- If Full-Stack: "What's the end-to-end user story? Walk me through what a user would do."
- If Not sure: "Describe what you want to achieve in plain language. What problem are you solving?"

**Question 3 — Technical Preferences**

> A few technical choices:
>
> - **Script Language**: Rust (recommended) / C / JavaScript?
> - **DApp Framework**: React / Vue / vanilla JS?
> - **Network Target**: Devnet (local) / Testnet / Mainnet?

**Question 4 — Experience Level**

> How familiar are you with CKB development?
>
> 1. 🆕 **Brand new** — First time with CKB
> 2. 📖 **Read the docs** — Understand Cell Model conceptually
> 3. 🛠️ **Built something** — Have written a Script or DApp before
> 4. 🧑‍💻 **Experienced** — Comfortable with CKB transaction building

### Phase 2: Design Summary

After gathering answers, present a concise design summary:

```
📋 Project Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Type:        [Script / DApp / Fiber / Full-Stack]
Goal:        [One-sentence description]
Language:    [Rust / TypeScript / etc.]
Framework:   [CCC SDK / React / etc.]
Network:     [Devnet / Testnet / Mainnet]
Experience:  [Level]

📦 What will be generated:
- [List of files/components]

🔧 Tools & Dependencies:
- [List of tools needed]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Ask: "Does this look right? Should I adjust anything before we start?"

### Phase 3: Implementation Plan

Once the user approves the design:

**Important**: If the project involves on-chain Scripts (Script or Full-Stack type), do NOT proceed directly to code generation. Route to the contract-design skill first:

1. Load `skills/contract-design/SKILL.md`
2. Complete all 4 design phases with user confirmation
3. Then hand off to the Contract Agent for implementation

For all other project types (DApp, Fiber):

1. Create a step-by-step implementation plan
2. Each step should be small and verifiable
3. Route to the appropriate specialist Agent for execution

## Agent routing after brainstorming

After brainstorming is complete, hand off to the Team Lead for proper agent dispatch:

- Script projects → 📝 ckb-contract Agent
- DApp projects → 🌐 ckb-dapp Agent
- Fiber projects → ⚡ ckb-fiber Agent
- Full-Stack → coordinated multi-agent workflow
- Conceptual questions → 🔗 ckb-core Agent
