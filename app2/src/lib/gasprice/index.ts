import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { createViemPublicClient } from "@unionlabs/sdk/evm"
import type { Chain } from "@unionlabs/sdk/schema"
import {
  Array as A,
  Effect,
  flow,
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
        rpc_type: "cosmos",
      },
      (chain) =>
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
              A.head,
              O.flatMap(flow(x => x.gasPriceStep, O.fromNullable)),
              O.map(x => x.average),
              Effect.mapError(() =>
                new GasPriceError({
                  module: "Cosmos",
                  method: "of",
                  description: `No chain configured with identifier ${chain.chain_id}.`,
                })
              ),
            )

            return BigInt(avg)
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
  idleTimeToLive: "5 seconds", // XXX: ???
  dependencies: [],
}) {}
