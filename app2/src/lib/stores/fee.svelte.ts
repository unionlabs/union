import { GasPriceMap } from "$lib/gasprice"
import { GasPriceError } from "$lib/gasprice/error"
import { AtomicGasPrice, BaseGasPrice, GasPrice } from "$lib/gasprice/service"
import { chainInfoMap } from "$lib/services/cosmos/chain-info/config"
import { transferData as TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import type { Intent } from "$lib/transfer/shared/services/filling/create-context.ts"
import * as Writer from "$lib/typeclass/Writer.js"
import * as ArrayInstances from "@effect/typeclass/data/Array"
import * as FlatMap from "@effect/typeclass/FlatMap"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms"
import { VIEM_CHAINS } from "@unionlabs/sdk/constants/viem-chains"
import { PriceError, PriceOracle, PriceResult } from "@unionlabs/sdk/PriceOracle"
import { Chain, TokenRawAmount } from "@unionlabs/sdk/schema"
import { type Fees, GasFee } from "@unionlabs/sdk/schema/fee"
import {
  Array as A,
  BigDecimal,
  Cause,
  Effect,
  Either as E,
  Match,
  Option as O,
  pipe,
  Predicate,
  Record as R,
  Schedule,
  Stream,
  Struct,
  Tuple,
  Unify,
} from "effect"
import { flow, identity } from "effect/Function"

const LogWriter = Writer.getMonad(ArrayInstances.getMonoid<string[]>())
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
 */
const createFeeStore = () => {
  const config = {
    feeMultiplier: BigDecimal.unsafeFromString("1.2"), // Union hardcoded fee
    batchDivideNumber: BigDecimal.unsafeFromString("3"), // TODO: source from API?
  } as const

  type Happy = O.Option<
    E.Either<
      {
        source: {
          gas: AtomicGasPrice
          gasDecimals: number
          gasMinimalDenom: string
          gasDenom: string
          additiveFee: O.Option<AtomicGasPrice>
          usd: PriceResult
        }
        destination: {
          gas: AtomicGasPrice
          gasDecimals: number
          gasMinimalDenom: string
          gasDenom: string
          additiveFee: O.Option<AtomicGasPrice>
          usd: PriceResult
        }
      },
      GasPriceError | PriceError
    >
  >

  let data = $state<Happy>(O.none())

  const fetchFee = $derived.by(() => {
    const _source = TransferData.sourceChain
    const _destination = TransferData.destinationChain
    const _channel = TransferData.channel

    const schedule = pipe(
      Schedule.once,
      Schedule.andThen(Schedule.spaced("2 minutes")),
    )
    return pipe(
      Stream.fromSchedule(schedule),
      Stream.mapEffect(() =>
        Effect.sync(() => {
          data = O.none()
        })
      ),
      Stream.mapEffect(() =>
        Effect.gen(function*() {
          const { source, destination, channel } = yield* pipe(
            O.all({
              source: _source,
              destination: _destination,
              channel: _channel,
            }),
          )

          const fetchAllFor = Effect.fn(function*(chain: Chain) {
            const {
              gas,
              usd,
            } = yield* Effect.all({
              gas: gasForChain(chain),
              usd: usdOfChainGas(chain),
            }, { concurrency: 2 })

            return {
              gas: gas.value,
              gasMinimalDenom: gas.minimalDenom,
              gasDenom: gas.denom,
              gasDecimals: gas.decimals,
              additiveFee: gas.additiveFee,
              usd,
            }
          })

          const c = yield* pipe(
            Effect.all({
              source: Effect.either(fetchAllFor(source)),
              destination: Effect.either(fetchAllFor(destination)),
            }, { concurrency: 2 }),
            Effect.map(E.all),
          )

          yield* Effect.sync(() => {
            data = O.some(c)
          })
        }).pipe(
          Effect.catchTag("NoSuchElementException", () =>
            Effect.sync(() => {
              data = O.none()
            })),
        )
      ),
      Stream.runDrain,
    )
  })

  const maybeRatio = $derived(pipe(
    data,
    O.flatMap(O.getRight),
    O.map((data) => ({
      ratio: BigDecimal.unsafeDivide(data.source.usd.price, data.destination.usd.price),
      source: data.source.usd.source,
      destination: data.destination.usd.source,
    })),
    O.map(x => x.ratio),
  ))

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

  const decorate = (
    self: {
      // TODO: consolidate type
      gasPrice: {
        value: AtomicGasPrice
        additiveFee: O.Option<AtomicGasPrice>
        decimals: number
        denom: string
        minimalDenom: string
      }
      ratio: BigDecimal.BigDecimal
    } & typeof config,
  ) => {
    const gasDecimals = self.gasPrice.decimals
    const _sourceSymbol = O.getOrUndefined(sourceSymbol)
    const _destSymbol = O.getOrUndefined(destSymbol)

    const asBaseUnitK = (a: AtomicGasPrice): Writer.Writer<readonly string[][], BaseGasPrice> => {
      const div = BigDecimal.make(1n, -gasDecimals)
      const result = pipe(
        BigDecimal.unsafeDivide(a, div),
        BaseGasPrice,
      )

      return [
        result,
        [
          [
            `${self.gasPrice.minimalDenom} &rarr; ${self.gasPrice.denom}`,
            "&divide;",
            `${BigDecimal.format(div)}`,
            // `${
            //   JSON.stringify(a.toJSON())
            // } atomic gas units &times; 10<sup>${gasDecimals}</sup> = ${result} base gas units`,
          ],
        ],
      ]
    }

    const applyGasPriceK = (
      gasUnits: GasFee,
    ): Writer.Writer<readonly string[][], AtomicGasPrice> => {
      const multiply = (a: AtomicGasPrice): Writer.Writer<readonly string[][], AtomicGasPrice> => {
        const result = pipe(
          AtomicGasPrice(BigDecimal.multiply(gasUnits, a)),
        )
        return [
          result,
          [
            [
              "Gas Fee",
              "=",
              `${BigDecimal.format(gasUnits)}`,
              //     `${JSON.stringify(gasUnits.toJSON())} gas multiplier &times; ${
              //   JSON.stringify(a.toJSON())
              // } atomic gas price = ${JSON.stringify(result.toJSON())} atomic gas units`],
            ],
            [
              `Gas (${self.gasPrice.minimalDenom})`,
              "&times;",
              `${BigDecimal.format(self.gasPrice.value)}`,
              //     `${JSON.stringify(gasUnits.toJSON())} gas multiplier &times; ${
              //   JSON.stringify(a.toJSON())
              // } atomic gas price = ${JSON.stringify(result.toJSON())} atomic gas units`],
            ],
          ],
        ]
      }

      return multiply(self.gasPrice.value)
    }

    const applyAdditiveFeeK = (
      amount: AtomicGasPrice,
    ): Writer.Writer<readonly string[][], AtomicGasPrice> => {
      const result = self.gasPrice.additiveFee.pipe(
        O.map(BigDecimal.sum(amount)),
        O.getOrElse(() => amount),
        AtomicGasPrice,
      )

      return [
        result,
        [
          [
            `Settlement`,
            `+`,
            `${
              BigDecimal.format(
                O.getOrElse(self.gasPrice.additiveFee, () => BigDecimal.make(0n, 0)),
              )
            }`,
          ],
        ],
      ]
    }

    const applyRatioK = (a: BaseGasPrice): Writer.Writer<readonly string[][], BaseGasPrice> => {
      const ratio = BigDecimal.round(
        BigDecimal.unsafeDivide(BigDecimal.make(1n, 0), self.ratio),
        { scale: 6, mode: "half-even" },
      )
      const result = pipe(
        BigDecimal.multiply(a, ratio),
        BaseGasPrice,
      )

      return [
        result,
        [
          [
            `${_destSymbol} &rarr; ${_sourceSymbol}`,
            "&times;",
            `${BigDecimal.format(ratio)}`,
            // `${JSON.stringify(a.toJSON())} &divide; ${JSON.stringify(ratio.toJSON())} scalar = ${
            //   JSON.stringify(result.toJSON())
            // } base gas units`,
          ],
        ],
      ]
    }

    const applyFeeMultiplierK = (
      a: BaseGasPrice,
    ): Writer.Writer<readonly string[][], BaseGasPrice> => {
      const result = pipe(
        BigDecimal.multiply(a, self.feeMultiplier),
        BaseGasPrice,
      )
      return [
        result,
        [
          [
            "Relaying",
            "&times;",
            `${BigDecimal.format(self.feeMultiplier)}`,
            // `${JSON.stringify(a.toJSON())} base gas units × ${
            //   JSON.stringify(self.feeMultiplier.toJSON())
            // } scalar = ${JSON.stringify(result.toJSON())} base gas units`,
          ],
        ],
      ]
    }

    const applyBatchDivisionK = (
      a: BaseGasPrice,
    ): [BaseGasPrice, readonly string[][]] => {
      const result = pipe(
        BigDecimal.unsafeDivide(a, self.batchDivideNumber),
        BaseGasPrice,
      )
      return [
        result,
        [
          [
            `<span class="batch-savings">Batch Savings</span>`,
            `<span class="batch-savings">&divide;</span>`,
            `<span class="batch-savings">${BigDecimal.format(self.batchDivideNumber)}</span>`,
            // `${JSON.stringify(a.toJSON())} base gas units &divide; ${
            //   JSON.stringify(self.batchDivideNumber.toJSON())
            // } scalar`,
          ],
        ],
      ]
    }

    const formatToDisplayK = (a: BaseGasPrice): Writer.Writer<readonly string[][], string> => {
      const round = (x: BaseGasPrice): [BaseGasPrice, readonly string[][]] => {
        const scale = 10
        const mode = "half-even"
        const result = pipe(
          BigDecimal.round(x, { scale, mode }),
          BaseGasPrice,
        )
        return [
          result,
          [
            [
              `Subtotal`,
              "=",
              `${BigDecimal.format(result)}`,
              // `<code>round</code><sub>${scale}</sub>(${
              //   JSON.stringify(a.toJSON())
              // }) base gas unit with mode <i>${mode}</i> = ${result}`,
            ],
          ],
        ]
      }

      const format = (x: BaseGasPrice): Writer.Writer<readonly string[][], string> => {
        const result = BigDecimal.format(x)
        return [
          result,
          [
            // [
            // `<code>format</code>(${JSON.stringify(x.toJSON())})`
            // ],
          ],
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
      applyAdditiveFeeK,
      applyFeeMultiplierK,
      applyRatioK,
      applyBatchDivisionK,
      formatToDisplayK,
    })
  }

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

  const destSymbol = $derived(O.gen(function*() {
    const chain = yield* TransferData.destinationChain
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
    data,
    O.flatMap(O.getRight),
    O.map(x => x.source.gas),
  ))

  const decoratedConfig = $derived(pipe(
    O.all({
      // XXX: source / dest here should be determined on per-transaction basis
      gasPrice: pipe(
        data,
        O.flatMap(O.getRight),
        O.map(x => ({
          value: x.destination.gas,
          decimals: x.destination.gasDecimals,
          minimalDenom: x.destination.gasMinimalDenom,
          denom: x.destination.gasDenom,
          additiveFee: x.destination.additiveFee,
        })),
      ),
      ratio: maybeRatio,
    }),
    O.map(({ gasPrice, ratio }) => decorate({ gasPrice, ratio, ...config })),
  ))

  // TODO: tuple-ify outputs; concatenate to show record of calculations performed
  const calculatedFees = $derived(pipe(
    O.all({ fees: baseFees, config: decoratedConfig }),
    O.map(({ config, fees }) =>
      Struct.evolve(fees, {
        PACKET_SEND_LC_UPDATE_L1: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.applyAdditiveFeeK),
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
            composeK(config.applyBatchDivisionK),
            composeK(config.applyRatioK),
          ),
        ),
        PACKET_SEND_LC_UPDATE_L0: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.applyAdditiveFeeK),
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
            composeK(config.applyBatchDivisionK),
            composeK(config.applyRatioK),
          ),
        ),
        PACKET_RECV: flow(
          pipe( // TODO: extract
            config.applyGasPriceK,
            composeK(config.applyAdditiveFeeK),
            composeK(config.asBaseUnitK),
            composeK(config.applyFeeMultiplierK),
            composeK(config.applyBatchDivisionK), // This should be removed later
            composeK(config.applyRatioK),
          ),
        ),
      })
    ),
  ))

  const displayFees = $derived(pipe(
    O.all({ calculatedFees, decoratedConfig }),
    O.map(({ calculatedFees, decoratedConfig: { formatToDisplayK } }) =>
      Struct.evolve(calculatedFees, {
        PACKET_SEND_LC_UPDATE_L0: (x) => {
          const f = pipe(
            formatToDisplayK,
          )

          const g = composeK(identity<typeof x>, f)
          const h = g(x)
          return h
        },
        PACKET_SEND_LC_UPDATE_L1: (x) =>
          composeK(
            () => x,
            pipe(
              formatToDisplayK,
            ),
          )(x),
        PACKET_RECV: (x) =>
          composeK(
            () => x,
            pipe(
              formatToDisplayK,
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
        const amount = yield* O.map(displayFees, flow(Struct.get(x.key), Tuple.getFirst))
        const baseFee = yield* O.map(calculatedFees, flow(Struct.get(x.key), Tuple.getFirst))
        const calc = yield* O.map(displayFees, flow(Struct.get(x.key), (x) => x[1]))

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
    const extractError = <E>(x: O.Option<E.Either<any, E>>) =>
      pipe(
        x,
        O.flatMap(E.getLeft),
      )
    return pipe(
      [
        extractError(data),
      ] as const,
      A.getSomes,
      Unify.unify,
      A.map(Cause.originalError),
      A.map(x => (x as any)?.message),
      A.filter(Predicate.isNotUndefined),
    )
  })

  const usdSources = $derived(pipe(
    data,
    O.flatMap(O.getRight),
    O.map(x => ({
      source: x.source.usd.source,
      destination: x.destination.usd.source,
    })),
  ))

  /** total in base source chain */
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
      perUsd: pipe(
        data,
        O.flatMap(O.getRight),
        O.map(x => x.source.usd.price),
      ),
      totalFee,
    }),
    O.map(({ perUsd, totalFee }) => BigDecimal.multiply(totalFee, perUsd)),
    O.map(BigDecimal.round({
      scale: 4,
      mode: "half-even",
    })),
    O.map(BigDecimal.format),
  ))

  const intent: O.Option<E.Either<FeeIntent, string>> = $derived(O.gen(function*() {
    const _totalFee = yield* totalFee
    const sourceChain = yield* TransferData.sourceChain

    return E.gen(function*() {
      const { decimals, address: baseToken } = yield* pipe(
        R.get(GAS_DENOMS, sourceChain.universal_chain_id),
        E.fromOption(() => `No gas denom for ${sourceChain.universal_chain_id}`),
      )
      const baseAmount = BigDecimal.round(_totalFee, { scale: decimals, mode: "half-even" })
      return {
        decimals,
        baseToken,
        quoteAmount: TokenRawAmount.make(0n),
        baseAmount: TokenRawAmount.make(baseAmount.value),
      } as const
    })
  }))

  return {
    get intent() {
      return intent
    },
    get baseFees() {
      return baseFees
    },
    get sourceGasUnitPrice() {
      return sourceGasUnitPrice
    },
    get feeBreakdown() {
      return newFeeBreakdown
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
    get usdSources() {
      return usdSources
    },
    /**
     * Total cost in gas token symbol.
     */
    get feeDisplay() {
      return pipe(
        O.all({ totalFee, decoratedConfig }),
        O.map(({ totalFee, decoratedConfig: { formatToDisplayK } }) =>
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
    get isReady(): boolean {
      return pipe(
        O.all([
          TransferData.sourceChain,
          TransferData.destinationChain,
          TransferData.channel,
        ]),
        O.isSome,
      )
    },
    get isLoading(): boolean {
      return O.isNone(data)
    },
    get init() {
      return fetchFee
    },
    get errors() {
      return errors
    },
  } as const
}

export const FeeStore = createFeeStore()
