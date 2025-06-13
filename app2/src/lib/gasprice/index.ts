import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { createViemPublicClient } from "@unionlabs/sdk/evm"
import type { Chain } from "@unionlabs/sdk/schema"
import {
  Array as A,
  BigDecimal,
  Effect,
  Layer,
  LayerMap,
  Match,
  Option as O,
  pipe,
  Record as R,
  unsafeCoerce,
} from "effect"
import { type GetGasPriceErrorType, http } from "viem"
import { GasPriceError } from "./error"
import * as GasPrice from "./service"

export class GasPriceMap extends LayerMap.Service<GasPriceMap>()("GasPriceByChain", {
  provides: GasPrice.GasPrice,
  lookup: pipe(
    Match.type<Chain>(),
    Match.when(
      {
        rpc_type: "evm",
      },
      (chain) =>
        Layer.effect(
          GasPrice.GasPrice,
          Effect.gen(function*() {
            const client = yield* pipe(
              chain.toViemChain(),
              Effect.flatMap((chain) =>
                createViemPublicClient({
                  chain,
                  transport: http(),
                })
              ),
              Effect.mapError((cause) =>
                new GasPriceError({
                  module: "Evm",
                  method: "of",
                  description: "Could not create public client.",
                  cause,
                })
              ),
            )

            const of = pipe(
              Effect.tryPromise({
                try: () => client.getGasPrice(),
                catch: (cause) =>
                  new GasPriceError({
                    module: "Evm",
                    method: "of",
                    description: `Could not read gas price on-chain for ${chain.display_name}.`,
                    cause: unsafeCoerce<unknown, GetGasPriceErrorType>(cause),
                  }),
              }),
              // XXX: take from constants file
              Effect.tap((x) => Effect.log(`${chain.display_name} gas price (wei): ${x}`)),
              Effect.map((a) => ({
                value: GasPrice.BaseGasPrice(BigDecimal.make(a, 18)),
                decimals: 18,
              })),
              Effect.tap((x) => Effect.log(`${chain.display_name} gas price (ETH): ${x}`)),
            )

            return GasPrice.GasPrice.of({
              of,
            })
          }),
        ),
    ),
    Match.when(
      {
        rpc_type: "cosmos",
      },
      (chain) =>
        Layer.effect(
          GasPrice.GasPrice,
          Effect.gen(function*() {
            const of = Effect.gen(function*() {
              const config = yield* R.get(chainInfoMap, chain.chain_id).pipe(
                Effect.mapError(() =>
                  new GasPriceError({
                    module: "Cosmos",
                    method: "of",
                    description: `No chain configured with identifier ${chain.chain_id}.`,
                  })
                ),
              )

              const avg = yield* pipe(
                config.feeCurrencies,
                A.head, // XXX: how to handle multiple?
                O.flatMap(x =>
                  O.Do.pipe(
                    O.bind("average", () =>
                      pipe(
                        O.fromNullable(x.gasPriceStep),
                        O.map(x => x.average),
                      )),
                    O.let("decimals", () => x.coinDecimals),
                    O.map(({ average, decimals }) =>
                      pipe(
                        BigDecimal.unsafeFromNumber(average),
                        BigDecimal.multiply(BigDecimal.make(1n, decimals)),
                        GasPrice.BaseGasPrice,
                        (value) => ({
                          value,
                          decimals,
                        }),
                      )
                    ),
                  )
                ),
                Effect.tap((x) => Effect.log(`${chain.display_name} gas price (?): ${x}`)),
                Effect.mapError(() =>
                  new GasPriceError({
                    module: "Cosmos",
                    method: "of",
                    description: `No chain configured with identifier ${chain.chain_id}.`,
                  })
                ),
              )

              return avg
            })

            return GasPrice.GasPrice.of({
              of,
            })
          }),
        ),
    ),
    Match.orElseAbsurd,
  ),
  idleTimeToLive: "30 seconds",
  dependencies: [],
}) {}
