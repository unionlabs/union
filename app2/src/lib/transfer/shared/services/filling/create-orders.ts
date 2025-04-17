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
import { OrderCreationError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"

export function createOrdersBatch(
  context: TransferContext
): Effect.Effect<Option.Option<Instruction>, OrderCreationError> {
  return Effect.gen(function* () {
    if (context.intents.length === 0) {
      return Option.none<Instruction>()
    }

    const grouped = Arr.groupBy(
      context.intents,
      intents =>
        `${intents.sourceChainId}-${intents.destinationChain.universal_chain_id}-${intents.channel.destination_channel_id}-${intents.ucs03address}`
    )

    // We only support one group per batch
    const firstGroup = Object.values(grouped)[0]
    const [first] = firstGroup

    const decodeIntent = Schema.decode(FungibleIntent.AssetOrderIntentFromTransferIntent, {
      errors: "all",
      onExcessProperty: "ignore"
    })

    const newIntents = firstGroup.map(intent =>
      decodeIntent({
        sender: intent.sender,
        receiver: intent.receiver,
        baseToken: intent.baseToken,
        baseAmount: intent.baseAmount,
        quoteAmount: intent.quoteAmount,
        sourceChainId: intent.sourceChainId,
        sourceChannelId: intent.sourceChannelId,
        sourceChain: intent.sourceChain,
        destinationChain: intent.destinationChain
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
          details: { error, intents: context.intents.length }
        })
      )
    )
  )
}
