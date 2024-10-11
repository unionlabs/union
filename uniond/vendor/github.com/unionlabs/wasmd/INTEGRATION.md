

# Integration

If you want to use Wasm in your own app, here is how you can get this working
quickly and easily. 
First start with this [article](https://medium.com/cosmwasm/cosmwasm-for-ctos-iv-native-integrations-713140bf75fc) 
in the "CosmWasm for CTOs" series that gives you a high level view. 
Then check to make sure you fit the pre-requisites,
then integrate the `x/wasm` module as described below, and finally, you
can add custom messages and queries to your custom Go/SDK modules, exposing
them to any chain-specific contract.

## Prerequisites

The pre-requisites of integrating `x/wasm` into your custom app is to be using 
a compatible version of the Cosmos SDK, and to accept some limits to the
hardware it runs on.

| wasmd | Cosmos SDK |
|:-----:|:----------:|
| v0.50 |  v0.50.1   |
| v0.45 |  v0.47.5   |
| v0.44 |  v0.47.5   |
| v0.43 |  v0.47.5   |
| v0.42 |  v0.47.5   |
| v0.41 |  v0.47.4   |
| v0.40 |  v0.47.1   |
| v0.31 |  v0.45.14  |
| v0.30 |  v0.45.11  |
| v0.29 |  v0.45.8   |
| v0.28 |  v0.45.5   |
| v0.27 |  v0.45.4   |
| v0.26 |  v0.45.1   |
| v0.25 |  v0.45.1   |
| v0.24 |  v0.45.0   |
| v0.23 |  v0.45.0   |
| v0.22 |  v0.45.0   |
| v0.21 |  v0.42.x   |


We currently only support Intel/AMD64 CPUs and OSX or Linux. For Linux, the standard build
commands work for `glibc` systems (Ubuntu, Debian, CentOS, etc). If you wish to compile
for a `muslc` based system (like Alpine), you need to compile the static library `wasmvm` locally
and compile go with the `muslc` build tag. Or just use the [Dockerfile](./Dockerfile),
which builds a static go binary in an Alpine system.

This limit comes from the Rust dll we use to run the wasm code, which comes 
from [`wasmvm`](https://github.com/CosmWasm/wasmvm). There are open issues
for adding [ARM support](https://github.com/CosmWasm/wasmvm/issues/53), and
[Windows support](https://github.com/CosmWasm/wasmvm/issues/28).
However, these issues are not high on the roadmap and unless you are championing
them, please count on the current limits for the near future.

## Quick Trial

The simplest way to try out CosmWasm is simply to run `wasmd` out of the box,
and focus on writing, uploading, and using your custom contracts. There is
plenty that can be done there, and lots to learn. 

Once you are happy with it and want to use a custom Cosmos SDK app, 
you may consider simply forking `wasmd`. *I highly advise against this*. 
Instead you should try one of the methods below.

## Integrating wasmd

### As external module

The simplest way to use `wasmd` is just to import `x/wasm` and wire it up
in `app.go`.  You now have access to the whole module and your custom modules
running side by side. (But the CosmWasm contracts will still only have access
to `bank` and `staking`... more below on [customization](#Adding-Custom-Hooks)).

The requirement here is that you have imported the standard sdk modules
from the Cosmos SDK, and enabled them in `app.go`. If so, you can just look
at [`wasmd/app/app.go`](https://github.com/CosmWasm/wasmd/blob/master/app/app.go#)
for how to do so (just search there for lines with `wasm`).

`wasmd` also comes with 2 custom `ante handlers`: 
* `CountTXDecorator` adds the TX position in the block into the context and passes it to the contracts
* `LimitSimulationGasDecorator` prevents an "infinite gas" query

In order to support these features you would need to add our custom
ante handlers into the `ante handler chain` as in: [`app/ante.go`](https://github.com/CosmWasm/wasmd/blob/master/app/ante.go)

### Copied into your app

Sometimes, however, you will need to copy `x/wasm` into your app. This should
be in limited cases, and makes upgrading more difficult, so please take the
above path if possible. This is required if you have either disabled some key
SDK modules in your app (eg. using PoA not staking and need to disable those
callbacks and feature support), or if you have copied in the core `x/*` modules
from the Cosmos SDK into your application and customized them somehow.

In either case, your best approach is to copy the `x/wasm` module from the
latest release into your application. Your goal is to make **minimal changes**
in this module, and rather add your customizations in a separate module.
This is due to the fact that you will have to copy and customize `x/wasm`
from upstream on all future `wasmd` releases, and this should be as simple
as possible.

If, for example, you have forked the standard SDK libs, you just want to
change the imports (from eg. `github.com/cosmos/cosmos-sdk/x/bank` to
`github.com/YOUR/APP/x/bank`), and adjust any calls if there are compiler 
errors due to differing APIs (maybe you use Decimals not Ints for currencies?).

By the end of this, you should be able to run the standard CosmWasm contracts
in your application, alongside all your custom logic.

## Adding custom hooks

Once you have gotten this integration working and are happy with the
flexibility it offers you, you will probably start wishing for deeper
integration with your custom SDK modules. "It sure is nice to have custom
tokens with a bonding curve from my native token, but I would love
to trade them on the exchange I wrote as a Go module. Or maybe use them
to add options to the exchange."

At this point, you need to dig down deeper and see how you can add this
power without forking either CosmWasm or `wasmd`. 

### Calling contracts from native code

This is perhaps the easiest part. Let's say your native exchange module
wants to call into a token that lives as a CosmWasm module. You need to
pass the `wasm.Keeper` into your `exchange.Keeper`. If you know the format
for sending messages and querying the contract (exported as json schema 
from each contract), and have a way of configuring addresses of supported
token contracts, your exchange code can simply call `wasm.Keeper.Execute`
with a properly formatted message to move funds, or `wasm.Keeper.SmartQuery`
to check balances.

If you look at the unit tests in [`x/wasm/internal/keeper`](https://github.com/CosmWasm/wasmd/tree/master/x/wasm/internal/keeper),
it should be pretty straight forward.

<!-- TODO dead link above -->

### Extending the Contract Interface

If you want to let the contracts access your native modules, the first
step is to define a set of Messages and Queries that you want to expose,
and then add them as `CosmosMsg::Custom` and `QueryRequest::Custom`
variants. You can see an example of the [bindings for Terra](https://github.com/CosmWasm/terra-contracts/tree/master/packages/bindings).

Once you have those bindings, use them to build a 
[simple contract using much of the API](https://github.com/CosmWasm/terra-contracts/tree/master/contracts/maker).
Don't worry too much about the details, this should be usable, but mainly
you will want to upload it to your chain and use for integration tests
with your native Cosmos SDK modules. Once that is solid, then add more
and more complex contracts.

You will then likely want to add a `mocks` package so you can provide
mocks for the functionality of your native modules when unit testing
the contracts (provide static data for exchange rates when your contracts
query it). You can see an example of [mocks for Terra contracts](https://github.com/CosmWasm/terra-contracts/tree/master/packages/mocks).

What these three steps provide is basically a chain-specific extension to the CosmWasm contract SDK.
Any CosmWasm contract can import your library (bindings and mocks) and easily get started using
your custom, chain-specific extensions just as easily as using the standard CosmWasm interfaces.
What is left is actually wiring them up in your chain so they work as desired.

Note, in order to ensure that no one tries to run the contracts on an unsupported chain,
you will want to include a `requires_XYZ` directive in your `bindings` library, this will
mean that only blockchain apps that explicitly declare their support for the `XYZ` extensions
(please rename XYZ to your project name) will allow the contract to be uploaded, and others
get error messages upon upload, not while running a critical feature later on.
You just need to add [a line like this](https://github.com/CosmWasm/terra-contracts/blob/master/packages/bindings/src/lib.rs#L13-L16)
to your bindings library to add the requirement to any contract that imports your `bindings` lib.

### Calling into the SDK

Before I show how this works, I want to remind you, if you have copied `x/wasm`,
please **do not make these changes to `x/wasm`**.

We will add a new module, e.g. `x/contracts`, that will contain custom
bindings between CosmWasm contracts and your native modules. There are two entry points
for you to use. The first is 
[`CustomQuerier`](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/query_plugins.go#L35),
which allows you to handle your custom queries. The second is 
[`CustomEncoder`](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/handler_plugin.go#L30),
which allows you to convert the `CosmosMsg::Custom(YourMessage)` types to `[]sdk.Msg` to be dispatched.

Writing stubs for these is rather simple. You can look at the `reflect_test.go` file to see this in action.
In particular, here [we define a `CustomQuerier`](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/reflect_test.go#L355-L385),
and here [we define a `CustomHandler`](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/reflect_test.go#L303-L353).
This code is responsible to take `json.RawMessage` from the raw bytes serialized from your custom types in Rust and parse it into
Go structs. Then take these go structs and properly convert them for your custom SDK modules.

You can look at the implementations for the `staking` module to see how to build these for non-trivial
cases, including passing in the `Keeper` via a closure. Here we 
[encode staking messages](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/handler_plugin.go#L114-L192).
Note that withdraw returns 2 messages, which is an option you can use if needed to translate into native messages.
When we [handle staking queries](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/x/wasm/internal/keeper/query_plugins.go#L109-L172)
we take in a `Keeper in the closure` and dispatch the custom `QueryRequest` from the contract to the native `Keeper` interface,
then encodes a response. When defining the return types, note that for proper parsing in the Rust contract, you 
should properly name the JSON fields and use the `omitempty` keyword if Rust expects `Option<T>`. You must also use
`omitempty` and pointers for all fields that correspond to a Rust `enum`, so exactly one field is serialized.

### Wiring it all together

Once you have written and tested these custom callbacks for your module, you need to enable it in your application.
The first step is to write an integration test with a contract compiled with your custom SDK to ensure it works properly,
then you need to configure this in `app.go`.

For the test cases, you must 
[define the supported feature set](https://github.com/CosmWasm/wasmd/blob/ade03a1d39a9b8882e9a1ce80572d39d57bb9bc3/x/wasm/internal/keeper/reflect_test.go#L52)
to include your custom name (remember `requires_XYZ` above?). Then, when creating `TestInput`, 
you can [pass in your custom encoder and querier](https://github.com/CosmWasm/wasmd/blob/ade03a1d39a9b8882e9a1ce80572d39d57bb9bc3/x/wasm/internal/keeper/reflect_test.go#L52).
Run a few tests with your compiled contract, ideally exercising the majority of the interfaces to ensure that all parsing between the contract and
the SDK is implemented properly.

Once you have tested this and are happy with the results, you can wire it up in `app.go`.
Just edit [the default `NewKeeper` constructor](https://github.com/CosmWasm/wasmd/blob/v0.8.0-rc1/app/app.go#L257-L258)
to have the proper `availableCapabilities` and pass in the `CustomEncoder` and `CustomQuerier` as the last two arguments to `NewKeeper`.
Now you can compile your chain and upload your custom contracts on it.