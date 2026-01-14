# Voyager Architecture

## Overview

Relaying in IBC-based systems is intrinsically difficult. A relayer operates in a highly adversarial and failure-prone environment: multiple relayers compete to submit the same messages, RPC endpoints are unreliable, chains may experience reorgs, and incorrect submissions can cause permanent protocol-level failures. Voyager is designed from first principles to address these realities.

A reliable relayer must satisfy three core properties:

### Speed

IBC relaying is a competitive, latency-sensitive process. Multiple relayers observe the same on-chain events and race to submit packets, acknowledgements, or client updates. In practice, this often leads to frontrunning, wasted transactions, and redundant work. A relayer must therefore maximize throughput while avoiding unnecessary serialization.

### Data Integrity

Speed alone is insufficient. Submitting incorrect data—whether malformed packets, invalid proofs, or stale client updates—results in failed transactions at best and protocol-level faults at worst. In particular, **ordered channels are unforgiving**: if a single packet times out or is skipped, the entire channel may be permanently closed. A correct relayer must never drop packets and must guarantee that submitted data is internally consistent.

### Fast Startup and Crash Recovery

RPCs are unreliable and heterogeneous, especially when interacting with many different chains. While defensive error handling and retries help, crashes are inevitable. A production relayer must treat crashes as a normal operating condition and be able to restart quickly, resuming exactly where it left off without expensive recovery logic or resynchronization (see the xion relayer postmortem for a concrete example).

---

## Architectural Approach

Voyager addresses these constraints by modeling the entire relaying system as a **persistent finite state machine**.

At the core of Voyager is `voyager-vm`, an execution model in which:

- Every chain query
- Every transaction submission
- Every proof generation step
- And even intermediate data itself

is represented explicitly as a state transition.

These states are persisted in PostgreSQL via `pg-queue`, providing strong transactional guarantees.

### Transactional State Machine

Each message processed by Voyager runs inside a database transaction:

- State transitions are atomic
- Partial execution cannot corrupt global state
- Failed operations are safely retried

Because all state lives in PostgreSQL rather than in-memory structures, Voyager can crash and restart without losing progress. On restart, workers simply resume processing queued states, resulting in **near-instant startup times**.

### Parallelism by Construction

Each message fully encapsulates the state it needs to execute. This allows Voyager to safely process multiple messages concurrently without coordination hazards.

For example:

- One worker may be fetching block events
- Another may be submitting a light client update
- A third may be generating a state proof

All of these operations can proceed in parallel, dramatically improving throughput while preserving correctness. In this way, Voyager’s architecture simultaneously satisfies speed, data integrity, and fast recovery—without trade-offs.

For a deeper dive into the execution model, see the detailed Voyager architecture documentation.

## Light Clients

Voyager implements relaying logic for multiple light client protocols.

### Supported Client Types

- **Standard specifications**:

  - Tendermint
  - Ethereum

- **Custom client implementations**:

  - Designed for chains connected to Union
  - Often extend existing Ethereum or Tendermint logic
  - Add protocol-specific finality rules

The modular architecture allows Voyager to support new light clients without rewriting core logic.

---

## Ethereum L2 Clients (Conditional Clients)

Voyager supports several Ethereum Layer 2 networks using **conditional (recursive) light clients**.

### Key Verification Steps

For Ethereum L2s, Voyager verifies:

1. The L2 rollup contract’s account root on L1
2. That the L2 state is committed in the rollup contract
3. That the IBC account root matches the rollup state root

### Consensus Height Model

- The consensus height of an L2 client is derived from the L1
- For Ethereum L2s, this is typically the **Beacon Chain height**
- An L2 client can only be updated if the corresponding L1 client has a valid consensus state

### Finality Model

L2 finality is defined as:

```
L2 finality time = L2 settlement period + L1 finality time
```

This ensures that L2 state is only considered finalized once it is economically and cryptographically secure.

---

## Core Concepts

### IBC Specification

An IBC specification defines the semantics of a light-client-based bridging protocol. Every specification must include:

- A notion of a **light client update**
- A **provable store** for client and consensus states
- A host environment capable of producing cryptographic proofs (typically Merkle-based)

Voyager supports multiple specifications simultaneously, including:

- **ibc-union**
- **ibc-classic** (traditional IBC)

---

### Chain

A chain in Voyager is defined by:

- Monotonically increasing block heights
- A consensus mechanism with finality
- A provable storage layer
- One or more IBC interfaces

---

### Consensus

Consensus defines:

- Client state type
- Consensus state type
- Verification rules

Examples:

- Tendermint
- CometBLS
- Ethereum

---

### IBC Interface

An IBC interface defines how an IBC specification is implemented on a chain.

A single chain may expose multiple interfaces.

Examples:

- ibc-go-v8 / 08-wasm
- ibc-go-v8 native
- ibc-solidity
- ibc-cosmwasm

---

### Client Type

A client type is defined by four properties:

1. Compatible IBC specification
2. IBC interface
3. Consensus mechanism
4. Verifier implementation

This abstraction allows the same consensus to be verified across different chains and interfaces.

---

## Modules and Plugins

Voyager functionality is entirely composed of **modules** and **plugins**.

### Modules

- Provide read-only functionality
- Examples:

  - Chain height queries
  - Proof generation
  - Finality tracking

### Plugins

- Interact directly with the message queue
- Perform state transitions
- Submit transactions
- Coordinate complex workflows

Plugins may define custom internal message types and manage their own queues.

---

## JSON-RPC Interface

Voyager exposes a JSON-RPC API for querying chain and client state.

Capabilities include:

- Querying client state at a specific height
- Querying latest finalized state
- Decoding client state into structured JSON

This interface significantly improves developer experience and debugging capabilities.

---

## Modularity and Reuse

Voyager’s architecture allows extensive reuse:

- Ethereum state modules can be reused across:

  - Ethereum mainnet
  - L2s
  - Custom EVM chains (BSC, SEI, etc.)

Adding support for a new chain often requires **configuration, not new code**.

---

## Plugins and the Queue

Each plugin:

- Owns a dedicated queue topic
- Defines an interest filter
- Consumes only relevant messages

Plugins can:

- Spawn internal workflows
- Communicate across plugins
- Chain asynchronous operations

This model enables complex relaying logic without tight coupling.

---

## Recursive (State Lens) Clients

Recursive clients rely on state from other chains.

### Example: State Lens Clients

- L2 client finalized via L1
- L1 client finalized via L0 (host chain)

Voyager coordinates these dependencies using structured VM messages.

### Update Flow

- Update L2 client on L1
- Wait for finality
- Update L1 client on L0
- Submit final state lens update

Concurrency (`conc`) and sequencing (`seq`) primitives ensure correctness while maximizing parallelism.

---

## Required Components for Recursive Clients

To support state lens clients, Voyager requires:

- Client module
- Client bootstrap module
- Client update plugin
- Transaction plugins for each involved chain
- State, proof, and finality modules

Most components are reusable; only minimal new logic is required.

---

## Extensibility

The same architectural principles apply to:

- Ethereum rollups (Arbitrum, Optimism)
- Custom execution environments (Ethermint, SEI)
- Novel consensus mechanisms (Beacon Kit)
- Entirely new chains (Sui, Aptos)

Voyager’s design ensures that complexity scales **linearly**, not exponentially, as new chains and protocols are added.

---

## Summary

Voyager’s architecture achieves:

- **High-speed relaying** through parallel execution
- **Strong correctness guarantees** via transactional state machines
- **Instant recovery** after crashes
- **Deep modularity and reuse** across chains and protocols

This makes Voyager a robust foundation for next-generation IBC relaying across heterogeneous blockchain ecosystems.
