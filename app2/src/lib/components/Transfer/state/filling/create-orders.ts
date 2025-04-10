import { Effect, Match, Option, pipe, Schema } from "effect"
import { fromHex, http } from "viem"
import {
  createViemPublicClient,
  ViemPublicClientSource,
  ViemPublicClientDestination,
  EvmChannelDestination
} from "@unionlabs/sdk/evm"
import {
  createCosmWasmClient,
  CosmWasmClientSource,
  CosmWasmClientDestination,
  CosmosChannelDestination
} from "@unionlabs/sdk/cosmos"
import {
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createCosmosToCosmosFungibleAssetOrder
} from "@unionlabs/sdk/ucs03"
import { Batch } from "@unionlabs/sdk/ucs03/instruction"
import { type Channel, type Chain, FungibleIntent } from "@unionlabs/sdk/schema"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { OrderCreationError } from "$lib/components/Transfer/state/errors.ts"

export function createOrdersBatch(
  sourceChain: Chain,
  destinationChain: Chain,
  channel: Channel,
  ucs03address: string,
  intents: TransferIntents
) {
  return Effect.gen(function* () {
    if (!(sourceChain && destinationChain && channel && ucs03address) || intents.length === 0) {
      console.log("lukas: Missing required params or no intents → returning Option.none")
      return Option.none<Batch>()
    }

    const source = sourceChain.rpc_type
    const destination = destinationChain.rpc_type

    const provideViemPublicClientSource = Effect.provideServiceEffect(
      ViemPublicClientSource,
      pipe(
        sourceChain.toViemChain(),
        Option.map(chain => createViemPublicClient({ chain, transport: http() })),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const provideViemPublicClientDestination =
      destination === "evm"
        ? Effect.provideServiceEffect(
            ViemPublicClientDestination,
            pipe(
              destinationChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          )
        : Effect.succeed({})

    const provideCosmWasmClientSource = Effect.provideServiceEffect(
      CosmWasmClientSource,
      pipe(
        sourceChain.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const provideCosmWasmClientDestination = Effect.provideServiceEffect(
      CosmWasmClientDestination,
      pipe(
        destinationChain.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const evmChannelDestinationData = {
      ucs03address: channel.destination_port_id,
      channelId: channel.destination_channel_id
    }
    const cosmosChannelDestinationData = {
      ucs03address: fromHex(channel.destination_port_id, "string"),
      channelId: channel.destination_channel_id
    }

    const provideEvmChannelDestination = Effect.provideServiceEffect(
      EvmChannelDestination,
      Effect.succeed(evmChannelDestinationData)
    )
    const provideCosmosChannelDestination = Effect.provideServiceEffect(
      CosmosChannelDestination,
      Effect.succeed(cosmosChannelDestinationData)
    )

    const intentsInput = intents.map(x => ({ ...x, sourceChain, destinationChain, ucs03address }))
    const newIntents = intentsInput.map(
      Schema.decode(FungibleIntent.AssetOrderIntentFromTransferIntent, {
        errors: "all",
        onExcessProperty: "ignore"
      })
    )
    const resolvedIntents = yield* Effect.all(newIntents, { concurrency: "unbounded" })

    const orders = yield* Match.value([source, destination]).pipe(
      Match.when(["evm", "cosmos"], () =>
        Effect.all([
          createEvmToCosmosFungibleAssetOrder(resolvedIntents[0]),
          intents.length > 1
            ? createEvmToCosmosFungibleAssetOrder(resolvedIntents[1])
            : Effect.succeed(null)
        ]).pipe(
          provideViemPublicClientSource,
          provideCosmWasmClientDestination,
          provideCosmosChannelDestination
        )
      ),
      Match.when(["evm", "evm"], () =>
        Effect.all([
          createEvmToEvmFungibleAssetOrder(resolvedIntents[0]),
          intents.length > 1
            ? createEvmToEvmFungibleAssetOrder(resolvedIntents[1])
            : Effect.succeed(null)
        ]).pipe(
          provideViemPublicClientSource,
          provideViemPublicClientDestination,
          provideEvmChannelDestination
        )
      ),
      Match.when(["cosmos", "evm"], () =>
        Effect.all([createCosmosToEvmFungibleAssetOrder(resolvedIntents[0])]).pipe(
          provideCosmWasmClientSource,
          provideViemPublicClientDestination,
          provideEvmChannelDestination
        )
      ),
      Match.when(["cosmos", "cosmos"], () =>
        Effect.all([createCosmosToCosmosFungibleAssetOrder(resolvedIntents[0])]).pipe(
          provideCosmWasmClientSource,
          provideCosmWasmClientDestination,
          provideCosmosChannelDestination
        )
      ),
      Match.orElse(() =>
        Effect.fail(
          new OrderCreationError({
            details: { reason: `Unsupported combo: ${source} -> ${destination}` }
          })
        )
      )
    )

    const filtered = orders.filter((o): o is NonNullable<typeof o> => o !== null)
    return Option.some(new Batch({ operand: filtered }))
  }).pipe(
    Effect.catchAll(error =>
      Effect.fail(
        new OrderCreationError({
          details: {
            error,
            source: sourceChain.display_name,
            destination: destinationChain.display_name,
            channel,
            intents: intents.length
          }
        })
      )
    )
  )
}
