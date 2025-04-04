import { Effect, Match, Option, pipe } from "effect";
import { fromHex, http } from "viem";
import {
  createViemPublicClient,
  ViemPublicClientSource,
  ViemPublicClientDestination,
  EvmChannelDestination
} from "@unionlabs/sdk/evm";
import {
  createCosmWasmClient,
  CosmWasmClientSource,
  CosmWasmClientDestination,
  CosmosChannelDestination
} from "@unionlabs/sdk/cosmos";
import {
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createCosmosToCosmosFungibleAssetOrder
} from "@unionlabs/sdk/ucs03";
import { Batch } from "@unionlabs/sdk/ucs03/instruction";
import {Channel, Chain} from "@unionlabs/sdk/schema";
import {transfer} from "$lib/components/Transfer/transfer.svelte.ts";

export function createOrdersBatch(
  sourceChain: Chain,
  destinationChain: Chain,
  channel: Channel,
  ucs03address: string,
  intents: typeof transfer.intents
) {
  return Effect.gen(function* () {
    if (!sourceChain || !destinationChain || !channel || !ucs03address || !intents?.length) {
      return Option.none<Batch>();
    }

    const source = sourceChain.rpc_type;
    const destination = destinationChain.rpc_type;

    // Provide viem public client for the source chain
    const provideViemPublicClientSource = Effect.provideServiceEffect(
      ViemPublicClientSource,
      pipe(
        sourceChain.toViemChain(),
        Option.map(chain =>
          createViemPublicClient({
            chain,
            transport: http()
          })
        ),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    );

    // Provide viem public client for the destination chain
    const provideViemPublicClientDestination = Effect.provideServiceEffect(
      ViemPublicClientDestination,
      pipe(
        destinationChain.toViemChain(),
        Option.map(chain =>
          createViemPublicClient({
            chain,
            transport: http()
          })
        ),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    );

    // Provide CosmWasm client for the source chain
    const provideCosmWasmClientSource = Effect.provideServiceEffect(
      CosmWasmClientSource,
      pipe(
        sourceChain.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    );

    // Provide CosmWasm client for the destination chain
    const provideCosmWasmClientDestination = Effect.provideServiceEffect(
      CosmWasmClientDestination,
      pipe(
        destinationChain.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    );

    // Provide channel addresses
    const provideEvmChannelDestination = Effect.provideServiceEffect(EvmChannelDestination, {
      ucs03address: channel.source_port_id,
      channelId: channel.source_channel_id
    });

    const provideCosmosChannelDestination = Effect.provideServiceEffect(CosmosChannelDestination, {
      ucs03address: fromHex(channel.destination_port_id, "string"),
      channelId: channel.destination_channel_id
    });

    // Build the orders (Batch) based on source/destination
    const batchEffect = Effect.gen(function* () {
      const orders = yield* Match.value([source, destination]).pipe(
        Match.when(["evm", "cosmos"], () =>
          // Example: EVM -> Cosmos with two intents
          Effect.all([
            Effect.tap(createEvmToCosmosFungibleAssetOrder(intents[0]), createdOrder =>
              Effect.sync(() => console.log("First EVM->Cosmos order created", createdOrder))
            ),
            Effect.tap(createEvmToCosmosFungibleAssetOrder(intents[1]), createdOrder =>
              Effect.sync(() => console.log("Second EVM->Cosmos order created", createdOrder))
            )
          ]).pipe(
            Effect.tap(createdOrders =>
              Effect.sync(() => console.log("All EVM->Cosmos orders created", createdOrders))
            ),
            Effect.catchAll(error => {
              console.error("Error creating EVM->Cosmos orders", error.cause);
              return Effect.fail(error);
            }),
            // Service injection for cross-chain creation
            provideCosmosChannelDestination,
            provideViemPublicClientSource,
            provideCosmWasmClientDestination
          )
        ),

        Match.when(["evm", "evm"], () =>
          // EVM -> EVM
          Effect.all([
            createEvmToEvmFungibleAssetOrder(intents[0]),
            createEvmToEvmFungibleAssetOrder(intents[1])
          ]).pipe(
            Effect.tap(createdOrders =>
              Effect.sync(() => console.log("EVM->EVM orders created", createdOrders))
            ),
            Effect.catchAll(error => {
              console.error("Error creating EVM->EVM orders", error.cause);
              return Effect.fail(error);
            }),
            provideViemPublicClientSource,
            provideViemPublicClientDestination,
            provideEvmChannelDestination
          )
        ),

        Match.when(["cosmos", "evm"], () =>
          // Cosmos -> EVM
          createCosmosToEvmFungibleAssetOrder(intents[0]).pipe(
            Effect.tap(createdOrder =>
              Effect.sync(() => console.log("Cosmos->EVM order created", createdOrder))
            ),
            Effect.catchAll(error => {
              console.error("Error creating Cosmos->EVM order", error);
              return Effect.fail(error);
            }),
            provideCosmWasmClientSource,
            provideViemPublicClientDestination,
            provideEvmChannelDestination
          )
        ),

        Match.when(["cosmos", "cosmos"], () =>
          // Cosmos -> Cosmos
          createCosmosToCosmosFungibleAssetOrder(intents[0]).pipe(
            Effect.tap(createdOrder =>
              Effect.sync(() => console.log("Cosmos->Cosmos order created", createdOrder))
            ),
            Effect.catchAll(error => {
              console.error("Error creating Cosmos->Cosmos order", error.cause);
              return Effect.fail(error);
            }),
            provideCosmWasmClientSource,
            provideCosmWasmClientDestination,
            provideCosmosChannelDestination
          )
        ),

        Match.orElse(() => {
          // Fallback if no combination matched
          console.warn(`Unsupported source->destination: ${source} -> ${destination}`);
          throw new Error(`Unsupported chain combo: ${source} -> ${destination}`);
        })
      );

      // Return a new Batch, wrapping single or multiple orders
      return new Batch({
        operand: Array.isArray(orders) ? orders : [orders]
      });
    });

    const batchResult = yield* batchEffect;
    return Option.some(batchResult);
  });
}
