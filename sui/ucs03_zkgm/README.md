This document only explains the sui zkgm implementation details which might be useful to the relayer devs, operators and zkgm devs.

## Sui ZKGM's receive process

Traditionally, all implementations have a single entrypoint to receive an ibc packet, where the relevant function is called by the ibc protocol.
And the receiving of multiple packets and instructions happen within a single function. But with Sui, this is simply not possible because of how
the type system and coins work in Sui.

In Sui, instead of having a contract as a token similar to `ERC20` or `CW20` contracts, you have types, which construct a coin:

```move
public struct Coin<phantom T> has key, store

public struct TreasuryCap<phantom T> has key, store

// ..
```

When you create a token, you are actually creating the necessary types wrapping the type `T` which defines the attributes of a specific token/coin.
The important thing to note here is that, you have to know the type `T` at compile time to be able to do any token operation with that specific
token. And unfortunately, SUI doesn't have variadic generic or templates to define arbitrary amount of type `T`'s in the function signature
similar to:

```move
/// this function might be called with arbitrary amount of types
fun recv_packet<T..>() {}
```

This makes it impossible to receive a zkgm packet that makes a token operation with different tokens multiple times like this:

```move

Batch [
  TokenOrderV2<T1>,
  TokenOrderV2<T2>,
]
```

Note that, when calling the `recv_packet`, there's no way of providing arbitrary amount of types.

The naive solution to this could be having multiple `recv_packet_N` functions which will take different number of types. But this is hard to deal with
and it's hardly a generic solution.

We are taking advantage of SUI's PTB (Programmable Transaction Block) mechanism to offload the logic of calling `recv_packet` with multiple types and still enforce
correct execution.

This begins with defining a struct with no capabilities such that it cannot be stored, saved as an object or dropped:

```move
public struct RecvCtx {
    packet_ctxs: vector<ZkgmPacketCtx>,
    packets: vector<Packet>,
    cursor: u64,
}
```

Then we start the execution of the PTB by calling `begin_recv`:

```move
public fun begin_recv(
    packet_source_channels: vector<u32>,
    packet_destination_channels: vector<u32>,
    packet_data: vector<vector<u8>>,
    packet_timeout_heights: vector<u64>,
    packet_timeout_timestamps: vector<u64>,
): RecvCtx
```

This function parses the zkgm instructions and return the context where the instructions, ibc packets and the pointer to the instruction to be run are stored.
The fact that `RecvCtx` is not implementing drop and it cannot be stored or turned into an object guarantees that the whole PTB will fail unless the `RecvCtx`
is destructed. We will see in a moment how this happens.

Then the returned context will be used to call `recv_packet<T>` as much as needed:

```move
public fun recv_packet<T>(
    ibc: &mut ibc::IBCStore,
    zkgm: &mut RelayStore,
    clock: &Clock,
    relayer: address,
    relayer_msg: vector<u8>,
    mut exec_ctx: RecvCtx,
    ctx: &mut TxContext
): RecvCtx
```

Here, the `recv_packet<T>` function will stop executing the instructions if it already consumed a `T` and it is now needed to be called with a new
type `T`. Here we show an example:

```
Batch [
  TokenOrder<T1>,
  Forward,
  TokenOrder<T2>,
]
```

In the case above, `recv_packet` would have to be called twice. At the first iteration, it will run the `TokenOrder<T1>` and `Forward` instructions
and when it hits the `TokenOrder<T2>`, it will properly set the pointer and return the `RecvCtx`. Then it will run the `TokenOrder<T2>` in the next
call.

We have the `end_recv` entrypoint to finalize the receiving process:

```move
public fun end_recv(
    ibc: &mut ibc::IBCStore,
    clock: &Clock,
    proof: vector<u8>,
    proof_height: u64,
    relayer: address,
    relayer_msg: vector<u8>,
    exec_ctx: RecvCtx,
)
```

This function checks whether all the instructions are run within the `exec_ctx` and if not, it aborts. But they are already run, then it consumes
the `RecvCtx` and calls the `ibc::recv_packet` to commit the packet. Note again that the Sui's type-system enforces `RecvCtx` to be passed
to this function within the same PTB. Otherwise, it will not be possible to consume the type and the whole PTB will be reverted.

Most of the work here is done by voyager because now it cannot simply forward the packets but it needs to check whether the packet is a zkgm packet.
And if so, it needs to parse the instructions and decide on how many times it needs to call the `recv_packet` function and with what types.

## Receiving a `TokenOrder`

You might be confused about how voyager knows what's the type `T`. We defined several entrypoints to manage coins of arbitrary types.

The first one here is `register_capability<T>` where, the ownership of the `TreasuryCap` of a coin is transferred into ZKGM. Note that in SUI,
owning the `TreasuryCap` means you have privileged rights to the coin (mint, burn, etc).

Then when the first `TokenOrderV2` happens without the solver api, the token `T` is claimed for the used channel, path and metadata.
This guarantees that only this token will be used furthermore for this combination. When doing the claim, we also check whether the metadata of
the token matches the order. This means that the first transfer which uses this token must always use the kind `TOKEN_ORDER_KIND_INITIALIZE`.
And the metadata must be:

```move
public struct SuiTokenMetadata has copy, drop {
    name: String,
    symbol: String,
    decimals: u8,
    owner: address,
    icon_url: Option<String>,
    description: String,
}
```

This way, we will be able to check whether the coin has the correct attributes.

The other important thing to consider here is that in other implementations, zkgm will be the minter of the token but the actual owner might be
someone else. But in this case, the owner of all tokens is ZKGM in the perspective of SUI. This is due to the fact that we need to pass the ownership
of the `TreasuryCap` to ZKGM and there is no such mechanism as `minter` or `burner`. If you have own the `TreasuryCap`, you can mint and burn. But
when doing the registration, the sender can choose to put themselves as admin or they can choose to give the total ownership to ZKGM by setting the
owner to `@0x0`. Our ZKGM implementation exposes some passthrough functions that call the coin functions only if the sender is the admin of that token.
Right now, the token ownership is locked but in the future, we might consider adding the capability to extract out the full ownership of the tokens (this will
make it impossible to receive the tokens in SUI tho).

## Receiving a `TokenOrder` with the solver API

The solver api is very similar to a regular `TokenOrderV2` where the owner of the token is still ZKGM. But this time, the `solve` functionality is embedded
in ZKGM and will be used instead of regular `mint` and `burn`. This function will ensure that this token can be received from some certain destinations (fungibility).
The difference from the previous one is in the regular version, a specific token can only be received using a destination single channel, path and metadata. But
with the solver API, the token might be coming from multiple channels and paths. It will be valid as long as the fungible counterparty info is set.
