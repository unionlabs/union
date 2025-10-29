import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { fallbackTransport } from "$lib/wallet/evm/wagmi-config.svelte"
import { createViemPublicClient } from "@unionlabs/sdk/evm"
import { type Chain, UniversalChainId } from "@unionlabs/sdk/schema"
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
import { type GetGasPriceErrorType } from "viem"
import { publicActionsL2 } from "viem/op-stack"
import { GasPriceError } from "./error"
import * as GasPrice from "./service"

export class GasPriceMap extends LayerMap.Service<GasPriceMap>()("GasPriceByChain", {
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
            const viemChain = yield* chain.toViemChain().pipe(
              Effect.mapError((cause) =>
                new GasPriceError({
                  module: "Evm",
                  method: "chain",
                  description: "Could not convert internal chain to viem chain",
                  cause,
                })
              ),
            )

            const client = yield* pipe(
              createViemPublicClient({
                chain: viemChain,
                transport: fallbackTransport(viemChain),
              }),
              Effect.mapError((cause) =>
                new GasPriceError({
                  module: "Evm",
                  method: "of",
                  description: "Could not create public client",
                  cause,
                })
              ),
            )

            const additiveFee = yield* pipe(
              Match.value(chain.universal_chain_id),
              Match.whenOr(
                Match.is(UniversalChainId.make("bob.60808")),
                Match.is(UniversalChainId.make("bob.808813")),
                (id) =>
                  pipe(
                    client.extend(publicActionsL2()),
                    (client) =>
                      Effect.tryPromise({
                        try: () =>
                          client.estimateL1Fee({
                            // TODO: re-evaluate correctness
                            account: "0x0000000000000000000000000000000000000000",
                            chain: undefined,
                          }),
                        catch: (cause) =>
                          new GasPriceError({
                            module: "Evm",
                            method: "additiveFee",
                            description: `Could not calculate L1 fee for ${id}`,
                            cause,
                          }),
                      }),
                    Effect.map((atomic) => BigDecimal.make(atomic, 0)),
                    Effect.map(GasPrice.AtomicGasPrice),
                  ),
              ),
              Match.option,
              Effect.transposeOption,
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
              Effect.tap((x) =>
                Effect.logDebug(
                  `${chain.display_name} gas price (atomic): ${JSON.stringify(x)}`,
                )
              ),
              Effect.map((a) => ({
                value: GasPrice.AtomicGasPrice(BigDecimal.fromBigInt(a)),
                minimalDenom: "wei",
                denom: viemChain.nativeCurrency.symbol,
                additiveFee,
                decimals: 18,
              })),
              Effect.tap((x) =>
                Effect.logDebug(`${chain.display_name} gas price (ETH): ${JSON.stringify(x)}`)
              ),
              Effect.tapError((cause) => Effect.logError("GasPrice.of", cause)),
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
                    O.let("minimalDenom", () => x.coinMinimalDenom),
                    O.let("denom", () => x.coinDenom),
                    O.map(({ average, decimals, minimalDenom, denom }) =>
                      pipe(
                        BigDecimal.unsafeFromNumber(average),
                        GasPrice.AtomicGasPrice,
                        (value) => ({
                          value,
                          minimalDenom,
                          denom,
                          additiveFee: O.none<GasPrice.AtomicGasPrice>(),
                          decimals,
                        }),
                      )
                    ),
                  )
                ),
                Effect.tap((x) =>
                  Effect.logDebug(`${chain.display_name} gas price (?): ${JSON.stringify(x)}`)
                ),
                Effect.mapError(() =>
                  new GasPriceError({
                    module: "Cosmos",
                    method: "of",
                    description: `No chain configured with identifier ${chain.chain_id}.`,
                  })
                ),
                Effect.tapError((cause) => Effect.logError("GasPrice.of", cause)),
              )

              return avg
            })

            return GasPrice.GasPrice.of({
              of,
            })
          }),
        ),
    ),
    Match.when(
    { rpc_type: "sui" },
    (chain) =>
      Layer.effect(
        GasPrice.GasPrice,
        Effect.gen(function* () {
          // ── MOCKED SUI GAS PRICE ───────────────────────────────────────────── 
          // TODO: Change it later.
          const of = Effect.gen(function* () {
            const atomic = BigDecimal.unsafeFromNumber(1_000)
            const value  = GasPrice.AtomicGasPrice(atomic)

            yield* Effect.logDebug(
              `${chain.display_name} gas price (mock MIST): ${JSON.stringify(atomic)}`
            )

            return {
              value,                   
              minimalDenom: "MIST",     
              denom: "SUI",             
              additiveFee: O.none(),    
              decimals: 9,              
            }
          }).pipe(Effect.tapError((cause) => Effect.logError("GasPrice.of (sui)", cause)))

          return GasPrice.GasPrice.of({ of })
        }),
      ),
  ),

    Match.orElseAbsurd,
  ),
  idleTimeToLive: "30 seconds",
  dependencies: [],
}) {}
