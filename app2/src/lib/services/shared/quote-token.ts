import { Effect, Schedule } from "effect"
import type { Hex } from "viem"
import { type Address, fromHex } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import type { Chain, Channel, TokenRawDenom } from "@unionlabs/sdk/schema"
import { getCosmosPublicClient } from "$lib/services/cosmos/clients.ts"
import { tokenWrappingQuery } from "$lib/queries/tokens.svelte.ts"
import { GetQuoteError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { MoveVector } from "@aptos-labs/ts-sdk"
import { getPublicClient as getAptosClient } from "$lib/services/aptos/clients"
import { getPublicClient } from "../evm/clients.ts"

const retryPolicy = Schedule.recurs(2).pipe(
  Schedule.compose(Schedule.exponential(200)),
  Schedule.compose(Schedule.spaced(500))
)

export const getQuoteToken = (
  sourceChain: Chain,
  base_token: TokenRawDenom,
  channel: Channel,
  destinationChain: Chain
) =>
  Effect.gen(function* () {
    // TODO: make safer
    const { v2_tokens } = yield* tokenWrappingQuery({
      denom: base_token,
      universal_chain_id: sourceChain.universal_chain_id
    })

    const quote_token = v2_tokens[0]?.wrapping[0]?.unwrapped_denom
    if (quote_token) {
      return { type: "UNWRAPPED" as const, quote_token }
    }

    if (destinationChain.rpc_type === "cosmos") {
      const rpc = yield* destinationChain.requireRpcUrl("rpc")

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
      }).pipe(
        Effect.map(res => res.wrapped_token as Hex),
        Effect.retry(retryPolicy)
      )

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "evm") {
      const client = yield* getPublicClient(destinationChain)

      const predictedQuoteToken = yield* Effect.tryPromise({
        try: () =>
          client.readContract({
            address: channel.destination_port_id,
            abi: ucs03ZkgmAbi,
            functionName: "predictWrappedToken",
            args: [0n, channel.destination_channel_id, base_token]
          }) as Promise<[Address, string]>,
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (EVM): ${error}` })
      }).pipe(
        Effect.map(([address]) => address),
        Effect.retry(retryPolicy)
      )

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "aptos") {
      const aptosClient = yield* getAptosClient(destinationChain)

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
      }).pipe(Effect.retry(retryPolicy))

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
