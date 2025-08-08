/**
 * This module provides a chain-agnostic service for determining gas price.
 *
 * @since 2.0.0
 */
import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import {
  Array as A,
  BigDecimal,
  Brand,
  Context,
  Effect,
  Layer,
  LayerMap,
  Match,
  Option as O,
  pipe,
  Record as R,
  Schema as S,
  unsafeCoerce,
} from "effect"
import { type GetGasPriceErrorType, http } from "viem"
import { publicActionsL2 } from "viem/op-stack"
import * as Evm from "./Evm.js"
import { Chain } from "./schema/chain.js"

/**
 * @since 2.0.0
 * @category type ids
 */
export const TypeId: unique symbol = Symbol("@unionlabs/sdk/GasPrice/GasPriceError")

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @since 2.0.0
 * @category errors
 */
export class GasPriceError
  extends S.TaggedError<GasPriceError>("@unionlabs/sdk/GasPrice/GasPriceError")("GasPriceError", {
    module: S.String,
    method: S.String,
    description: S.String,
    cause: S.optional(S.Defect),
  })
{
  /**
   * @since 2.0.0
   */
  readonly [TypeId]: TypeId = TypeId
  /**
   * @since 2.0.0
   */
  get message(): string {
    return `[GasPriceError] ${this.module}.${this.method}: ${this.description}`
  }
}

/**
 * @category models
 * @since 2.0.0
 */
export type AtomicGasPrice = BigDecimal.BigDecimal & Brand.Brand<"AtomicGasPrice">
/**
 * @category branding
 * @since 2.0.0
 */
export const AtomicGasPrice = Brand.nominal<AtomicGasPrice>()

/**
 * @category models
 * @since 2.0.0
 */
export type BaseGasPrice = BigDecimal.BigDecimal & Brand.Brand<"BaseGasPrice">
/**
 * @category branding
 * @since 2.0.0
 */
export const BaseGasPrice = Brand.nominal<BaseGasPrice>()

/**
 * Normalized gas price.
 *
 * @category contexts
 * @since 2.0.0
 */
export class GasPrice extends Context.Tag("@unionlabs/sdk/GasPrice/GasPrice")<
  GasPrice,
  GasPrice.Service
>() {}

/**
 * @since 2.0.0
 * @category models
 */
export declare namespace GasPrice {
  /**
   * @since 0.0.1
   * @category models
   */
  export interface Service {
    readonly of: Effect.Effect<{
      value: AtomicGasPrice
      /**
       * e.g. L1 settlement fee on BOB
       */
      additiveFee: O.Option<AtomicGasPrice>
      minimalDenom: string
      denom: string
      decimals: number
    }, GasPriceError>
  }
}

/**
 * @category layer maps
 * @since 2.0.0
 */
export class GasPriceMap extends LayerMap.Service<GasPriceMap>()("GasPriceByChain", {
  provides: GasPrice,
  lookup: pipe(
    Match.type<Chain>(),
    Match.when(
      {
        rpc_type: "evm",
      },
      (chain) =>
        Layer.effect(
          GasPrice,
          Effect.gen(function*() {
            const viemChain = yield* chain.toViemChain().pipe(
              Effect.mapError((cause) =>
                new GasPriceError({
                  module: "Evm",
                  method: "chain",
                  description: "could not convert internal chain to viem chain",
                  cause,
                })
              ),
            )

            const client = pipe(
              Evm.PublicClient.Live({
                chain: viemChain,
                transport: http(),
              }),
            )

            const additiveFee = yield* pipe(
              Match.value(chain.universal_chain_id),
              Match.whenOr(
                Match.is(UniversalChainId.make("bob.60808")),
                Match.is(UniversalChainId.make("bob.808813")),
                (id) =>
                  pipe(
                    Evm.PublicClient,
                    Effect.andThen(({ client }) =>
                      pipe(
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
                        Effect.map(AtomicGasPrice),
                      )
                    ),
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
    Match.orElseAbsurd,
  ),
  idleTimeToLive: "30 seconds",
  dependencies: [],
}) {}
