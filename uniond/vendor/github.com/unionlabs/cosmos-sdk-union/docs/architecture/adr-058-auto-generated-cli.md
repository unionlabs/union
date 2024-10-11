# ADR 058: Auto-Generated CLI

## Changelog

* 2022-05-04: Initial Draft

## Status

ACCEPTED Partially Implemented

## Abstract

In order to make it easier for developers to write Cosmos SDK modules, we provide infrastructure which automatically
generates CLI commands based on protobuf definitions.

## Context

Current Cosmos SDK modules generally implement a CLI command for every transaction and every query supported by the
module. These are handwritten for each command and essentially amount to providing some CLI flags or positional
arguments for specific fields in protobuf messages.

In order to make sure CLI commands are correctly implemented as well as to make sure that the application works
in end-to-end scenarios, we do integration tests using CLI commands. While these tests are valuable on some-level,
they can be hard to write and maintain, and run slowly. [Some teams have contemplated](https://github.com/regen-network/regen-ledger/issues/1041)
moving away from CLI-style integration tests (which are really end-to-end tests) towards narrower integration tests
which exercise `MsgClient` and `QueryClient` directly. This might involve replacing the current end-to-end CLI
tests with unit tests as there still needs to be some way to test these CLI commands for full quality assurance.

## Decision

To make module development simpler, we provide infrastructure - in the new [`client/v2`](https://github.com/cosmos/cosmos-sdk/tree/main/client/v2)
go module - for automatically generating CLI commands based on protobuf definitions to either replace or complement
handwritten CLI commands. This will mean that when developing a module, it will be possible to skip both writing and
testing CLI commands as that can all be taken care of by the framework.

The basic design for automatically generating CLI commands is to:

* create one CLI command for each `rpc` method in a protobuf `Query` or `Msg` service
* create a CLI flag for each field in the `rpc` request type
* for `query` commands call gRPC and print the response as protobuf JSON or YAML (via the `-o`/`--output` flag)
* for `tx` commands, create a transaction and apply common transaction flags

In order to make the auto-generated CLI as easy to use (or easier) than handwritten CLI, we need to do custom handling
of specific protobuf field types so that the input format is easy for humans:

* `Coin`, `Coins`, `DecCoin`, and `DecCoins` should be input using the existing format (i.e. `1000uatom`)
* it should be possible to specify an address using either the bech32 address string or a named key in the keyring
* `Timestamp` and `Duration` should accept strings like `2001-01-01T00:00:00Z` and `1h3m` respectively
* pagination should be handled with flags like `--page-limit`, `--page-offset`, etc.
* it should be possible to customize any other protobuf type either via its message name or a `cosmos_proto.scalar` annotation

At a basic level it should be possible to generate a command for a single `rpc` method as well as all the commands for
a whole protobuf `service` definition. It should be possible to mix and match auto-generated and handwritten commands.

## Consequences

### Backwards Compatibility

Existing modules can mix and match auto-generated and handwritten CLI commands so it is up to them as to whether they
make breaking changes by replacing handwritten commands with slightly different auto-generated ones.

For now the SDK will maintain the existing set of CLI commands for backwards compatibility but new commands will use
this functionality.

### Positive

* module developers will not need to write CLI commands
* module developers will not need to test CLI commands
* [lens](https://github.com/strangelove-ventures/lens) may benefit from this

### Negative

### Neutral

## Further Discussions

We would like to be able to customize:

* short and long usage strings for commands
* aliases for flags (ex. `-a` for `--amount`)
* which fields are positional parameters rather than flags

It is an [open discussion](https://github.com/cosmos/cosmos-sdk/pull/11725#issuecomment-1108676129)
as to whether these customizations options should line in:

* the .proto files themselves,
* separate config files (ex. YAML), or
* directly in code

Providing the options in .proto files would allow a dynamic client to automatically generate
CLI commands on the fly. However, that may pollute the .proto files themselves with information that is only relevant
for a small subset of users.

## References

* https://github.com/regen-network/regen-ledger/issues/1041
* https://github.com/cosmos/cosmos-sdk/tree/main/client/v2
* https://github.com/cosmos/cosmos-sdk/pull/11725#issuecomment-1108676129
