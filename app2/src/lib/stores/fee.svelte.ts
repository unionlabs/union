import { GasPriceMap } from "$lib/gasprice"
import type { GasPriceError } from "$lib/gasprice/error"
import { AtomicGasPrice, BaseGasPrice, GasPrice } from "$lib/gasprice/service"
import * as AppRuntime from "$lib/runtime"
import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { transferData as TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import type { Intent } from "$lib/transfer/shared/services/filling/create-context"
import * as Writer from "$lib/typeclass/Writer.js"
import type { RunPromiseExitResult } from "$lib/utils/effect.svelte"
import * as ArrayInstances from "@effect/typeclass/data/Array"
import * as FlatMap from "@effect/typeclass/FlatMap"
import * as Of from "@effect/typeclass/Of"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms"
import { VIEM_CHAINS } from "@unionlabs/sdk/constants/viem-chains"
import { PriceError, PriceOracle, PriceOraclePlan, PriceSource } from "@unionlabs/sdk/PriceOracle"
import { Chain, TokenRawAmount } from "@unionlabs/sdk/schema"
import { type Fees, GasFee } from "@unionlabs/sdk/schema/fee"
import {
  Array as A,
  BigDecimal,
  BigInt as BI,
  Cause,
  Effect,
  Either as E,
  Exit,
  Match,
  Option as O,
  pipe,
  Predicate,
  Record as R,
  Struct,
  Tuple,
  Unify,
} from "effect"
import { constant, flow, identity } from "effect/Function"

const LogWriter = Writer.getMonad(ArrayInstances.getMonoid<string>())
const composeK = FlatMap.composeK(LogWriter)

type BaseFees = Omit<
  { [K in keyof Fees]: Fees[K] extends O.Option<infer T> ? T : never },
  "PACKET_SEND_LC_UPDATE_L2"
>

export type FeeIntent = Pick<
  Intent,
  "decimals" | "baseToken" | "quoteAmount" | "baseAmount"
>

const usdOfChainGas = Effect.fn((chain: Chain) =>
  Effect.andThen(PriceOracle, (oracle) => oracle.of(chain.universal_chain_id))
)

const gasForChain = Effect.fn((chain: Chain) =>
  pipe(
    Effect.andThen(GasPrice, (gas) => gas.of),
    Effect.provide(GasPriceMap.get(chain)),
  )
)

/**
 * Store containing transfer fee data for a given channel.
 *
 * NOTE:
 * - Fees are optional to represent presence of base data.
 * - USD is derived from fee (if ready)
 * - Conversion rate is detremined optionally if USD is available for source and destination.
 *
 * TODO:
 * - How to represent loading? (only show loading for side-effecting)
 */
const createFeeStore = () => {
  const config = {
    feeMultiplier: BigDecimal.make(12n, 1), // Union hardcoded fee
    batchDivideNumber: BigDecimal.make(2n, 0), // Api?
  } as const

  let usdPrices!: RunPromiseExitResult<
    R.ReadonlyRecord<
      "source" | "destination",
      O.Option<Effect.Effect.Success<ReturnType<PriceOracle["of"]>>>
    >,
    Effect.Effect.Error<ReturnType<PriceOracle["of"]>>
  >

  let gasPrices!: RunPromiseExitResult<{
    source: O.Option<BaseGasPrice>
    destination: O.Option<BaseGasPrice>
  }, GasPriceError>

  /**
   * Ratio of source / destination
   */
  let ratio!: RunPromiseExitResult<
    O.Option<{
      ratio: BigDecimal.BigDecimal
      source: PriceSource
      destination: PriceSource
    }>,
    PriceError
  >

  $effect.root(() => {
    ratio = AppRuntime.runPromiseExit$(() =>
      pipe(
        O.all({
          source: TransferData.sourceChain,
          destination: TransferData.destinationChain,
        }),
        O.map(R.map(x => x.universal_chain_id)),
        O.map(({ source, destination }) =>
          Effect.andThen(PriceOracle, ({ ratio }) => ratio(source, destination))
        ),
        Effect.transposeOption,
        Effect.withExecutionPlan(PriceOraclePlan),
      ), {
      onInterrupt: "none",
    })

    usdPrices = AppRuntime.runPromiseExit$(() =>
      pipe(
        {
          source: TransferData.sourceChain,
          destination: TransferData.destinationChain,
        },
        R.map(Effect.transposeMapOption(usdOfChainGas)),
        Effect.allWith({ concurrency: 2 }),
        Effect.withExecutionPlan(PriceOraclePlan),
      ), {
      onInterrupt: "none",
    })

    gasPrices = AppRuntime.runPromiseExit$(() =>
      pipe(
        {
          source: TransferData.sourceChain,
          destination: TransferData.destinationChain,
        },
        R.map(Effect.transposeMapOption(gasForChain)),
        Effect.allWith({ concurrency: 2 }),
      ), {
      onInterrupt: "none",
    })
  })

  const maybeRatio = $derived(pipe(
    ratio.current,
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
    O.flatMap(O.map(x => x.ratio)),
  ))

  const decorate = (
    self: { fees: BaseFees; gasPrice: BaseGasPrice; ratio: BigDecimal.BigDecimal } & typeof config,
  ) => {
    const gasDecimals = self.gasPrice.scale

    const asBaseUnitK = (a: AtomicGasPrice): Writer.Writer<readonly string[], BaseGasPrice> => {
      const result = pipe(
        BigDecimal.multiply(a, BigDecimal.make(1n, gasDecimals)),
        BaseGasPrice,
      )

      return [
        result,
        [
          `${BigDecimal.format(a)} atomic gas units &times; 10<sup>-${gasDecimals}</sup>`,
        ],
      ]
    }

    const asAtomicUnitK = (a: BaseGasPrice): Writer.Writer<readonly string[], AtomicGasPrice> => {
      const result = pipe(
        BigDecimal.multiply(a, BigDecimal.make(1n, -gasDecimals)),
        AtomicGasPrice,
      )

      return [
        result,
        [
          `${a} base gas units &times; 10<sup>${-gasDecimals}</sup>`,
        ],
      ]
    }

    const applyGasPriceK = (
      gasUnits: GasFee,
    ): Writer.Writer<readonly string[], AtomicGasPrice> => {
      const atomicGasPrice = asAtomicUnitK(self.gasPrice)
      const result = AtomicGasPrice(BigDecimal.multiply(gasUnits, atomicGasPrice[0]))
      return [
        result,
        [
          `${BigDecimal.format(gasUnits)} gas units &times; ${
            BigDecimal.format(atomicGasPrice[0])
          } atomic gas unit/gas unit`,
        ],
      ]
    }

    const applyRatioK = (a: BaseGasPrice): Writer.Writer<readonly string[], BaseGasPrice> => {
      const result = pipe(
        BigDecimal.unsafeDivide(a, self.ratio),
        BaseGasPrice,
      )

      return [
        result,
        [
          `${BigDecimal.format(a)} &times; ${BigDecimal.format(self.ratio)}`,
        ],
      ]
    }

    const applyFeeMultiplierK = (
      a: BaseGasPrice,
    ): Writer.Writer<readonly string[], BaseGasPrice> => {
      const result = pipe(
        BigDecimal.multiply(a, self.feeMultiplier),
        BaseGasPrice,
      )
      return [
        result,
        [
          `${BigDecimal.format(a)} atomic gas units × ${BigDecimal.format(self.feeMultiplier)}`,
        ],
      ]
    }

    const applyBatchDivisionK = (
      a: BaseGasPrice,
    ): [BaseGasPrice, readonly string[]] => {
      const result = pipe(
        BigDecimal.unsafeDivide(a, self.batchDivideNumber),
        BaseGasPrice,
      )
      return [
        result,
        [
          `${BigDecimal.format(a)} atomic gas units &divide; ${
            BigDecimal.format(self.batchDivideNumber)
          }`,
        ],
      ]
    }

    const formatToDisplayK = (a: BaseGasPrice): Writer.Writer<readonly string[], string> => {
      const round = (x: BaseGasPrice): [BaseGasPrice, readonly string[]] => {
        const scale = 10
        const mode = "from-zero"
        const result = pipe(
          BigDecimal.round(x, { scale, mode }),
          BaseGasPrice,
        )
        return [
          result,
          [
            `<code>round</code><sub>${scale}</sub>(${
              BigDecimal.format(a)
            }) with mode <i>${mode}</i>`,
          ],
        ]
      }

      const format = (x: BaseGasPrice): Writer.Writer<readonly string[], string> => {
        const result = BigDecimal.unsafeToNumber(x).toFixed(10)
        return [
          result,
          [`<code>format</code>(${x})`],
        ]
      }

      return pipe(
        a,
        pipe(
          round,
          composeK(format),
        ),
      )
    }

    return Object.assign(self, {
      gasDecimals,
      applyGasPriceK,
      asBaseUnitK,
      asAtomicUnitK,
      applyFeeMultiplierK,
      applyRatioK,
      applyBatchDivisionK,
      formatToDisplayK,
    })
  }

  const baseFees: O.Option<BaseFees> = $derived(pipe(
    TransferData.channel,
    O.map(Struct.get("fees")),
    O.map(Struct.omit("PACKET_SEND_LC_UPDATE_L2")),
    O.map((fees) => {
      const withDefault = O.getOrElse<GasFee>(() => GasFee.make(BigDecimal.make(0n, 0)))<GasFee>
      return Struct.evolve(fees, {
        PACKET_RECV: withDefault,
        PACKET_SEND_LC_UPDATE_L0: withDefault,
        PACKET_SEND_LC_UPDATE_L1: withDefault,
      })
    }),
  ))

  /**
   * The denom symbol for the source transfer chain.
   * @example
   * ```md
   * Ethereum chain => "ETH"
   * ```
   */
  const sourceSymbol = $derived(O.gen(function*() {
    const chain = yield* TransferData.sourceChain
    const symbol = yield* Match.value(chain).pipe(
      Match.when({ rpc_type: "cosmos" }, (x) =>
        pipe(
          R.get(chainInfoMap, x.chain_id),
          O.map(x => x.feeCurrencies),
          O.flatMap(A.head),
          O.map(x => x.coinDenom),
        )),
      Match.when({ rpc_type: "evm" }, (x) =>
        pipe(
          A.findFirst(VIEM_CHAINS, y => String(y.id) === x.chain_id),
          O.map(x => x.nativeCurrency.symbol),
        )),
      Match.orElseAbsurd,
    )

    return symbol
  }))

  const sourceGasUnitPrice = $derived(pipe(
    gasPrices.current,
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
    O.flatMap(x => x.source),
  ))

  const destGasUnitPrice = $derived(pipe(
    gasPrices.current,
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
    O.flatMap(x => x.destination),
  ))

  const decoratedConfig = $derived(pipe(
    O.all({
      baseFees,
      // XXX: source / dest here should be determined on per-transaction basis
      gasPrice: destGasUnitPrice,
      ratio: maybeRatio,
    }),
    O.map(({ baseFees: fees, gasPrice, ratio }) => ({ fees, gasPrice, ratio, ...config })),
    O.map(decorate),
  ))

  // TODO: tuple-ify outputs; concatenate to show record of calculations performed
  const calculatedFees = $derived(pipe(
    O.map(decoratedConfig, (config) =>
      Struct.evolve(config.fees, {
        PACKET_SEND_LC_UPDATE_L1: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
            composeK(config.applyBatchDivisionK),
          ),
        ),
        PACKET_SEND_LC_UPDATE_L0: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
            composeK(config.applyBatchDivisionK),
          ),
        ),
        PACKET_RECV: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
          ),
        ),
      })),
  ))

  const displayFees = $derived(pipe(
    O.all({ calculatedFees, decoratedConfig }),
    O.map(({ calculatedFees, decoratedConfig: { formatToDisplayK, applyRatioK } }) =>
      Struct.evolve(calculatedFees, {
        PACKET_SEND_LC_UPDATE_L0: (x) => {
          const f = pipe(
            applyRatioK,
            composeK(formatToDisplayK),
          )

          const g = composeK(identity<typeof x>, f)
          const h = g(x)
          return h
        },
        PACKET_SEND_LC_UPDATE_L1: (x) =>
          composeK(
            () => x,
            pipe(
              applyRatioK,
              composeK(formatToDisplayK),
            ),
          )(x),
        PACKET_RECV: (x) =>
          composeK(
            () => x,
            pipe(
              applyRatioK,
              composeK(formatToDisplayK),
            ),
          )(x),
      })
    ),
  ))

  type BaseFeeInfo = {
    label: string
    key: keyof BaseFees
    isBatched: boolean
    description: string
  }

  const feeItems: BaseFeeInfo[] = [
    {
      label: "Light Client (L1)",
      key: "PACKET_SEND_LC_UPDATE_L1",
      isBatched: true,
      description: "L1 light client update fee (shared across batch).",
    },
    {
      label: "Light Client (L0)",
      key: "PACKET_SEND_LC_UPDATE_L0",
      isBatched: true,
      description: "L0 light client update fee (shared across batch).",
    },
    {
      label: "Packet Receive",
      key: "PACKET_RECV",
      isBatched: false,
      description: "Fee for receiving the packet on the destination chain.",
    },
  ]

  const newFeeBreakdown = $derived.by(() => {
    const enrich = (x: BaseFeeInfo) =>
      O.gen(function*() {
        const { formatToDisplayK: format, feeMultiplier } = yield* decoratedConfig
        const amount = yield* O.map(displayFees, flow(Struct.get(x.key), Tuple.getFirst))
        const baseFee = yield* O.map(calculatedFees, flow(Struct.get(x.key), Tuple.getFirst))
        const calc = A.join(
          yield* O.map(displayFees, flow(Struct.get(x.key), Tuple.getSecond)),
          "<br/>&rarr; ",
        )
        const symbol = yield* sourceSymbol

        // const baseFeeStep = O.gen(function*() {
        //   const gasUnit = yield* sourceGasUnitPrice
        //   const operation = `${format(baseFee)} × ${format(gasUnit)} ${symbol}`
        //   const result = BigDecimal.format(BigDecimal.multiply(baseFee, gasUnit))
        //   return {
        //     operation,
        //     result: `${result} ${symbol}`,
        //   }
        // })

        // const protocolFeeStep = O.gen(function*() {
        //   const gasUnit = yield* sourceGasUnitPrice
        //   const mult = BigDecimal.scale(feeMultiplier, 100)
        //   const operation = `+ ${BigDecimal.format(mult)}%`
        //   const result = pipe(
        //     baseFee,
        //     BigDecimal.multiply(gasUnit),
        //     BigDecimal.multiply(mult),
        //     format,
        //   )
        //   return {
        //     operation,
        //     result: `${result} ${symbol}`,
        //   }
        // })

        return {
          ...x,
          baseFee,
          steps: {
            calc,
            baseFee: O.none(),
            protocolFee: O.none(),
          },
          amount,
        }
      })

    return pipe(
      feeItems,
      A.map(enrich),
    )
  })

  const errors = $derived.by(() => {
    // TODO: extract to helper
    const extractError = <E>(x: O.Option<Exit.Exit<any, E>>) =>
      pipe(
        x,
        O.flatMap(Exit.causeOption),
      )
    return pipe(
      [
        extractError(gasPrices.current),
        extractError(usdPrices.current),
        extractError(ratio.current),
      ] as const,
      A.getSomes,
      Unify.unify,
      A.map(Cause.squash),
      A.map(x => (x as any)?.message),
      A.filter(Predicate.isNotUndefined),
    )
  })

  const usdSources = $derived(pipe(
    usdPrices.current,
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
    O.map(R.getSomes),
    O.map(R.map(x => x.source)),
  ))

  const gasDisplay = $derived(pipe(
    gasPrices.current,
    // TODO: extract to helper
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
    O.getOrNull,
  ))

  const totalFee = $derived(pipe(
    calculatedFees,
    O.map(R.values),
    O.map(A.map(Tuple.getFirst)),
    O.map(BigDecimal.sumAll),
    O.map(BaseGasPrice),
  ))

  // XXX: this is wrong; need to get usd price of source symbol instead of ratio
  const usdDisplay = $derived(pipe(
    O.all({
      decoratedConfig,
      perUsd: O.flatMap(
        usdPrices.current,
        Exit.match({
          onSuccess: O.some,
          onFailure: O.none,
        }),
      ).pipe(
        O.map(x => x.source),
        O.map(O.map(x => x.price)),
        O.flatten,
      ),
      totalFee,
    }),
    O.map(({ decoratedConfig, perUsd, totalFee }) => BigDecimal.multiply(perUsd, totalFee)),
    O.map(BigDecimal.round({
      scale: 4,
      mode: "from-zero",
    })),
    O.map(BigDecimal.format),
  ))

  // TODO
  // export type Intent = {
  //   sender: AddressCanonicalBytes
  //   receiver: AddressCanonicalBytes
  //   baseToken: string
  //   baseAmount: TokenRawAmount
  //   quoteAmount: TokenRawAmount
  //   decimals: number
  //   sourceChain: Chain
  //   sourceChainId: UniversalChainId
  //   sourceChannelId: ChannelId
  //   destinationChain: Chain
  //   channel: Channel
  //   ucs03address: string
  // }

  const feeIntent: E.Either<FeeIntent, string> = $derived(E.gen(function*() {
    const _totalFee = yield* pipe(
      totalFee,
      E.fromOption(() => "No total fee"),
    )
    const sourceChain = yield* pipe(
      TransferData.sourceChain,
      E.fromOption(() => "No source chain"),
    )
    const gasDenom = yield* pipe(
      R.get(GAS_DENOMS, sourceChain.universal_chain_id),
      E.fromOption(() => `No gas denom for ${sourceChain.universal_chain_id}`),
    )

    const baseToken = gasDenom.address
    // TODO: get from more reliable source
    const decimals = gasDenom.decimals
    const amount = BigDecimal.multiply(
      BigDecimal.make(_totalFee.value, 0),
      BigDecimal.make(1n, -decimals),
    )
    const BIamount = BI.multiply(
      amount.value,
      10n ** (BigInt(amount.scale * -1) - 1n),
    )

    return {
      decimals,
      baseToken,
      quoteAmount: TokenRawAmount.make(0n),
      baseAmount: TokenRawAmount.make(BIamount),
    } as const
  }))

  return {
    get feeIntent() {
      return feeIntent
    },
    get baseFees() {
      return baseFees
    },
    get gasPrices() {
      return gasPrices
    },
    get sourceGasUnitPrice() {
      return sourceGasUnitPrice
    },
    get feeBreakdown() {
      return newFeeBreakdown
    },
    get usdPrices() {
      return usdPrices
    },
    get gasDisplay() {
      return gasDisplay
    },
    get usdDisplay() {
      return usdDisplay
    },
    get feeMultiplier() {
      return config.feeMultiplier
    },
    get batchDivideNumber() {
      return config.batchDivideNumber
    },
    get totalFee() {
      console.log("Total fee (atomic):", totalFee.toString())
      return totalFee
    },
    get usdSources() {
      return usdSources
    },
    /**
     * Total cost in gas token symbol.
     */
    get feeDisplay() {
      return pipe(
        O.all({ totalFee, decoratedConfig }),
        O.map(({ totalFee, decoratedConfig: { formatToDisplayK, applyRatioK } }) =>
          pipe(
            // TODO: normalize to base gas price due to USD conversion being in base
            totalFee,
            (x) => formatToDisplayK(x)[0],
          )
        ),
      )
    },
    /**
     * Symbol for fee currency.
     */
    get symbol(): O.Option<string> {
      return sourceSymbol
    },
    get ratio() {
      return pipe(
        ratio?.current,
        O.flatMap(Exit.match({
          onSuccess: O.some,
          onFailure: O.none,
        })),
        O.flatten,
        O.map(x => x.ratio),
      )
    },
    get errors() {
      return errors
    },
  } as const
}

export const FeeStore = createFeeStore()
