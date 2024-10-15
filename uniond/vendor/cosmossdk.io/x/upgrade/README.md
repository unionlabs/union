---
sidebar_position: 1
---

# `x/upgrade`

## Abstract

`x/upgrade` is an implementation of a Cosmos SDK module that facilitates smoothly
upgrading a live Cosmos chain to a new (breaking) software version. It accomplishes this by
providing a `PreBlocker` hook that prevents the blockchain state machine from
proceeding once a pre-defined upgrade block height has been reached.

The module does not prescribe anything regarding how governance decides to do an
upgrade, but just the mechanism for coordinating the upgrade safely. Without software
support for upgrades, upgrading a live chain is risky because all of the validators
need to pause their state machines at exactly the same point in the process. If
this is not done correctly, there can be state inconsistencies which are hard to
recover from.

* [Concepts](#concepts)
* [State](#state)
* [Events](#events)
* [Client](#client)
    * [CLI](#cli)
    * [REST](#rest)
    * [gRPC](#grpc)
* [Resources](#resources)

## Concepts

### Plan

The `x/upgrade` module defines a `Plan` type in which a live upgrade is scheduled
to occur. A `Plan` can be scheduled at a specific block height.
A `Plan` is created once a (frozen) release candidate along with an appropriate upgrade
`Handler` (see below) is agreed upon, where the `Name` of a `Plan` corresponds to a
specific `Handler`. Typically, a `Plan` is created through a governance proposal
process, where if voted upon and passed, will be scheduled. The `Info` of a `Plan`
may contain various metadata about the upgrade, typically application specific
upgrade info to be included on-chain such as a git commit that validators could
automatically upgrade to.

```go
type Plan struct {
  Name   string
  Height int64
  Info   string
}
```

#### Sidecar Process

If an operator running the application binary also runs a sidecar process to assist
in the automatic download and upgrade of a binary, the `Info` allows this process to
be seamless. This tool is [Cosmovisor](https://github.com/cosmos/cosmos-sdk/tree/main/tools/cosmovisor#readme).

### Handler

The `x/upgrade` module facilitates upgrading from major version X to major version Y. To
accomplish this, node operators must first upgrade their current binary to a new
binary that has a corresponding `Handler` for the new version Y. It is assumed that
this version has fully been tested and approved by the community at large. This
`Handler` defines what state migrations need to occur before the new binary Y
can successfully run the chain. Naturally, this `Handler` is application specific
and not defined on a per-module basis. Registering a `Handler` is done via
`Keeper#SetUpgradeHandler` in the application.

```go
type UpgradeHandler func(Context, Plan, VersionMap) (VersionMap, error)
```

During each `EndBlock` execution, the `x/upgrade` module checks if there exists a
`Plan` that should execute (is scheduled at that height). If so, the corresponding
`Handler` is executed. If the `Plan` is expected to execute but no `Handler` is registered
or if the binary was upgraded too early, the node will gracefully panic and exit.

### StoreLoader

The `x/upgrade` module also facilitates store migrations as part of the upgrade. The
`StoreLoader` sets the migrations that need to occur before the new binary can
successfully run the chain. This `StoreLoader` is also application specific and
not defined on a per-module basis. Registering this `StoreLoader` is done via
`app#SetStoreLoader` in the application.

```go
func UpgradeStoreLoader (upgradeHeight int64, storeUpgrades *store.StoreUpgrades) baseapp.StoreLoader
```

If there's a planned upgrade and the upgrade height is reached, the old binary writes `Plan` to the disk before panicking.

This information is critical to ensure the `StoreUpgrades` happens smoothly at correct height and
expected upgrade. It eliminiates the chances for the new binary to execute `StoreUpgrades` multiple
times every time on restart. Also if there are multiple upgrades planned on same height, the `Name`
will ensure these `StoreUpgrades` takes place only in planned upgrade handler.

**Note:** The `StoreLoader` helper function for StoreUpgrades in v2 is not part of the `x/upgrade` module; 
instead, you can find it in the runtime v2 module.

### Proposal

Typically, a `Plan` is proposed and submitted through governance via a proposal
containing a `MsgSoftwareUpgrade` message.
This proposal prescribes to the standard governance process. If the proposal passes,
the `Plan`, which targets a specific `Handler`, is persisted and scheduled. The
upgrade can be delayed or hastened by updating the `Plan.Height` in a new proposal.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/upgrade/proto/cosmos/upgrade/v1beta1/tx.proto#L29-L40
```

#### Cancelling Upgrade Proposals

Upgrade proposals can be cancelled. There exists a gov-enabled `MsgCancelUpgrade`
message type, which can be embedded in a proposal, voted on and, if passed, will
remove the scheduled upgrade `Plan`.
Of course this requires that the upgrade was known to be a bad idea well before the
upgrade itself, to allow time for a vote.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/upgrade/proto/cosmos/upgrade/v1beta1/tx.proto#L47-L55
```

If such a possibility is desired, the upgrade height is to be
`2 * (VotingPeriod + DepositPeriod) + (SafetyDelta)` from the beginning of the
upgrade proposal. The `SafetyDelta` is the time available from the success of an
upgrade proposal and the realization it was a bad idea (due to external social consensus).

A `MsgCancelUpgrade` proposal can also be made while the original
`MsgSoftwareUpgrade` proposal is still being voted upon, as long as the `VotingPeriod`
ends after the `MsgSoftwareUpgrade` proposal.

## State

The internal state of the `x/upgrade` module is relatively minimal and simple. The
state contains the currently active upgrade `Plan` (if one exists) by key
`0x0` and if a `Plan` is marked as "done" by key `0x1`. The state
contains the consensus versions of all app modules in the application. The versions
are stored as big endian `uint64`, and can be accessed with prefix `0x2` appended
by the corresponding module name of type `string`. The state maintains a
`Protocol Version` which can be accessed by key `0x3`.

* Plan: `0x0 -> Plan`
* Done: `0x1 | byte(plan name)  -> BigEndian(Block Height)`
* ConsensusVersion: `0x2 | byte(module name)  -> BigEndian(Module Consensus Version)`
* ProtocolVersion: `0x3 -> BigEndian(Protocol Version)`

The `x/upgrade` module contains no genesis state.

## Events

The `x/upgrade` does not emit any events by itself. Any and all proposal related
events are emitted through the `x/gov` module.

## Client

### CLI

A user can query and interact with the `upgrade` module using the CLI.

#### Query

The `query` commands allow users to query `upgrade` state.

```bash
simd query upgrade --help
```

##### applied

The `applied` command allows users to query the block header for height at which a completed upgrade was applied.

```bash
simd query upgrade applied [upgrade-name] [flags]
```

If upgrade-name was previously executed on the chain, this returns the header for the block at which it was applied.
This helps a client determine which binary was valid over a given range of blocks, as well as more context to understand past migrations.

Example:

```bash
simd query upgrade applied "test-upgrade"
```

Example Output:

```bash
"block_id": {
    "hash": "A769136351786B9034A5F196DC53F7E50FCEB53B48FA0786E1BFC45A0BB646B5",
    "parts": {
      "total": 1,
      "hash": "B13CBD23011C7480E6F11BE4594EE316548648E6A666B3575409F8F16EC6939E"
    }
  },
  "block_size": "7213",
  "header": {
    "version": {
      "block": "11"
    },
    "chain_id": "testnet-2",
    "height": "455200",
    "time": "2021-04-10T04:37:57.085493838Z",
    "last_block_id": {
      "hash": "0E8AD9309C2DC411DF98217AF59E044A0E1CCEAE7C0338417A70338DF50F4783",
      "parts": {
        "total": 1,
        "hash": "8FE572A48CD10BC2CBB02653CA04CA247A0F6830FF19DC972F64D339A355E77D"
      }
    },
    "last_commit_hash": "DE890239416A19E6164C2076B837CC1D7F7822FC214F305616725F11D2533140",
    "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
    "validators_hash": "A31047ADE54AE9072EE2A12FF260A8990BA4C39F903EAF5636B50D58DBA72582",
    "next_validators_hash": "A31047ADE54AE9072EE2A12FF260A8990BA4C39F903EAF5636B50D58DBA72582",
    "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
    "app_hash": "28ECC486AFC332BA6CC976706DBDE87E7D32441375E3F10FD084CD4BAF0DA021",
    "last_results_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
    "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
    "proposer_address": "2ABC4854B1A1C5AA8403C4EA853A81ACA901CC76"
  },
  "num_txs": "0"
}
```

##### module versions

The `module_versions` command gets a list of module names and their respective consensus versions.

Following the command with a specific module name will return only
that module's information.

```bash
simd query upgrade module_versions [optional module_name] [flags]
```

Example:

```bash
simd query upgrade module_versions
```

Example Output:

```bash
module_versions:
- name: auth
  version: "2"
- name: authz
  version: "1"
- name: bank
  version: "2"
- name: distribution
  version: "2"
- name: evidence
  version: "1"
- name: feegrant
  version: "1"
- name: genutil
  version: "1"
- name: gov
  version: "2"
- name: ibc
  version: "2"
- name: mint
  version: "1"
- name: params
  version: "1"
- name: slashing
  version: "2"
- name: staking
  version: "2"
- name: transfer
  version: "1"
- name: upgrade
  version: "1"
- name: vesting
  version: "1"
```

Example:

```bash
regen query upgrade module_versions ibc
```

Example Output:

```bash
module_versions:
- name: ibc
  version: "2"
```

##### plan

The `plan` command gets the currently scheduled upgrade plan, if one exists.

```bash
regen query upgrade plan [flags]
```

Example:

```bash
simd query upgrade plan
```

Example Output:

```bash
height: "130"
info: ""
name: test-upgrade
time: "0001-01-01T00:00:00Z"
upgraded_client_state: null
```

##### authority

The `authority` command allows users to query the address that is authorized to submit upgrade proposals.

```bash
simd query upgrade authority [flags]
```

This command returns the bech32-encoded address of the account that has the authority to submit upgrade proposals.

Example:

```bash
simd query upgrade authority
```

Example Output:

```bash
cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn
```

#### Transactions

The upgrade module supports the following transactions:

* `software-proposal` - submits an upgrade proposal:

```bash
simd tx upgrade software-upgrade v2 --title="Test Proposal" --summary="testing" --deposit="100000000stake" --upgrade-height 1000000 \
--upgrade-info '{ "binaries": { "linux/amd64":"https://example.com/simd.zip?checksum=sha256:aec070645fe53ee3b3763059376134f058cc337247c978add178b6ccdfb0019f" } }' --from cosmos1..
```

* `cancel-upgrade-proposal` - cancels a previously submitted upgrade proposal:

```bash
simd tx upgrade cancel-upgrade-proposal --title="Test Proposal" --summary="testing" --deposit="100000000stake" --from cosmos1..
```

### REST

A user can query the `upgrade` module using REST endpoints.

#### Applied Plan

`AppliedPlan` queries a previously applied upgrade plan by its name.

```bash
/cosmos/upgrade/v1beta1/applied_plan/{name}
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/upgrade/v1beta1/applied_plan/v2.0-upgrade" -H "accept: application/json"
```

Example Output:

```bash
{
  "height": "30"
}
```

#### Current Plan

`CurrentPlan` queries the current upgrade plan.

```bash
/cosmos/upgrade/v1beta1/current_plan
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/upgrade/v1beta1/current_plan" -H "accept: application/json"
```

Example Output:

```bash
{
  "plan": "v2.1-upgrade"
}
```

#### Module versions

`ModuleVersions` queries the list of module versions from state.

```bash
/cosmos/upgrade/v1beta1/module_versions
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/upgrade/v1beta1/module_versions" -H "accept: application/json"
```

Example Output:

```bash
{
  "module_versions": [
    {
      "name": "auth",
      "version": "2"
    },
    {
      "name": "authz",
      "version": "1"
    },
    {
      "name": "bank",
      "version": "2"
    },
    {
      "name": "distribution",
      "version": "2"
    },
    {
      "name": "evidence",
      "version": "1"
    },
    {
      "name": "feegrant",
      "version": "1"
    },
    {
      "name": "genutil",
      "version": "1"
    },
    {
      "name": "gov",
      "version": "2"
    },
    {
      "name": "ibc",
      "version": "2"
    },
    {
      "name": "mint",
      "version": "1"
    },
    {
      "name": "params",
      "version": "1"
    },
    {
      "name": "slashing",
      "version": "2"
    },
    {
      "name": "staking",
      "version": "2"
    },
    {
      "name": "transfer",
      "version": "1"
    },
    {
      "name": "upgrade",
      "version": "1"
    },
    {
      "name": "vesting",
      "version": "1"
    }
  ]
}
```

#### Authority

`Authority` queries the address that is authorized to submit upgrade proposals.

```bash
/cosmos/upgrade/v1beta1/authority
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/upgrade/v1beta1/authority" -H "accept: application/json"
```

Example Output:

```json
{
"address": "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn"
}
```

### gRPC

A user can query the `upgrade` module using gRPC endpoints.

#### Applied Plan

`AppliedPlan` queries a previously applied upgrade plan by its name.

```bash
cosmos.upgrade.v1beta1.Query/AppliedPlan
```

Example:

```bash
grpcurl -plaintext \
    -d '{"name":"v2.0-upgrade"}' \
    localhost:9090 \
    cosmos.upgrade.v1beta1.Query/AppliedPlan
```

Example Output:

```bash
{
  "height": "30"
}
```

#### Current Plan

`CurrentPlan` queries the current upgrade plan.

```bash
cosmos.upgrade.v1beta1.Query/CurrentPlan
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.upgrade.v1beta1.Query/CurrentPlan
```

Example Output:

```bash
{
  "plan": "v2.1-upgrade"
}
```

#### Module versions

`ModuleVersions` queries the list of module versions from state.

```bash
cosmos.upgrade.v1beta1.Query/ModuleVersions
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.upgrade.v1beta1.Query/ModuleVersions
```

Example Output:

```bash
{
  "module_versions": [
    {
      "name": "auth",
      "version": "2"
    },
    {
      "name": "authz",
      "version": "1"
    },
    {
      "name": "bank",
      "version": "2"
    },
    {
      "name": "distribution",
      "version": "2"
    },
    {
      "name": "evidence",
      "version": "1"
    },
    {
      "name": "feegrant",
      "version": "1"
    },
    {
      "name": "genutil",
      "version": "1"
    },
    {
      "name": "gov",
      "version": "2"
    },
    {
      "name": "ibc",
      "version": "2"
    },
    {
      "name": "mint",
      "version": "1"
    },
    {
      "name": "params",
      "version": "1"
    },
    {
      "name": "slashing",
      "version": "2"
    },
    {
      "name": "staking",
      "version": "2"
    },
    {
      "name": "transfer",
      "version": "1"
    },
    {
      "name": "upgrade",
      "version": "1"
    },
    {
      "name": "vesting",
      "version": "1"
    }
  ]
}
```

#### Authority

`Authority` queries the address that is authorized to submit upgrade proposals.

```bash
cosmos.upgrade.v1beta1.Query/Authority
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.upgrade.v1beta1.Query/Authority
```

Example Output:

```json
{
  "address": "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn"
}
```

## Resources

A list of (external) resources to learn more about the `x/upgrade` module.

* [Cosmos Dev Series: Cosmos Blockchain Upgrade](https://medium.com/web3-surfers/cosmos-dev-series-cosmos-sdk-based-blockchain-upgrade-b5e99181554c) - The blog post that explains how software upgrades work in detail.
