# The Cosmos Security Handbook: Part 1 - Core Chain
<!-- markdown-link-check-disable-next-line -->
> Thank you to **[Roman Akhtariev](https://twitter.com/akhtariev) and [Alpin Yukseloglu](https://twitter.com/0xalpo)** for authoring this post. The original post can be found [here](https://www.faulttolerant.xyz/2024-01-16-cosmos-security-1/).

> [Trail of bits](https://www.trailofbits.com/) hosts another set of guidelines [here](https://github.com/crytic/building-secure-contracts/tree/master/not-so-smart-contracts/cosmos)

The defining property of the Cosmos stack is that it is unconstrained. The layers of the stack are porous, and, to a sufficiently motivated developer, nothing is off-limits. From a security standpoint, this freedom can be terrifying.

In this post, we aim to shed some light on the security landscape for the Cosmos stack.  We will emphasize areas that are particularly unintuitive, either because they are unique to Cosmos or because they are areas that developers who have not built appchains before are unlikely to have encountered.

Since the surface of new risks that come with developing appchains is vast, we cannot possibly fit everything into a single post. Thus, this article will be focused only on the security surface of the core chain. We are reserving CosmWasm and IBC-related risks for a future post.

## Overview

Application logic in Cosmos-based appchains can affect all parts of the stack. This level of expressivity necessitates important guardrails to be removed, which introduces certain risks that would otherwise be protected against. To a developer who is accustomed to building on general-purpose chains, the protections in place are often invisible to the point of going unnoticed. Thus, when faced with ultimate control, it can be difficult to differentiate between what is a new tool and what is an unmarked danger zone.

In the sections that follow, we break down the common ways developers can shoot themselves in the foot when building appchains. Some of these risks are more severe than others, but almost all are relatively unique to building appchains with the Cosmos SDK.

Specifically, we will cover the following areas with multiple concrete examples for each:

* Non-determinism
* In-protocol panics
* Unmetered/unbounded computation
* Prefix iteration & key malleability
* Fee market & gas Issues

## Non-Determinism

One of the consequences of opening up the consensus layer to app developers is that the code they write must not break critical properties required to reach consensus. Determinism is one such property that is particularly easy to compromise.

At a high level, determinism means that for the same input, all nodes in the network always produce the same output. It is an inherent requirement of blockchains. Without it, it is unclear what the nodes in the network are trying to agree on.

Simply put, non-determinism in the executed code can trigger the chain to fork or for honest validators to be unfairly slashed.

> As a brief side-note: while non-determinism should generally be avoided, we have provided a list in the appendix covering exactly which parts of the Cosmos SDK where the code needs to be deterministic. In general, anything that touches the state machine must be.
> 

### Randomness

Trivially, any use of randomness should be prohibited in the state machine. Keep an eye on the use of the Go `rand` package. It should not be used within the state-machine scope, including the imported dependencies.

In general, if randomness is used, it should be accessed in a deterministic way (much like [Chainlink's VRF](https://chain.link/vrf)).

### Go map internals

Under the hood, Go maps are implemented as a hash map of buckets where each bucket contains up to 8 key-value pairs. Since some key-value pairs within the bucket can be empty, Go uses randomness to select the starting element within the bucket. See [this article](https://medium.com/i0exception/map-iteration-in-go-275abb76f721) for the breakdown.

**When building on the Cosmos SDK, you should never iterate over a Go map**. Doing so results in non-determinism. Instead, if `map` usage is inevitable, it is necessary to convert it to a `slice` and sort it. See an example [here](https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/osmoutils/partialord/internal/dag/dag.go#L290-L302).

### Invalid Time Handling

Avoid using `time.Now()` since nodes are unlikely to process messages at the same point in time even if they are in the same timezone. Instead, always rely on `ctx.BlockTime()` which should be the canonical definition of what "now" is.

### API calls

Network requests are generally non-deterministic. As a result, they should be avoided in the state machine.

### Concurrency And Multithreading

Thread or goroutine pre-emption is likely to lack determinism. As a result, one should generally avoid using goroutines to be used anywhere within the state-machine scope. There are, of course, exceptions where we may process data concurrently for aggregation/counting which would be deterministic. However, such use cases are rare enough to consider the general use of goroutines in the app chain code as a red flag.

### Cross-Platform Floats

For reasons that could easily take up a [separate article](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/), it is safe to claim that float arithmetic is non-deterministic across platforms. Therefore, they must never be used in the app chain state-machine scope.

## In-Protocol Panics

One of the most unintuitive differences between developing a general-purpose chain and building one's own appchain is that code can be run in-protocol without being triggered by a specific transaction.

While this feature unlocks an incredible amount of expressivity for developers (such as custom precompiles and in-protocol arbitrage/liquidations), it also exposes various ways for the chain to be halted. One of the common ways this happens is through panics.

There are of course times when panics are appropriate to use instead of errors, but it is important to keep in mind that **panics in module-executed code (`Begin/EndBlock`) will cause the chain to halt**.

While these halts are generally not difficult to recover when isolated, they still pose a valid attack vector, especially if the panics can be triggered repeatedly. They also result in expensive social coordination and reputation costs stemming from downtime.

Thus, **we should be cognizant of when we use panics and ensure that we avoid them with behavior that could be handled well with an error.** Of course, it is still okay to guardrail unexpected flows with panics when needed, especially if the behavior is such that a chain halt *would* be appropriate.

Cosmos SDK takes care of catching and recovering from panics in all of `PrepareProposal` , `ProcessProposal`, `DeliverTx` , leaving only `Begin/EndBlock` for this class of vulnerabilities.

For reference, the Osmosis codebase catches and silently logs most panics stemming from `Begin/EndBlock` with [this](https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/osmoutils/cache_ctx.go#L13-L44) helper. In almost all cases, it is most productive to understand the reason behind panic and reconcile it without halting the chain entirely.

### Math Overflow

By default, all SDK math operations panic on overflows. This means that any math that is done in functions that get called in `Begin/EndBlock` should make sure to catch overflow panics using a helper similar to the one linked above.

For example, let's say a chain adds a feature that involves checking the spot price of arbitrary assets in `BeginBlock`. If the overflow panic is not caught, an attacker could create a market for a new asset and manipulate the price such that the spot price calculation overflows, triggering a panic at the top of each block. Since this is an easily repeatable attack, the attacker could presumably halt the chain in perpetuity until a hard fork patches the issue by catching overflow panics.

**The solution to this problem is to catch panics whenever there is SDK math run in `Begin/EndBlock`.**

### Bulk Coin Sends

If a chain supports custom token transfer logic (e.g. blacklists for USDC), it needs to make sure all token transfers in `Begin/EndBlock` properly catch panics. While this is generally quite straightforward to do, it is commonly missed in one context: bulk coin sends.

Specifically, the Cosmos SDK allows for multiple coins to be transferred in one function call through its `[SendCoins](https://github.com/cosmos/cosmos-sdk/blob/d55985637e1484309b09e76d29f04f2c7258c3de/x/bank/keeper/send.go#L202)` function. This is a black-box function that does not allow for individual validation of each token transfer, which often leads to it being overlooked. A single panic trigger in a call to `SendCoins` in `Begin/EndBlock` can trigger a chain halt.

While one can catch the panic on the entire `SendCoins` call, this would mean that an attacker can DoS all transfers in the batch. Thus, **the solution for these situations is to transfer coins one by one with `SendCoin` and verify each transfer so that problematic ones can be skipped.**

## Unmetered Computation

In the standard Cosmos stack, only stateful operations are gas-metered. This implies that out-of-block compute that is not triggered by messages has no notion of a gas limit. Thus, any form of unbounded execution in such a context can be used to halt the chain.

### Unmetered Execution in Hooks

Whenever one implements functionality involving hooks to arbitrary CosmWasm contracts, it is crucial to check whether this logic can be triggered by module-executed code. If it is, then an attacker can simply upload a contract that runs an infinite loop to halt the chain.

For instance, if a chain allows for arbitrary token transfer hooks and triggers them in `Begin/EndBlock`, then an attacker can create a token that executes an infinitely looping CosmWasm contract. Once this token is transferred in the next block's `BeginBlock`, the chain will halt.

**The solution to this problem is to [wrap risky function calls](https://github.com/osmosis-labs/osmosis/blob/2a64b0b6171478b81b017a001f5179b199a38628/x/tokenfactory/keeper/before_send.go#L121-L128) in a separate Context that has a gas limit.** This assigns a gas budget to such calls that prevent them from running unboundedly and halting the chain.

### Poorly Chosen Loop/Recursion Exit Condition

This is a consideration that seems trivial but comes up much more frequently than one might expect. If a loop in unmetered code is never exited or a recursion base case is never hit, it might lead to an expensive chain halt.

### Slow Convergence in Math Operations

A few months ago, a security researcher [reported a vulnerability](https://blog.trailofbits.com/2023/10/23/numbers-turned-weapons-dos-in-osmosis-math-library/) in the Osmosis codebase stemming from [PowApprox function](https://github.com/osmosis-labs/osmosis/blob/44a6a100a92f2984a760b41b7486fb9000ac670e/osmomath/math.go#L86). The crux of the issue was centered around long-lasting convergence for certain input values. A determined attacker could in theory use such edge cases to temporarily halt the chain. **The solution in these cases is simple - [introduce a constant loop bound](https://github.com/osmosis-labs/osmosis/pull/6627).**

As a side note, from our experience, rational approximation is a more accurate and performant substitute to Taylor expansion which is used in `PowApprox` of the above example. See [this article](https://xn--2-umb.com/22/approximation/) for details.

## Key Malleability and Prefix Iteration

When onboarding onto the Cosmos stack, developers must familiarize themselves with its [key/value stores](https://docs.cosmos.network/v0.46/core/store.html). One particularly insidious class of bugs is related to how one sets keys when writing to these stores. Even slight mistakes in this process can lead to critical vulnerabilities that are usually simple to detect and exploit.

### Store Prefix Ending In Serialized ID

To guarantee uniqueness, it is common to serialize IDs in a store key. However, since these IDs are often in the control of potential attackers (e.g. triggering higher pool IDs by creating more pools), some portion of the keys in these cases can be malleable.

One way that this frequently surfaces is when developers run a prefix iterator over keys that end in a malleable component (e.g. the pool's ID, which an attacker can increase by creating empty pools). In these cases, the iterator might include objects that were not supposed to be in the loop in the first place, meaning that an attacker can trigger unintended behavior. For instance, the prefix iteration on a key that ends with `42` would also loop over ID `421`, etc. A more involved example covering a concrete attack vector can be found in the appendix.

**This bug can be resolved in one of two ways:**

a) Add a key separator suffix to the prefix as done [here](https://github.com/osmosis-labs/osmosis/blob/450f7570a34876b14c61e883f2bf2ea81944f9c7/x/concentrated-liquidity/types/keys.go#L191-L195).

b) Convert malleable numbers in keys to big-endian as done [here](https://github.com/osmosis-labs/osmosis/blob/450f7570a34876b14c61e883f2bf2ea81944f9c7/x/gamm/types/key.go#L60-L62).

### Key Uniqueness

In many instances, it might be tempting to identify a data structure by a collection of their fields. For instance, one might want to key liquidity pools in the following way:

```go
// KeyPool returns the key for the given pool
func KeyPool(pool Pool) []byte {
    return []byte(fmt.Sprintf("%s%s%s%s", PoolPrefix, pool.GetToken0(), pool.GetToken1(), pool.GetSpreadFactor()))
}

```

However, note that there can be multiple pools with the same tokens and spread factor. As a result, an existing entry could be completely overwritten. While the example above is somewhat trivial and would be easily caught by unit tests, more complex instances of this issue come up frequently enough to justify mentioning it. **The solution here is to ensure that keys are unique, usually through the addition of an ID component or some equivalent.**

### Iterator Bounds

This is another simple example that sometimes catches even the most seasoned Cosmos developers. Prefix iteration is inclusive of the start byte but exclusive of the end byte.

As a result, iterating forwards requires initializing the iterator with the given start value. See this example from the Osmosis concentrated liquidity module:

```go
// <https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/x/concentrated-liquidity/swapstrategy/one_for_zero.go#L204-L205>
startKey := types.TickIndexToBytes(currentTickIndex)
iter := prefixStore.Iterator(startKey, nil)

```

On the other hand, iterating in reverse requires adding one more byte to the end byte of the reverse iterator:

```go
// <https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/x/concentrated-liquidity/swapstrategy/zero_for_one.go#L202-L204>
startKey := types.TickIndexToBytes(currentTickIndex + 1)
iter := prefixStore.ReverseIterator(nil, startKey)

```

Being off by one on such iterators is a frequent cause of critical vulnerabilities that are difficult to catch through testing.

## Fee Market and Gas Issues

For developers on general-purpose chains, fees are usually treated as a black box. Thus, fee-related issues can be particularly unintuitive for those who are not used to thinking about fees as an abstraction. Regardless of whether one is implementing [novel fee mechanisms](https://www.faulttolerant.xyz/2023-11-17-fee-credits/) or simply running something out-of-the-box, appchain developers generally have to grapple with risks that arise from having more control over gas.

### Mispriced State Writes

If a data structure is written to the state during a user-executed message flow, the creation of this data structure must be bounded by a high enough fee to deter spam. If this is not done properly (i.e. either there are insufficient or no fees), then an attacker can DoS the chain similar to how they would be able to through large/unbounded compute in unmetered flows.

While this might seem trivial in simple cases, there are many scenarios where pricing state writes is nontrivial. This complexity generally surfaces in actions that create externalities for other users.

For instance, if the protocol includes logic where an arbitrary number of tokens are iterated through linearly, then each new token can potentially push an increasing cost onto the system. In such cases, additional (and scaling) gas [must be charged](https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/x/tokenfactory/keeper/createdenom.go#L19-L22) to ensure the protocol is not vulnerable to DoS attacks.

### Fees on Contract-called Functions

Charging fixed fees distinct from gas is a relatively common design pattern (for instance, Osmosis charges a governance-set fee denominated in OSMO for creating pools). However, introducing such fees often results in risks in scenarios where contracts act on behalf of users. Specifically, this design pattern can cause the fees to be charged incorrectly or, in some cases, even prevent the contract from being used at all.

In such cases, it often makes sense to simply [charge the fee as additional gas](https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/x/tokenfactory/keeper/createdenom.go#L95-L98) so that it gets floated up to the caller. If this is not possible due to the fee being too high, then the fee charge needs to be factored into the design of the contract(s) triggering it.

### Broken or Missing Fee Market

During periods of high network usage, it is critical to ensure that high-value transactions have a priority for getting on-chain. For example, liquidations can continue to happen to ensure the health of the market as long as a higher fee is provided. To achieve this, there has to be a proxy for demand. [EIP-1559](https://www.youtube.com/watch?v=62UI3Js30Io) is a protocol that incorporates a variable base fee that increases or decreases based on historic block sizes. [Osmosis recently implemented](https://github.com/osmosis-labs/osmosis/blob/b0aee0006ce55d0851773084bd7880db7e32ad70/x/txfees/keeper/mempool-1559) this directly in the mempool.

However, the superior long-term solution is to integrate the fee market directly into consensus by leveraging `ABCI 2.0`. The Skip team has been spearheading [the implementation](https://github.com/skip-mev/feemarket) of this initiative which will be released as a pluggable component that chains can integrate.

### Transaction Simulation and Execution Gas Consistency

Before submitting a transaction on-chain, clients attempt to simulate its execution to determine how much gas to provide. There is a separate execution mode for simulation that does not commit state but attempts to estimate gas.

Due to [challenges with how Cosmos SDK gas estimation works](https://github.com/cosmos/cosmos-sdk/issues/18834), the gas estimate often ends up being inconsistent with the actual execution. Many clients get around this today by multiplying their gas estimates by a constant multiplier.

If the chain increases gas usage in ways that are not included in simulation logic, this could break many clients at chain upgrade time until they increase their gas multipliers.

The specific area to pay attention to on this front is the `simulate` parameter in the `AnteHandler` API. An example that could cause issues might look like the following:

```go
func (mfd MyMemPoolDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
    if !simulate {
    // Do some gas intensive logic such as many store reads/writes
    // This will lead to inconsistencies between transaction simulation and execution
    }
}

```

While this does not affect chain security, it can cause severe client instability until clients update their gas estimation logic. If your appchain supports high-value and time-sensitive transactions such as ones related to collateral and leverage, such issues could be quite problematic.

## Closing Remarks

Over the past few months, Cosmos has seen an influx of liquidity and users. If we as an ecosystem want to thrive and compete with centralized entities, there has to be a shared focus on security and the desire to share insights on best practices around our stack.

Our hope with this post was to lay the groundwork from a security standpoint for engineers onboarding onto the Cosmos SDK and appchains more broadly. While this is far from being exhaustive, it is a distillation of years of experience and scar tissue accumulated over many multi-million dollar war rooms.

As an ecosystem, we cannot afford recklessness or lack of security awareness. **If you have any feedback or contributions you would like to see added to this post (or one of our future posts on CosmWasm and IBC), please reach out to us on Twitter/X @0xalpo and @akhtariev!**

## Appendix

### State-machine Scope

The state-machine scope includes the following areas:

* All messages supported by the chain, including:
    * Every msg's `ValidateBasic` method
    * Every msg's `MsgServer` method
    * Net gas usage, in all execution paths
    * Errors (assuming use of Cosmos-SDK errors)
    * State changes (namely every store write)
    * `AnteHandler`s in `execModeFinalize`
    * `PostHandler`s in `execModeFinalize`
    * Cosmwasm-whitelisted queries
* All BeginBlock/EndBlock logic
* ABCI 2.0 `ProcessProposal`
* ABCI 2.0 `VerifyVoteExtensions`

The following are NOT in the state-machine scope:

* Events
* Queries that are not Cosmwasm-whitelisted
* CLI interfaces
* Errors (assuming use of Go-native errors)
* ABCI 2.0 `PrepareProposal`
* ABCI 2.0 `ExtendVote`
* `AnteHandler`s in any mode other than `execModeFinalize`
* `PostHandler`s in any mode other than `execModeFinalize`

### Key Malleability and Prefix Iteration Attack Example

Consider the code below that checks whether the given address is the owner of a given position ID via an`IsPositionOwner` function. The `KeyAddressPoolIdPositionId` formats the key ending with a pool ID as a string. `HasAnyAtPrefix` function checks if there exists an entry at a given prefix.

```go
// KeyAddressPoolIdPositionId returns the full key needed to store the position id for given addr + pool id + position id combination.
func KeyAddressPoolIdPositionId(addr sdk.AccAddress, poolId uint64, positionId uint64) []byte {
    return []byte(fmt.Sprintf("%s%s%x%s%d%s%d", PositionPrefix, KeySeparator, addr.Bytes(), KeySeparator, poolId, KeySeparator, positionId))
}

```

`IsPositionOwner(address sdk.AccAddress, positionID uint64) bool` function checks if there exists an entry in the key-value store with the given prefix formatted per `KeyAddressPoolIdPositionId` structure.

To achieve that, it might be tempting to use a store helper such as `HasAnyAtPrefix`. Unfortunately, this would be fatal.

```go
// HasAnyAtPrefix returns true if there is at least one value in the given prefix.
func HasAnyAtPrefix[T any](storeObj store.KVStore, prefix []byte, parseValue func([]byte) (T, error)) (bool, error) {
    _, err := GetFirstValueInRange(storeObj, prefix, sdk.PrefixEndBytes(prefix), false, parseValue)
    if err != nil {
        if err == ErrNoValuesInRange {
            return false, nil
        }
        return false, err
    }

    return true, nil
}

// GetFirstValueInRange returns the first value between [keyStart, keyEnd)
func GetFirstValueInRange[T any](storeObj store.KVStore, keyStart []byte, keyEnd []byte, reverseIterate bool, parseValue func([]byte) (T, error)) (T, error) {
    iterator := makeIterator(storeObj, keyStart, keyEnd, reverseIterate)
    defer iterator.Close()

    if !iterator.Valid() {
        var blankValue T
        return blankValue, ErrNoValuesInRange
    }

    return parseValue(iterator.Value())
}

```

To fully grasp the root of the issue, consider the following snapshot:

```go
// <https://go.dev/play/p/Uzl3cqYPtG1>
func test(poolId uint64) {
    formattedString := fmt.Sprintf("%d", poolId)
    byteSlice := []byte(formattedString)
    fmt.Printf("Original String: %s\\n", formattedString)
    fmt.Printf("Byte Slice: %v\\n", byteSlice)
}

func main() {
    poolIDOne := uint64(42)
    poolIDTwo := uint64(421)

    // Prints:
    // Original String: 42
    // Byte Slice: [52 50]
    test(poolIDOne)

    // Prints:
    // Original String: 421
    // Byte Slice: [52 50 49]
    test(poolIDTwo)
}

```

Both `poolIDOne` and `poolIDTwo` have the same prefix.

Now, our original position existence check with `HasAnyAtPrefix` would pass if it was run on `poolID` of `42` when the user only owned a position ID `421`. This can result in malicious users getting access to positions that they do not own.
