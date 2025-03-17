import { Effect } from "effect"
import { type Address, createPublicClient, fromHex, http } from "viem"
import type { Hex } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import type { Channel } from "$lib/schema/channel.ts"
import { request } from "graphql-request"
import { GRAQPHQL_URL } from "@unionlabs/client"
import type { Chain } from "$lib/schema/chain.ts"
import { getChainFromWagmi } from "$lib/wallet/evm"
import { getCosmosPublicClient } from "$lib/services/cosmos/clients.ts"
import { tokenWrappingQuery } from "$lib/queries/tokens.svelte.ts"
import { GetQuoteError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { Aptos, AptosConfig, Network, MoveVector } from "@aptos-labs/ts-sdk"

export const getQuoteToken = (
  sourceChain: Chain,
  base_token: Hex,
  channel: Channel,
  destinationChain: Chain
) =>
  Effect.gen(function* () {
    const { v1_ibc_union_tokens } = yield* Effect.tryPromise({
      try: () =>
        request(GRAQPHQL_URL, tokenWrappingQuery, {
          base_token,
          destination_channel_id: channel.source_channel_id,
          source_chain_id: sourceChain.chain_id
        }),
      catch: error => {
        return new GetQuoteError({ cause: `Failed to get quote token from GraphQL: ${error}` })
      }
    })

    const quote_token = v1_ibc_union_tokens[0]?.wrapping[0]?.unwrapped_address_hex
    if (quote_token) {
      return { type: "UNWRAPPED" as const, quote_token }
    }

    if (destinationChain.rpc_type === "cosmos") {
      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      const client = yield* getCosmosPublicClient(rpc.toString())
      const predictedQuoteToken = yield* Effect.tryPromise({
        try: () =>
          client.queryContractSmart(fromHex(channel.destination_port_id, "string"), {
            predict_wrapped_token: {
              path: "0",
              channel: channel.destination_channel_id,
              token: base_token
            }
          }),
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (Cosmos): ${error}` })
      }).pipe(Effect.map(res => res.wrapped_token as Hex))

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "evm") {
      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      const client = createPublicClient({
        chain: getChainFromWagmi(Number.parseInt(channel.destination_chain_id)),
        transport: http(rpc.toString())
      })
      const predictedQuoteToken = yield* Effect.tryPromise({
        try: () =>
          client.readContract({
            address: `0x${channel.destination_port_id}`,
            abi: ucs03ZkgmAbi,
            functionName: "predictWrappedToken",
            args: [0, channel.destination_channel_id, base_token]
          }) as Promise<[Address, string]>,
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (EVM): ${error}` })
      }).pipe(Effect.map(([address]) => address))

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "aptos") {
      console.info("destinationChain is aptos")
      let network: Network
      // let rpcUrl: string

      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      console.info("rpc: ", rpc.origin)
      if (channel.destination_chain_id === "250") {
        network = Network.TESTNET
        // rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"
      } else {
        return yield* Effect.fail(
          new GetQuoteError({ cause: `Unsupported Aptos network: ${channel.destination_chain_id}` })
        )
      }

      const config = new AptosConfig({ network, fullnode: rpc.origin + "/v1" }) //TODO: rpc.origin is coming without "/v1" at the end, discuss this later

      const aptosClient = new Aptos(config)

      const output = yield* Effect.tryPromise({
        try: () =>
          aptosClient.view({
            payload: {
              function: `${channel.destination_port_id}::ibc_app::predict_wrapped_token`,
              typeArguments: [],
              functionArguments: [
                0, // path
                channel.destination_channel_id,
                MoveVector.U8(base_token)
              ]
            }
          }),
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (Aptos): ${error}` })
      })

      const wrappedAddressHex = output[0]?.toString()
      if (!wrappedAddressHex) {
        return yield* Effect.fail(
          new GetQuoteError({ cause: "Failed to get wrapped address from Aptos" })
        )
      }
      return { type: "NEW_WRAPPED" as const, quote_token: wrappedAddressHex }
    }

    return yield* Effect.fail(
      new GetQuoteError({ cause: `${destinationChain.rpc_type} not supported` })
    )
  })
