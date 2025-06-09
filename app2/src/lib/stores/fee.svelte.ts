import { GasPriceMap } from "$lib/gasprice"
import type { GasPriceError } from "$lib/gasprice/error"
import { GasPrice } from "$lib/gasprice/service"
import * as AppRuntime from "$lib/runtime"
import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { transferData as TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import type { RunPromiseExitResult } from "$lib/utils/effect.svelte"
import { VIEM_CHAINS } from "@unionlabs/sdk/constants/viem-chains"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import { Chain } from "@unionlabs/sdk/schema"
import type { Fees } from "@unionlabs/sdk/schema/fee"
import {
  Array as A,
  BigDecimal,
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
  Unify,
} from "effect"

import { constant, flow } from "effect/Function"

type BaseFees = { [K in keyof Fees]: Fees[K] extends O.Option<infer T> ? T : never }

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
 * - Conversion rate is detremined optionally if USD is avaliable for source and destination.
 *
 * TOOD:
 * - How to represent loading? (only show loading for side-effecting)
 */
const createFeeStore = () => {
  const config = {
    gasPrice: BigDecimal.make(10n, 0), // gasPrice from chain
    decimals: 6, // BABY token decimals (in rep)
    feeMultiplier: BigDecimal.make(12n, 1), // Union hardcoded fee
    batchDivideNumber: BigDecimal.unsafeFromNumber(2), // Api?
    gasTokenDecimals: 6, // Token data
    gasTokenSymbol: "BABY", // Token data
  } as const

  let usdPrices!: RunPromiseExitResult<
    R.ReadonlyRecord<
      "source" | "destination",
      O.Option<Effect.Effect.Success<ReturnType<PriceOracle["of"]>>>
    >,
    Effect.Effect.Error<ReturnType<PriceOracle["of"]>>
  >

  let gasPrices!: RunPromiseExitResult<{
    source: O.Option<bigint>
    destination: O.Option<bigint>
  }, GasPriceError>

  $effect.root(() => {
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

  const decorate = (self: { fees: BaseFees } & typeof config) => {
    const applyGasPrice = (gasUnits: BigDecimal.BigDecimal) =>
      BigDecimal.multiply(gasUnits, self.gasPrice)

    const applyFeeMultiplier = (ubbnAmount: BigDecimal.BigDecimal) =>
      BigDecimal.multiply(ubbnAmount, self.feeMultiplier)

    const applyBatchDivision = (ubbnAmount: BigDecimal.BigDecimal) =>
      BigDecimal.divide(ubbnAmount, self.batchDivideNumber)

    const formatToDisplay = (ubbnAmount: BigDecimal.BigDecimal): string => {
      const exp = BigDecimal.make(10n, -6)
      const babyAmount = E.fromOption(
        BigDecimal.divide(ubbnAmount, exp),
        () => `could not divide by ${exp}`,
      )
      if (E.isLeft(babyAmount)) {
        return babyAmount.left
      }
      return Match.value(babyAmount.right).pipe(
        Match.when(BigDecimal.lessThan(BigDecimal.make(1n, 3)), BigDecimal.format),
        Match.when(BigDecimal.lessThan(BigDecimal.make(1n, 0)), BigDecimal.format),
        Match.when(BigDecimal.lessThan(BigDecimal.make(100n, 0)), BigDecimal.format),
        Match.orElse(BigDecimal.format),
      )
    }

    return Object.assign(self, {
      applyGasPrice,
      applyFeeMultiplier,
      applyBatchDivision,
      formatToDisplay,
    })
  }

  const baseFees: O.Option<BaseFees> = $derived(pipe(
    TransferData.channel,
    O.map(Struct.get("fees")),
    O.map(Struct.evolve({
      PACKET_SEND: O.getOrElse(constant(0n))<bigint>,
      PACKET_RECV: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L0: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L1: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L2: O.getOrElse(constant(0n))<bigint>,
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

  const decoratedConfig = $derived(pipe(
    baseFees,
    O.map((fees) => ({ fees, ...config })),
    O.map(decorate),
  ))

  const calculatedFees = $derived(pipe(
    O.map(decoratedConfig, (config) =>
      Struct.evolve(config.fees, {
        PACKET_SEND: flow(
          BigDecimal.fromBigInt,
          config.applyGasPrice,
          config.applyFeeMultiplier,
          O.some,
        ),
        PACKET_SEND_LC_UPDATE_L1: flow(
          BigDecimal.fromBigInt,
          config.applyGasPrice,
          config.applyFeeMultiplier,
          config.applyBatchDivision,
        ),
        PACKET_SEND_LC_UPDATE_L0: flow(
          BigDecimal.fromBigInt,
          config.applyGasPrice,
          config.applyFeeMultiplier,
          config.applyBatchDivision,
        ),
        PACKET_SEND_LC_UPDATE_L2: flow(
          BigDecimal.fromBigInt,
          config.applyGasPrice,
          config.applyFeeMultiplier,
          config.applyBatchDivision,
        ),
        PACKET_RECV: flow(
          BigDecimal.fromBigInt,
          config.applyGasPrice,
          config.applyFeeMultiplier,
          O.some,
        ),
      })),
  ))

  const displayFees = $derived(pipe(
    O.all({ calculatedFees, decoratedConfig }),
    O.map(({ calculatedFees, decoratedConfig: { formatToDisplay } }) =>
      Struct.evolve(calculatedFees, {
        PACKET_SEND: O.map(formatToDisplay),
        PACKET_SEND_LC_UPDATE_L0: O.map(formatToDisplay),
        PACKET_SEND_LC_UPDATE_L1: O.map(formatToDisplay),
        PACKET_SEND_LC_UPDATE_L2: O.map(formatToDisplay),
        PACKET_RECV: O.map(formatToDisplay),
      })
    ),
  ))

  type BaseFeeInfo = {
    label: string
    key: keyof Fees
    isBatched: boolean
    description: string
  }

  const feeItems: BaseFeeInfo[] = [
    {
      label: "Packet Send",
      key: "PACKET_SEND",
      isBatched: false,
      description: "Fee for sending the packet to the destination chain",
    },
    {
      label: "Light Client (L2)",
      key: "PACKET_SEND_LC_UPDATE_L2",
      isBatched: true,
      description: "L2 light client update fee (shared across batch)",
    },
    {
      label: "Light Client (L1)",
      key: "PACKET_SEND_LC_UPDATE_L1",
      isBatched: true,
      description: "L1 light client update fee (shared across batch)",
    },
    {
      label: "Light Client (L0)",
      key: "PACKET_SEND_LC_UPDATE_L0",
      isBatched: true,
      description: "L0 light client update fee (shared across batch)",
    },
    {
      label: "Packet Receive",
      key: "PACKET_RECV",
      isBatched: false,
      description: "Fee for receiving the packet on the destination chain",
    },
  ]

  /**
   * Fee breakdown items for iteration
   */
  const feeBreakdown = $derived([
    pipe(
      O.all({
        amount: O.flatMap(displayFees, Struct.get("PACKET_SEND")),
        baseFee: O.flatMap(calculatedFees, Struct.get("PACKET_SEND")),
      }),
      O.map(({ amount, baseFee }) => ({
        label: "Packet Send",
        amount,
        baseFee,
        isBatched: false,
        description: "Fee for sending the packet to the destination chain.",
      })),
    ),
    pipe(
      O.all({
        amount: O.flatMap(displayFees, Struct.get("PACKET_SEND_LC_UPDATE_L2")),
        baseFee: O.flatMap(calculatedFees, Struct.get("PACKET_SEND_LC_UPDATE_L2")),
      }),
      O.map(({ amount, baseFee }) => ({
        label: "Light Client (L2)",
        amount,
        baseFee,
        isBatched: true,
        description: "L2 light client update fee (shared across batch).",
      })),
    ),
    pipe(
      O.all({
        amount: O.flatMap(displayFees, Struct.get("PACKET_SEND_LC_UPDATE_L1")),
        baseFee: O.flatMap(calculatedFees, Struct.get("PACKET_SEND_LC_UPDATE_L1")),
      }),
      O.map(({ amount, baseFee }) => ({
        label: "Light Client (L1)",
        amount,
        baseFee,
        isBatched: true,
        description: "L1 light client update fee (shared across batch).",
      })),
    ),
    pipe(
      O.all({
        amount: O.flatMap(displayFees, Struct.get("PACKET_SEND_LC_UPDATE_L0")),
        baseFee: O.flatMap(calculatedFees, Struct.get("PACKET_SEND_LC_UPDATE_L0")),
      }),
      O.map(({ amount, baseFee }) => ({
        label: "Light Client (L0)",
        amount,
        baseFee,
        isBatched: true,
        description: "L0 light client update fee (shared across batch).",
      })),
    ),
    pipe(
      O.all({
        amount: O.flatMap(displayFees, Struct.get("PACKET_RECV")),
        baseFee: O.flatMap(calculatedFees, Struct.get("PACKET_RECV")),
      }),
      O.map(({ amount, baseFee }) => ({
        label: "Packet Receive",
        amount,
        baseFee,
        isBatched: false,
        description: "Fee for receiving the packet on the destination chain.",
      })),
    ),
  ])

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

  const usdDisplay = $derived(pipe(
    usdPrices.current,
    // TODO: extract to helper
    O.flatMap(Exit.match({
      onSuccess: O.some,
      onFailure: O.none,
    })),
  ))

  const totalFee = $derived(pipe(
    calculatedFees,
    O.map(R.values),
    O.map(O.all),
    O.map(O.map(BigDecimal.sumAll)),
    O.flatten,
  ))

  return {
    get baseFees() {
      return baseFees
    },
    get gasPrices() {
      return gasPrices
    },
    get sourceGasUnitPrice() {
      return pipe(
        gasPrices.current,
        O.flatMap(Exit.match({
          onSuccess: O.some,
          onFailure: O.none,
        })),
        O.flatMap(x => x.source),
        O.map(x => BigDecimal.make(x, 0)),
      )
    },
    get feeBreakdown() {
      return feeBreakdown
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
    get errors() {
      return errors
    },
  } as const
}

export const FeeStore = createFeeStore()
