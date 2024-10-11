# WebAssembly integration in Cosmos-SDK

This is a proposal for how to integrate the wasmer engine into the cosmos-sdk. Not on the technical details of the runtime implementation, but on how it connects to the rest of the SDK framework and all modules build upon it.

## External Requirements

This is based on work with the wasmer.io engine, including the following other repositories:

[cosmwasm](https://github.com/confio/cosmwasm):

* Configuration to build rust to WebAssembly
* Helper library to remove boilerplate (also see [cosmwasm-template](https://github.com/confio/cosmwasm-template))
* Testing support for smart contracts

[cosmwasm-vm](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm):

* Repeatable gas metering (fixed backend compiler options)
* Feature flags for backends

[cosmwasm-opt](https://github.com/confio/cosmwasm-opt):

* Deterministic builds (so we can map rust to on-chain wasm)
* Small wasm binaries (~100KB typical)

## Scope

When we discuss Web Assembly, most people see it as a magic fix that
allows us to upload sandboxed, user-defined code *anywhere*. The truth
is that while Web Assembly allows us to upload sandboxed functions to
run, we need to design the connections (arguments, return value, and environment)
with one specific case in mind.

As Aaron put it well, anywhere we accept a Go *interface* in the SDK,
we could place a WASM adapter there. But just as a struct only
implements one interface, we need to concentrate on one interface to adapt.

The majority of the use cases could be covered by a properly designed adapter for *Handler*
and this is where we will focus our work. This is also where most zone development is going -
into modules that expose a custom Handler. The initial implementation should minimally allow
us to create such contracts:

* Payment Channels
* Atomic Swap
* Escrow (with arbiter)
* Automatic payment distribution based on shares
* (maybe) Programmable NFTs: just as ethereum NFTs include some custom code to perform actions

As we expand the allows messages and query options, we will enable more use-cases.
The extensions to the API should probably be driven by real use cases -
please add your needs as issues on this repo.

### Future Directions

However, there are a number of other places where we could potentially provide another interface
for custom web assembly contracts,to be added after the original integration work. Some other
important use cases that will need different adapters:

* **IBC verification function** The current ICS23 spec mentions wasm uploaded code to verify external proofs. This should be easily creatable with a custom interface .
* **Delegation Rewards** Support different commission to different validators based on on-chain rules.
* **Signature Verification** Allow uploading new algorithms, like `ed25519` or `BLS`. These would obviously need to be enabled by a governance vote.

## Contents

More information on the current Web Assembly framework design:

* [Architecture](./Architecture.md) - a high-level view of what capabilities we expose to the Wasm contracts
* [Specification](./Specification.md) - a more detailed look at interfaces, methods, and structs to be used
* [Tutorial](https://www.cosmwasm.com) - tutorials and reference on building with cosmwasm
