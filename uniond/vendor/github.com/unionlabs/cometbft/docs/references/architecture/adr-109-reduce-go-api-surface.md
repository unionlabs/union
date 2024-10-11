# ADR 109: Reduce CometBFT Go API Surface Area

## Changelog

- 2023-10-09: First draft (@thanethomson)

## Status

Accepted ([\#1484])

## Context

At present, the CometBFT codebase is somewhat monolithic, resulting in a very
large Go API surface area. This results in much more difficulty in terms of
changing the Go APIs, since making trivial breaking changes in non-critical
packages requires a major version bump. Doing so ultimately results in much
slower uptake of CometBFT releases and has produced substantial stagnation in
the codebase.

In order to mitigate this, several changes are proposed:

1. From when CometBFT v1.0 is released, major version bumps are only made when
   state-breaking changes are released. Minor version bumps can result in Go
   API-breaking changes (after deprecation warning for a reasonable period of
   time, as is customary for the Go standard library). Patch version bumps would
   guarantee no breaking changes.
2. Internalize a number of packages that do not need to be externally accessible
   along similar lines to that proposed in [ADR 060]. This involves moving these
   packages under the `/internal/` path in the repository, making those packages
   only accessible to the CometBFT codebase.

## Alternative Approaches

The following alternative approaches were considered.

1. Do nothing. This approach will keep the status quo along with its related
   problems.
2. Implement only one or two of the proposed changes. This will result in less
   flexibility than implementing all three.
3. Implement [ADR 060] as-is. The context in which ADR 060 was written, however,
   has changed, so certain changes need to be made to accommodate the new
   context.

## Decision

To implement all three approaches, using [ADR 060] as input, but updating
recommendations based on the current context.

## Detailed Design

### Versioning

The Go API stability guarantees provided by the new versioning policy must be
explicitly added to the documentation.

### Package Internalization

In order to move certain packages into the `internal` folder, effectively hiding
them from public use, the current package usage by some of the primary CometBFT
users should be considered. This ADR considers the [Cosmos SDK], [IBC Go] and
the [Cosmos Hub].

#### Cosmos SDK Imports

Since the [Cosmos SDK] is one of the primary users of CometBFT, it would make
sense to expose the minimal surface area needed by the Cosmos SDK in CometBFT
v1. Exposing internalized packages at a later stage constitutes a non-breaking
change, whereas internalizing packages later is breaking.

At the time of this writing, on the `main` branch, the Cosmos SDK imports the
following packages from the CometBFT repository for use at compile/run time and
during testing:

```bash
> go list -json ./... | jq '.Imports, .TestImports, .XTestImports' | grep cometbft | sort | uniq | tr -d '", '
github.com/cometbft/cometbft/abci/server
github.com/cometbft/cometbft/abci/types
github.com/cometbft/cometbft/abci/types
github.com/cometbft/cometbft/cmd/cometbft/commands
github.com/cometbft/cometbft/config
github.com/cometbft/cometbft/crypto
github.com/cometbft/cometbft/crypto/ed25519
github.com/cometbft/cometbft/crypto/encoding
github.com/cometbft/cometbft/crypto/secp256k1
github.com/cometbft/cometbft/crypto/sr25519
github.com/cometbft/cometbft/crypto/tmhash
github.com/cometbft/cometbft/libs/bytes
github.com/cometbft/cometbft/libs/cli
github.com/cometbft/cometbft/libs/json
github.com/cometbft/cometbft/libs/log
github.com/cometbft/cometbft/mempool
github.com/cometbft/cometbft/node
github.com/cometbft/cometbft/p2p
github.com/cometbft/cometbft/privval
github.com/cometbft/cometbft/proto/tendermint/crypto
github.com/cometbft/cometbft/proto/tendermint/p2p
github.com/cometbft/cometbft/proto/tendermint/types
github.com/cometbft/cometbft/proto/tendermint/types
github.com/cometbft/cometbft/proto/tendermint/version
github.com/cometbft/cometbft/proxy
github.com/cometbft/cometbft/rpc/client
github.com/cometbft/cometbft/rpc/client/http
github.com/cometbft/cometbft/rpc/client/local
github.com/cometbft/cometbft/rpc/client/mock
github.com/cometbft/cometbft/rpc/core/types
github.com/cometbft/cometbft/rpc/jsonrpc/server
github.com/cometbft/cometbft/types
github.com/cometbft/cometbft/types/time
github.com/cometbft/cometbft/version
```

#### Packages used by IBC Go

[IBC Go] on its `main` branch imports the following packages from CometBFT,
while using CometBFT v0.38.x:

```bash
> go list -json ./... | jq '.Imports, .TestImports, .XTestImports' | grep cometbft | sort | uniq | tr -d '", '
github.com/cometbft/cometbft/abci/types
github.com/cometbft/cometbft/config
github.com/cometbft/cometbft/crypto
github.com/cometbft/cometbft/crypto/secp256k1
github.com/cometbft/cometbft/crypto/tmhash
github.com/cometbft/cometbft/libs/bytes
github.com/cometbft/cometbft/libs/math
github.com/cometbft/cometbft/light
github.com/cometbft/cometbft/proto/tendermint/crypto
github.com/cometbft/cometbft/proto/tendermint/types
github.com/cometbft/cometbft/proto/tendermint/version
github.com/cometbft/cometbft/state
github.com/cometbft/cometbft/types
github.com/cometbft/cometbft/version
```

#### Packages used by the Cosmos Hub

The [Cosmos Hub], at the time of this writing, still uses the CometBFT v0.34.x
series (effectively still using Tendermint Core with the CometBFT alias):

```bash
> go list -json ./... | jq '.Imports, .TestImports, .XTestImports' | grep 'tendermint/tendermint' | sort | uniq | tr -d '", '
github.com/tendermint/tendermint/abci/types
github.com/tendermint/tendermint/abci/types
github.com/tendermint/tendermint/config
github.com/tendermint/tendermint/crypto
github.com/tendermint/tendermint/libs/cli
github.com/tendermint/tendermint/libs/json
github.com/tendermint/tendermint/libs/log
github.com/tendermint/tendermint/libs/os
github.com/tendermint/tendermint/libs/rand
github.com/tendermint/tendermint/libs/strings
github.com/tendermint/tendermint/p2p
github.com/tendermint/tendermint/privval
github.com/tendermint/tendermint/proto/tendermint/types
github.com/tendermint/tendermint/proto/tendermint/types
github.com/tendermint/tendermint/rpc/client/http
github.com/tendermint/tendermint/types
github.com/tendermint/tendermint/types/time
```

#### Public Package Inventory

Only the packages from the following table marked as necessary should still
remain publicly exported. All other packages in CometBFT should be moved under
`internal`.

| Package        | Used By                  | Necessary | Explanation |
|----------------|--------------------------|-----------|-------------|
| `abci`         | Cosmos SDK, IBC Go, Gaia | ✅ | |
| `cmd`          | Cosmos SDK               | ✅ | |
| `config`       | Cosmos SDK, IBC Go, Gaia | ✅ | |
| `crypto`       | Cosmos SDK, IBC Go, Gaia | ✅ | |
| `libs/bytes`   | Cosmos SDK, IBC Go       | ✅ | |
| `libs/cli`     | Cosmos SDK, Gaia         | ✅ | |
| `libs/json`    | Cosmos SDK, Gaia         | ✅ | |
| `libs/log`     | Cosmos SDK, Gaia         | ✅ | |
| `libs/math`    | IBC Go                   | ❓ | Necessary for `Fraction` type used by light client, which could be moved into `light` package instead |
| `libs/os`      | Gaia                     | ❌ | Uses `Exit` and `EnsureDir` functions |
| `libs/rand`    | Gaia                     | ❌ | |
| `libs/strings` | Gaia                     | ❌ | Uses `StringInSlice` function |
| `light`        | IBC Go                   | ✅ | |
| `mempool`      | Cosmos SDK               | ✅ | |
| `node`         | Cosmos SDK               | ✅ | |
| `p2p`          | Cosmos SDK, Gaia         | ✅ | |
| `privval`      | Cosmos SDK, Gaia         | ✅ | |
| `proto`        | Cosmos SDK, IBC Go, Gaia | ✅ | |
| `proxy`        | Cosmos SDK               | ✅ | |
| `rpc`          | Cosmos SDK, Gaia         | ✅ | |
| `state`        | IBC Go                   | ❌ | Only uses `TxResultsHash` type to check hash equivalence in test |
| `types`        | Cosmos SDK, IBC Go, Gaia | ✅ | |
| `version`      | Cosmos SDK, IBC Go       | ✅ | |

## Consequences

### Positive

- A smaller, more manageable Go API surface area.
- The team will be able to make internal Go API-breaking changes much quicker.

### Negative

- Some users (especially "power users" that make more extensive use of CometBFT
  internals) may experience breakages. If absolutely necessary, certain packages
  can be moved back out of the `internal` directory in subsequent minor
  releases.

[\#1484]: https://github.com/cometbft/cometbft/issues/1484
[ADR 060]: tendermint-core/adr-060-go-api-stability.md
[Cosmos SDK]: https://github.com/cosmos/cosmos-sdk/
[Cosmos Hub]: https://github.com/cosmos/gaia
[IBC Go]: https://github.com/cosmos/ibc-go
