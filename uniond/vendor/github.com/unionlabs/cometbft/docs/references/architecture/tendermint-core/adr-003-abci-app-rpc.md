# ADR 3: Must an ABCI-app have an RPC server?

## Context

ABCI-server could expose its own RPC-server and act as a proxy to Tendermint.

The idea was for the Tendermint RPC to just be a transparent proxy to the app.
Clients need to talk to Tendermint for proofs, unless we burden all app devs
with exposing Tendermint proof stuff. Also seems less complex to lock down one
server than two, but granted it makes querying a bit more kludgy since it needs
to be passed as a `Query`. Also, **having a very standard rpc interface means
the light-client can work with all apps and handle proofs**. The only
app-specific logic is decoding the binary data to a more readable form (eg.
json). This is a huge advantage for code-reuse and standardization.

## Decision

We dont expose an RPC server on any of our ABCI-apps.

## Status

Implemented

## Consequences

### Positive

- Unified interface for all apps

### Negative

- `Query` interface

### Neutral
