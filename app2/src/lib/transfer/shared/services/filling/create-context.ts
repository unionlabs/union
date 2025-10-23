import type { FeeIntent } from "$lib/stores/fee.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { Batch, Token, TokenOrder, Ucs05, Utils, ZkgmClientRequest } from "@unionlabs/sdk"
import { graphqlQuoteTokenUnwrapQuery } from "@unionlabs/sdk/graphql/unwrapped-quote-token.js"
import type {
  Chain,
  Channel,
  ChannelId,
  TokenRawAmount,
  UniversalChainId,
} from "@unionlabs/sdk/schema"
import { Effect, Match, Option, ParseResult, pipe } from "effect"
import * as A from "effect/Array"
import type { NoSuchElementException, UnknownException } from "effect/Cause"
import * as S from "effect/Schema"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling"

export type Intent = {
  sender: Ucs05.AnyDisplay
  receiver: Ucs05.AnyDisplay
  baseToken: Token.Any
  baseAmount: TokenRawAmount
  quoteAmount: TokenRawAmount
  decimals: number
  sourceChain: Chain
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
  kind: TokenOrder.Kind
  quoteToken: Token.Any
  destinationChain: Chain
  channel: Channel
  ucs03address: string
}

export type Allowance = {
  token: Token.Any
  requiredAmount: bigint
  currentAllowance: bigint
}

export type TransferContext = {
  intents: Array<Intent>
  allowances: Option.Option<A.NonEmptyReadonlyArray<Allowance>>
  request: Option.Option<ZkgmClientRequest.ZkgmClientRequest>
  message: Option.Option<string>
}

export const createContext = Effect.fn((
  args: TransferArgs,
): Effect.Effect<
  TransferContext,
  NoSuchElementException | ParseResult.ParseError | UnknownException,
  never
> =>
  Effect.gen(function*() {

    console.log("[createContext] args:", args)

    const sendOrder = yield* TokenOrder.make({
      baseAmount: Option.getOrThrow(parseBaseAmount(args.baseAmount)),
      baseToken: args.baseToken,
      quoteToken: args.quoteToken,
      quoteAmount: Option.getOrThrow(parseBaseAmount(args.quoteAmount)),
      destination: args.destinationChain,
      receiver: args.receiver,
      sender: args.sender,
      kind: args.kind,
      source: args.sourceChain,
      metadata: args.metadata,
      version: args.version,
    })

    const maybeFeeQuoteToken = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: Utils.ensureHex(args.fee.baseToken.address),
      sourceChainId: args.sourceChain.universal_chain_id,
      sourceChannelId: args.sourceChannelId,
    })

    // on destination chain tokens, find wrappings[] such that one exists where unwrapped_denom matches basetoken and unwrapped_chain and wrapped_chain universal ids match
    const encodedFeeBaseToken = S.encodeSync(Token.AnyFromEncoded(args.sourceChain.rpc_type))(
      args.fee.baseToken,
    )

    const shouldIncludeFees = shouldChargeFees(args.fee, uiStore.edition, args.sourceChain)

    const produceBatch = Effect.gen(function*() {
      if (shouldIncludeFees) {
        const feeQuoteToken = yield* maybeFeeQuoteToken.pipe(
          Option.orElse(() =>
            pipe(
              tokensStore.getData(args.destinationChain.universal_chain_id),
              Option.flatMap(
                A.findFirst((token) =>
                  A.filter(token.wrapping, (x) =>
                    x.unwrapped_denom === encodedFeeBaseToken
                    && x.unwrapped_chain.universal_chain_id === args.sourceChain.universal_chain_id
                    && x.wrapped_chain.universal_chain_id
                      === args.destinationChain.universal_chain_id)
                    .length
                    === 1
                ),
              ),
              Option.map(x => x.denom),
              Option.flatMap((raw) =>
                S.decodeOption(Token.AnyFromEncoded(args.destinationChain.rpc_type))(raw)
              ),
            )
          ),
          Option.orElse(() => {
            if (args.baseToken.address === "au") {
              return Option.some(args.quoteToken)
            }
            console.error("Could not determine fee quote token.")
            return Option.none()
          }),
        )

        const feeOrder = yield* TokenOrder.make({
          baseAmount: args.fee.baseAmount,
          baseToken: args.fee.baseToken,
          quoteToken: feeQuoteToken,
          quoteAmount: args.fee.quoteAmount,
          destination: args.destinationChain,
          receiver: args.receiver,
          sender: args.sender,
          kind: "escrow",
          source: args.sourceChain,
          metadata: undefined,
          version: args.version,
        })

        return Batch.make([sendOrder, feeOrder]).pipe(
          Batch.optimize,
        )
      } else {
        return sendOrder
      }
    })

    const batch = yield* produceBatch

    const maybeTransport =
      args.transport?.sui && args.sourceChain.rpc_type === "sui"
        ? { sui: args.transport.sui }
        : undefined
        
    const request = ZkgmClientRequest.make({
      channelId: args.sourceChannelId,
      destination: args.destinationChain,
      source: args.sourceChain,
      instruction: batch,
      ucs03Address: args.ucs03address,
      ...(maybeTransport ? { transport: maybeTransport } : {}),
    }).pipe(
      Option.some,
    )

    const ctx = yield* parseBaseAmount(args.baseAmount).pipe(
      Option.flatMap((baseAmount) => {
        const intents = createIntents(args, baseAmount)

        return intents.length > 0
          ? Option.some({
            intents,
            allowances: Option.none(),
            request: Option.none(),
            message: Option.none(),
          })
          : Option.none()
      }),
    )

    return {
      ...ctx,
      request,
    } as const
  })
)

const createIntents = (args: TransferArgs, baseAmount: TokenRawAmount): Intent[] => {
  const shouldIncludeFees = shouldChargeFees(args.fee, uiStore.edition, args.sourceChain)
  const baseIntent = createBaseIntent(args, baseAmount)

  return Match.value(args.sourceChain.rpc_type).pipe(
    Match.when("evm", () => {
      const intent: Intent = {
        ...baseIntent,
        baseToken: args.baseToken,
        kind: args.kind,
        quoteToken: args.quoteToken,
      }

      const feeIntent: Intent = {
        ...baseIntent,
        baseToken: args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
        kind: args.kind,
        quoteToken: args.quoteToken,
      }

      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),
    Match.when("cosmos", () => {
      const intent: Intent = {
        ...baseIntent,
        baseToken: args.baseToken,
        kind: args.kind,
        quoteToken: args.quoteToken,
        // baseToken: normalizeToken(args.baseToken.address, "cosmos"),
      }

      const feeIntent: Intent = {
        ...baseIntent,
        // baseToken: normalizeToken(args.fee.baseToken, "cosmos"),
        baseToken: args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
        kind: args.kind,
        quoteToken: args.quoteToken,
      }

      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),
    Match.when("sui", () => {
      const intent: Intent = {
        ...baseIntent,
        baseToken: args.baseToken,
        kind: args.kind,
        quoteToken: args.quoteToken,
      }

      const feeIntent: Intent = {
        ...baseIntent,
        baseToken: args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
        kind: args.kind,
        quoteToken: args.quoteToken,
      }
      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),

    Match.orElse(() => []),
  )
}

const createBaseIntent = (
  args: TransferArgs,
  baseAmount: TokenRawAmount,
): Omit<Intent, "baseToken" | "quoteToken" | "kind"> => ({
  sender: args.sender,
  receiver: args.receiver,
  baseAmount,
  quoteAmount: baseAmount,
  decimals: args.decimals,
  sourceChain: args.sourceChain,
  sourceChainId: args.sourceChain.universal_chain_id,
  sourceChannelId: args.channel.source_channel_id,
  destinationChain: args.destinationChain,
  channel: args.channel,
  ucs03address: args.ucs03address,
})

// Fee strategy: BTC edition only charges fees when going FROM Babylon
const shouldChargeFees = (fee: FeeIntent, edition: string, sourceChain: Chain): boolean => {
  if (fee.baseAmount === 0n) {
    return false
  }
  if (sourceChain.testnet) {
    return true
  }
  return sourceChain.universal_chain_id === "babylon.bbn-1"
}

const normalizeToken = (token: string | `0x${string}`, rpcType: string): string => {
  return rpcType === "cosmos" && isHex(token) ? fromHex(token, "string") : token
}

const parseBaseAmount = (amount: string): Option.Option<TokenRawAmount> => {
  return Option.fromNullable(amount)
    .pipe(
      Option.filter(str => str.trim() !== ""),
      Option.filter(str => /^\d+$/.test(str.trim())),
      Option.map(str => BigInt(str.trim()) as TokenRawAmount),
    )
}
