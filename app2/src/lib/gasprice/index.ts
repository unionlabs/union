import { FetchHttpClient } from "@effect/platform"
import { createViemPublicClient, ViemPublicClient } from "@unionlabs/sdk/evm"
import type { Chain, UniversalChainId } from "@unionlabs/sdk/schema"
import { Config, Effect, flow, identity, Layer, LayerMap, Match, pipe, unsafeCoerce } from "effect"
import { type GetGasPriceErrorType, http } from "viem"
import { GasPriceError } from "./error"
import * as GasPrice from "./service"

// create the openai client layer
// const OpenAiLayer = OpenAiClient.layerConfig({
//   apiKey: Config.redacted("OPENAI_API_KEY"),
// }).pipe(Layer.provide(FetchHttpClient.layer))

// create a service that wraps a LayerMap
class GasPriceMap extends LayerMap.Service<GasPriceMap>()("GasPriceByChain", {
  // this LayerMap will provide the ai Completions service
  provides: GasPrice.GasPrice,

  // define the lookup function for the layer map
  //
  // The returned Layer will be used to provide the Completions service for the
  // given model.
  // lookup: (model: "evm" | "cosmos") => OpenAiCompletions.layer({ model }),
  lookup: pipe(
    Match.type<Chain>(),
    Match.when(
      {
        rpc_type: "evm",
      },
      (chain) =>
        Effect.gen(function*() {
          const client = yield* pipe(
            chain.toViemChain(),
            Effect.flatMap((chain) =>
              createViemPublicClient({
                chain,
                transport: http(),
              })
            ),
          )

          const of = Effect.tryPromise({
            try: () => client.getGasPrice(),
            catch: (cause) =>
              new GasPriceError({
                module: "Evm",
                method: "of",
                description: "some",
                cause: unsafeCoerce<unknown, GetGasPriceErrorType>(cause),
              }),
          })

          return GasPrice.GasPrice.of({
            of,
          })
        }).pipe(
          e => Layer.effect(GasPrice.GasPrice, e),
        ),
    ),
    Match.when(
      {
        rpc_type: "evm",
      },
      (chain) =>
        Effect.gen(function*() {
          const client = yield* pipe(
            chain.toViemChain(),
            Effect.flatMap((chain) =>
              createViemPublicClient({
                chain,
                transport: http(),
              })
            ),
          )

          const of = Effect.tryPromise({
            try: () => client.getGasPrice(),
            catch: (cause) =>
              new GasPriceError({
                module: "Evm",
                method: "of",
                description: "some",
                cause: unsafeCoerce<unknown, GetGasPriceErrorType>(cause),
              }),
          })

          return GasPrice.GasPrice.of({
            of,
          })
        }).pipe(
          e => Layer.effect(GasPrice.GasPrice, e),
        ),
    ),
    Match.orElseAbsurd,
  ),

  // layers: {
  //   evm: Layer.effect(
  //     GasPrice.GasPrice,
  //     Effect.gen(function*() {
  //       const viemChain = yield* step.intent.sourceChain.toViemChain()
  //       const publicClient = yield* createViemPublicClient({
  //         chain: viemChain,
  //         transport: http(),
  //       })

  //       return GasPrice.GasPrice.of({
  //         of: () =>
  //           pipe(
  //             Effect.tryPromise({
  //               try: () => client.getGasPrice(),
  //               catch: (cause) =>
  //                 new GasPriceError({
  //                   module: "Evm",
  //                   method: "of",
  //                   description: "some",
  //                   cause: unsafeCoerce<unknown, GetGasPriceErrorType>(cause),
  //                 }),
  //             }),
  //           ),
  //       })
  //     }),
  //   ),
  //   cosmos: Layer.effect(
  //     GasPrice.GasPrice,
  //     Effect.gen(function*() {
  //       return GasPrice.GasPrice.of({
  //         of: () => Effect.succeed(0n),
  //       })
  //     }),
  //   ),
  // },

  // If a layer is not used for a certain amount of time, it can be removed
  idleTimeToLive: "5 seconds",

  // Supply the dependencies for the layers in the LayerMap
  dependencies: [],
}) {}

// usage
const b = Effect.gen(function*() {
  const gasPrice = yield* GasPrice.GasPrice
  const response = yield* gasPrice.of
  console.log(response)
}).pipe(
  // use the AiClients service to provide a variant of the Completions service
  GasPriceMap.provide(void 0 as unknown as Chain),
  GasPriceMap.provide(void 0 as unknown as Chain),
  // provide the LayerMap service
  Effect.provide(GasPriceMap.Default),
)
