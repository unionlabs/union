import { GasPriceMap } from "$lib/gasprice"
import type { GasPriceError } from "$lib/gasprice/error"
import { AtomicGasPrice, BaseGasPrice, GasPrice } from "$lib/gasprice/service"
import * as AppRuntime from "$lib/runtime"
import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { transferData as TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import * as Writer from "$lib/typeclass/Writer.js"
import type { RunPromiseExitResult } from "$lib/utils/effect.svelte"
import * as StringInstances from "@effect/typeclass/data/String"
import * as FlatMap from "@effect/typeclass/FlatMap"
import { VIEM_CHAINS } from "@unionlabs/sdk/constants/viem-chains"
import { PriceError, PriceOracle, PriceSource } from "@unionlabs/sdk/PriceOracle"
import { Chain } from "@unionlabs/sdk/schema"
import type { Fees } from "@unionlabs/sdk/schema/fee"
import {
  Array as A,
  BigDecimal,
  Cause,
  Effect,
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
import { constant, flow } from "effect/Function"

const composeK = pipe(
  StringInstances.Semigroup,
  Writer.fromSemigroup,
  FlatMap.composeK,
)

type BaseFees = Omit<
  { [K in keyof Fees]: Fees[K] extends O.Option<infer T> ? T : never },
  "PACKET_SEND_LC_UPDATE_L2"
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
      )
    )

    usdPrices = AppRuntime.runPromiseExit$(() =>
      pipe(
        {
          source: TransferData.sourceChain,
          destination: TransferData.destinationChain,
        },
        R.map(Effect.transposeMapOption(usdOfChainGas)),
        Effect.allWith({ concurrency: 2 }),
      )
    )

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
    console.log({ symbol: gasDecimals })

    const asBaseUnit = (a: AtomicGasPrice): [BaseGasPrice, string] => {
      const result = pipe(
        BigDecimal.multiply(a, BigDecimal.make(1n, gasDecimals)),
        BaseGasPrice,
      )

      return [
        result,
        `${a} atomic unit * 10e-${gasDecimals} = ${result} base unit`,
      ]
    }

    const asAtomicUnit = (a: BaseGasPrice): [AtomicGasPrice, string] => {
      const result = pipe(
        BigDecimal.multiply(a, BigDecimal.make(1n, -gasDecimals)),
        AtomicGasPrice,
      )

      return [
        result,
        `${a} base unit * 10e${-gasDecimals} = ${result} atomic unit`,
      ]
    }

    const applyGasPrice = (gasUnits: BigDecimal.BigDecimal): [AtomicGasPrice, string] => {
      const atomicGasPrice = asAtomicUnit(self.gasPrice)
      // TODO: composeK
      const result = AtomicGasPrice(BigDecimal.multiply(gasUnits, atomicGasPrice[0]))
      return [
        result,
        `${gasUnits} gas units × ${atomicGasPrice[0]} wei/gas unit = ${result} wei`,
      ]
    }

    const applyRatio = (a: BaseGasPrice): [BaseGasPrice, string] => {
      const result = pipe(
        BigDecimal.multiply(a, self.ratio),
        BaseGasPrice,
      )

      return [
        result,
        `${a} base unit * ${self.ratio} = ${result} base unit`,
      ]
    }

    const applyFeeMultiplier = (amount: AtomicGasPrice): [AtomicGasPrice, string] => {
      const result = pipe(
        BigDecimal.multiply(amount, self.feeMultiplier),
        AtomicGasPrice,
      )
      return [
        result,
        `${amount} × ${self.feeMultiplier} = ${result} atomic units`,
      ]
    }

    const applyBatchDivision = (
      a: AtomicGasPrice,
    ): [AtomicGasPrice, string] => {
      const result = pipe(
        BigDecimal.unsafeDivide(a, self.batchDivideNumber),
        AtomicGasPrice,
      )
      return [
        result,
        `${a} atomic units ÷ ${self.batchDivideNumber} = ${result} atomic units`,
      ]
    }

    const formatToDisplay = (a: AtomicGasPrice): string => {
      // TODO: add composeK
      return pipe(
        asBaseUnit(a)[0],
        BigDecimal.format,
      )
    }

    return Object.assign(self, {
      gasDecimals,
      applyGasPrice,
      asBaseUnit,
      asAtomicUnit,
      applyFeeMultiplier,
      applyRatio,
      applyBatchDivision,
      formatToDisplay,
    })
  }

  const baseFees: O.Option<BaseFees> = $derived(pipe(
    TransferData.channel,
    O.map(Struct.get("fees")),
    O.map(Struct.omit("PACKET_SEND_LC_UPDATE_L2")),
    O.map(Struct.evolve({
      PACKET_RECV: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L0: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L1: O.getOrElse(constant(0n))<bigint>,
    })),
  ))

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
          BigDecimal.fromBigInt,
          pipe( // TODO: extract
            config.applyGasPrice,
            composeK(config.applyFeeMultiplier),
            composeK(config.applyBatchDivision),
          ),
        ),
        PACKET_SEND_LC_UPDATE_L0: flow(
          BigDecimal.fromBigInt,
          pipe( // TODO: extract
            config.applyGasPrice,
            composeK(config.applyFeeMultiplier),
            composeK(config.applyBatchDivision),
          ),
        ),
        PACKET_RECV: flow(
          BigDecimal.fromBigInt,
          pipe( // TODO: extract
            config.applyGasPrice,
            composeK(config.applyFeeMultiplier),
          ),
        ),
      })),
  ))

  const displayFees = $derived(pipe(
    O.all({ calculatedFees, decoratedConfig }),
    O.map(({ calculatedFees, decoratedConfig: { formatToDisplay } }) =>
      Struct.evolve(calculatedFees, {
        PACKET_SEND_LC_UPDATE_L0: Tuple.mapFirst(formatToDisplay),
        PACKET_SEND_LC_UPDATE_L1: Tuple.mapFirst(formatToDisplay),
        PACKET_RECV: Tuple.mapFirst(formatToDisplay),
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
        const { formatToDisplay: format, feeMultiplier } = yield* decoratedConfig
        const amount = yield* O.map(displayFees, flow(Struct.get(x.key), Tuple.getFirst))
        const baseFee = yield* O.map(calculatedFees, flow(Struct.get(x.key), Tuple.getFirst))
        const calc = yield* O.map(calculatedFees, flow(Struct.get(x.key), Tuple.getSecond))
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
    x => {
      console.log(JSON.stringify(calculatedFees, null, 2))
      return x
    },
    O.map(R.values),
    O.map(A.map(Tuple.getFirst)),
    O.map(BigDecimal.sumAll),
    O.map(AtomicGasPrice),
    O.tap(x => {
      console.log("totalFee", { scale: x.scale })
      return O.some(x)
    }),
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
    x => {
      console.log({ totalFee: x })
      return x
    },
    O.map(({ decoratedConfig, perUsd, totalFee }) =>
      BigDecimal.multiply(perUsd, decoratedConfig.asBaseUnit(totalFee)[0])
    ),
    O.map(BigDecimal.round({
      scale: 4,
      mode: "from-zero",
    })),
    // O.map(BigDecimal.truncate(2)),
    O.map(BigDecimal.format),
  ))

  return {
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
      console.log({ totalFee })
      return totalFee
    },
    /**
     * Total cost in gas token symbol.
     */
    get feeDisplay() {
      return pipe(
        O.all({ totalFee, decoratedConfig }),
        O.map(({ totalFee, decoratedConfig: { formatToDisplay } }) => formatToDisplay(totalFee)),
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
