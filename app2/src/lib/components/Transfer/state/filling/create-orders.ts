import { Effect, Match, Option, pipe, Schema, Array as Arr } from "effect"
import { fromHex, http } from "viem"
import {
  createViemPublicClient,
  EvmChannelDestination,
  ViemPublicClientDestination,
  ViemPublicClientSource
} from "@unionlabs/sdk/evm"
import {
  CosmosChannelDestination,
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient
} from "@unionlabs/sdk/cosmos"
import {
  createCosmosToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder
} from "@unionlabs/sdk/ucs03"
import { Batch, type Instruction } from "@unionlabs/sdk/ucs03/instruction"
import { FungibleIntent } from "@unionlabs/sdk/schema"
import { OrderCreationError } from "$lib/components/Transfer/state/errors.ts"
import type { TransferIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"

export function createOrdersBatch(intents: TransferIntents) {
  return Effect.gen(function* () {
    if (intents.length === 0) {
      return Option.none<Instruction>()
    }

    const grouped = Arr.groupBy(
      intents,
      intent =>
        `${intent.context.sourceChainId}-${intent.context.destinationChain.universal_chain_id}-${intent.context.channel.destination_channel_id}-${intent.context.ucs03address}`
    )

    // You only support 1 batch group at a time
    const firstGroup = Object.values(grouped)[0]
    const [{ context }] = firstGroup

    const decodeIntent = Schema.decode(FungibleIntent.AssetOrderIntentFromTransferIntent, {
      errors: "all",
      onExcessProperty: "ignore"
    })

    const newIntents = firstGroup.map(x =>
      decodeIntent({
        sender: x.context.sender,
        receiver: x.context.receiver,
        baseToken: x.context.baseToken,
        baseAmount: x.context.baseAmount,
        quoteAmount: x.context.quoteAmount,
        sourceChainId: x.context.sourceChainId,
        sourceChannelId: x.context.sourceChannelId,
        sourceChain: x.context.sourceChain,
        destinationChain: x.context.destinationChain
      })
    )

    const resolvedIntents = yield* Effect.all(newIntents, { concurrency: "unbounded" })

    const provideClients = yield* Match.value([
      context.sourceChain.rpc_type,
      context.destinationChain.rpc_type
    ]).pipe(
      Match.when(["evm", "cosmos"], () =>
        Effect.all(resolvedIntents.map(createEvmToCosmosFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            ViemPublicClientSource,
            pipe(
              context.sourceChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmWasmClientDestination,
            pipe(
              context.destinationChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmosChannelDestination,
            Effect.succeed({
              ucs03address: fromHex(context.channel.destination_port_id, "string"),
              channelId: context.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["evm", "evm"], () =>
        Effect.all(resolvedIntents.map(createEvmToEvmFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            ViemPublicClientSource,
            pipe(
              context.sourceChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            ViemPublicClientDestination,
            pipe(
              context.destinationChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            EvmChannelDestination,
            Effect.succeed({
              ucs03address: context.channel.destination_port_id,
              channelId: context.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["cosmos", "evm"], () =>
        Effect.all(resolvedIntents.map(createCosmosToEvmFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            CosmWasmClientSource,
            pipe(
              context.sourceChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            ViemPublicClientDestination,
            pipe(
              context.destinationChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            EvmChannelDestination,
            Effect.succeed({
              ucs03address: context.channel.destination_port_id,
              channelId: context.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["cosmos", "cosmos"], () =>
        Effect.all(resolvedIntents.map(createCosmosToCosmosFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            CosmWasmClientSource,
            pipe(
              context.sourceChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmWasmClientDestination,
            pipe(
              context.destinationChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmosChannelDestination,
            Effect.succeed({
              ucs03address: fromHex(context.channel.destination_port_id, "string"),
              channelId: context.channel.destination_channel_id
            })
          )
        )
      ),
      Match.orElse(() =>
        Effect.fail(
          new OrderCreationError({
            details: {
              reason: `Unsupported combo: ${context.sourceChain.rpc_type} -> ${context.destinationChain.rpc_type}`
            }
          })
        )
      )
    )

    const filtered = provideClients.filter((o): o is NonNullable<typeof o> => o !== null)

    return Option.some(new Batch({ operand: filtered }))
  }).pipe(
    Effect.catchAll(error =>
      Effect.fail(
        new OrderCreationError({
          details: { error, intents: intents.length }
        })
      )
    )
  )
}
