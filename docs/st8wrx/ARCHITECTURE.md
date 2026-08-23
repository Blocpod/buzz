# ST8WRX Architecture

## Goal

ST8WRX extends Buzz into an AI-native venture creation network. Buzz remains the collaboration, identity, project, agent, Git, workflow, and compute substrate. Bitcoin SV provides durable provenance, economic settlement, programmable agreements, and optional project-native assets.

The platform must let humans, agents, compute nodes, IP, and capital assemble around projects while preserving auditable evidence of who contributed what and enabling teams to launch, license, sell, or operate what they build.

## Non-goals

ST8WRX is not:

- a replacement for the Buzz relay/event model
- a blockchain chat system
- an on-chain Git implementation for day-to-day development
- a DAO-first governance system
- a token launchpad with collaboration attached
- a system where AI unilaterally determines legal ownership
- a system that stores secrets, private prompts, credentials, or proprietary source on a public chain by default

## Architectural principle

Keep high-frequency collaboration off-chain. Put only economically meaningful state, durable proofs, settlements, selected public artifacts, and contract state on BSV.

```
Humans / Agents / Compute
          |
          v
        Buzz
          |
  Projects / Git / Workflows / Mesh / Audit
          |
          v
   ST8WRX Protocol
          |
  Contribution / Metering / Agreements / Market
          |
          v
         BSV
          |
 Provenance / Settlement / Assets / Contracts
```

## Existing Buzz capabilities reused

ST8WRX should extend, not duplicate:

- Nostr/secp256k1 identities for humans and agents
- community membership and access control
- projects and repositories
- Git events and signed approvals
- workflows and agent execution
- audit trail
- desktop/mobile clients
- Buzz Mesh shared compute
- project contribution/activity visualization where present

## New ST8WRX subsystems

### 1. Contribution Ledger

A project-scoped append-only evidence model that records meaningful contributions from humans, agents, organizations, and compute nodes.

Contribution classes:

- intellectual/IP
- architecture
- engineering/code
- product/design
- agent work
- compute
- testing/security/review
- research/data
- commercial/distribution
- capital

Activity is not automatically contribution. Evidence is gathered objectively, evaluated, and then accepted/rejected according to project governance.

Canonical flow:

```
Evidence -> Attribution -> Impact Assessment -> Acceptance -> Contribution Units
```

Contribution Units are initially non-transferable project accounting units. They are not automatically equity, securities, or tokens.

### 2. Compute Metering

Buzz Mesh remains responsible for discovery, routing, and execution. ST8WRX adds economic metering.

Each completed job should be attributable to:

- project
- requesting identity/agent
- serving node identity
- model/runtime
- start/end timestamps
- metered resource units
- input/output hashes where safe
- result acceptance
- pricing schedule/version

High-frequency compute receipts remain off-chain. Net balances settle periodically.

### 3. Project Agreements

Projects define signed, versioned economic rules covering:

- founding members
- contribution pool
- treasury/reserve
- contribution classes/weights
- governance thresholds
- revenue participation
- compute compensation
- IP/licensing rules
- dispute process
- token policy

Agreement documents remain private unless explicitly published. Their canonical digest and signatures may be anchored to BSV.

### 4. BSV Provenance and Settlement

`buzz-bsv` is an isolated service crate. It must not be required for the relay hot path.

Responsibilities:

- deterministic commitment construction
- Merkle batching
- anchor receipts
- transaction/provider abstraction
- BEEF/Atomic BEEF integration
- SPV-oriented verification boundary
- BRC-100-compatible wallet boundary
- settlement transactions
- future BSV-21 asset issuance
- future smart-contract integration

The relay submits qualifying records asynchronously. Chain/provider failure must never prevent ordinary Buzz collaboration.

### 5. Project Economy

Projects may choose one or more launch paths:

- sell
- license
- operate
- open source
- raise growth capital
- issue a utility asset when real project utility exists

Project tokens are optional. The platform must work without them.

### 6. ST8WRX Market

Discovery and transaction layer for:

- projects
- completed products
- licenses
- agents
- compute
- models
- APIs
- datasets
- bounties

The market should expose verifiable build history and project due-diligence evidence without exposing private project material.

## Identity model

Keep these authorities separate:

1. Buzz/Nostr identity
2. BSV wallet/spending authority
3. project governance authority
4. compute node identity

Bindings are signed claims, not key reuse. A compromised social identity must not automatically compromise funds.

## Privacy policy

Never publish to the public chain by default:

- private keys or seed material
- passwords/tokens
- private prompts
- customer PII
- trade secrets
- proprietary source
- confidential project agreements

For private artifacts, publish only safe metadata and cryptographic commitments.

## Contribution governance

AI may score and explain likely impact but cannot autonomously create binding ownership changes.

Recommended default for a three-founder project:

- equal founder social contract at formation
- configurable contribution pool
- configurable treasury/reserve
- objective evidence collection
- AI impact recommendation
- project approval for economically meaningful contribution state

All scoring-model/rule changes are versioned and signed.

## Disputes

Never rewrite contribution history. Disputes append decisions.

A rejected or adjusted contribution remains visible as evidence with a later adjudication record explaining the outcome and signatures/threshold used.

## On-chain storage tiers

### Tier 1: Buzz
Live messages, project state, prompts, private work, normal development activity.

### Tier 2: content/object storage
Large artifacts, media, datasets, models, repository bundles.

### Tier 3: BSV commitments
Merkle roots, agreements, contribution snapshots, release manifests, settlement state.

### Tier 4: fully on-chain publication
Explicit opt-in when permanence is itself valuable: public releases, licenses, public datasets, small applications/artifacts, public agent definitions.

## Execution sequence

### Milestone 1: current-main BSV foundation
- `buzz-bsv` crate
- deterministic Merkle commitment primitives
- provider interfaces
- no custody and no relay dependency

### Milestone 2: provenance vertical slice
- contribution event/data model
- project-scoped qualifying policy
- persisted anchor queue/receipt
- real BSV testnet transaction
- proof verification

### Milestone 3: compute accounting
- Mesh job metering
- node identity binding
- signed compute receipts
- project balance aggregation
- testnet settlement

### Milestone 4: agreements/economics
- project economy definition
- signed agreement versions
- governance thresholds
- Contribution Units and project dashboards

### Milestone 5: payments/contracts
- wallet binding
- treasury/payments
- bounty escrow
- milestone escrow
- revenue split

### Milestone 6: market
- project discovery
- verified due diligence
- sale/license flows

### Milestone 7: project asset launcher
- BSV-21 integration
- application utility configuration
- distribution/treasury controls
- wallet/API integration

### Milestone 8: platform economy
Only after real network utility is measurable. Define platform-token economics from observed compute, market, launch, storage, and service demand rather than speculative assumptions.

## Success criteria

The first complete vertical slice is successful when three developers can:

1. create/join a Buzz project
2. work using their own agents and machines
3. generate attributable contribution evidence
4. have a meaningful contribution accepted
5. produce a project contribution snapshot
6. anchor that snapshot to BSV testnet
7. independently verify the receipt
8. continue using Buzz normally if BSV infrastructure is unavailable

Everything after that builds on proven economic truth rather than blockchain decoration.
