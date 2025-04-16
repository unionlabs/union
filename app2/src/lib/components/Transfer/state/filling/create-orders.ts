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
import type { TransferIntent } from "$lib/components/Transfer/state/filling/create-intents.ts"

export function createOrdersBatch(
  intent: TransferIntent
): Effect.Effect<Option.Option<Instruction>, OrderCreationError> {
  return Effect.gen(function* () {
    if (intent.contexts.length === 0) {
      return Option.none<Instruction>()
    }

    const grouped = Arr.groupBy(
      intent.contexts,
      ctx =>
        `${ctx.sourceChainId}-${ctx.destinationChain.universal_chain_id}-${ctx.channel.destination_channel_id}-${ctx.ucs03address}`
    )

    // We only support one group per batch
    const firstGroup = Object.values(grouped)[0]
    const [first] = firstGroup

    const decodeIntent = Schema.decode(FungibleIntent.AssetOrderIntentFromTransferIntent, {
      errors: "all",
      onExcessProperty: "ignore"
    })

    const newIntents = firstGroup.map(ctx =>
      decodeIntent({
        sender: ctx.sender,
        receiver: ctx.receiver,
        baseToken: ctx.baseToken,
        baseAmount: ctx.baseAmount,
        quoteAmount: ctx.quoteAmount,
        sourceChainId: ctx.sourceChainId,
        sourceChannelId: ctx.sourceChannelId,
        sourceChain: ctx.sourceChain,
        destinationChain: ctx.destinationChain
      })
    )

    const resolvedIntents = yield* Effect.all(newIntents, { concurrency: "unbounded" })

    const provideClients = yield* Match.value([
      first.sourceChain.rpc_type,
      first.destinationChain.rpc_type
    ]).pipe(
      Match.when(["evm", "cosmos"], () =>
        Effect.all(resolvedIntents.map(createEvmToCosmosFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            ViemPublicClientSource,
            pipe(
              first.sourceChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmWasmClientDestination,
            pipe(
              first.destinationChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmosChannelDestination,
            Effect.succeed({
              ucs03address: fromHex(first.channel.destination_port_id, "string"),
              channelId: first.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["evm", "evm"], () =>
        Effect.all(resolvedIntents.map(createEvmToEvmFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            ViemPublicClientSource,
            pipe(
              first.sourceChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            ViemPublicClientDestination,
            pipe(
              first.destinationChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            EvmChannelDestination,
            Effect.succeed({
              ucs03address: first.channel.destination_port_id,
              channelId: first.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["cosmos", "evm"], () =>
        Effect.all(resolvedIntents.map(createCosmosToEvmFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            CosmWasmClientSource,
            pipe(
              first.sourceChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            ViemPublicClientDestination,
            pipe(
              first.destinationChain.toViemChain(),
              Option.map(chain => createViemPublicClient({ chain, transport: http() })),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            EvmChannelDestination,
            Effect.succeed({
              ucs03address: first.channel.destination_port_id,
              channelId: first.channel.destination_channel_id
            })
          )
        )
      ),
      Match.when(["cosmos", "cosmos"], () =>
        Effect.all(resolvedIntents.map(createCosmosToCosmosFungibleAssetOrder)).pipe(
          Effect.provideServiceEffect(
            CosmWasmClientSource,
            pipe(
              first.sourceChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmWasmClientDestination,
            pipe(
              first.destinationChain.getRpcUrl("rpc"),
              Option.map(createCosmWasmClient),
              Effect.flatten,
              Effect.map(client => ({ client }))
            )
          ),
          Effect.provideServiceEffect(
            CosmosChannelDestination,
            Effect.succeed({
              ucs03address: fromHex(first.channel.destination_port_id, "string"),
              channelId: first.channel.destination_channel_id
            })
          )
        )
      ),
      Match.orElse(() =>
        Effect.fail(
          new OrderCreationError({
            details: {
              reason: `Unsupported combo: ${first.sourceChain.rpc_type} -> ${first.destinationChain.rpc_type}`
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
          details: { error, intents: intent.contexts.length }
        })
      )
    )
  )
}
