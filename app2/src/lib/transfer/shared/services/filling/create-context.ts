import type { FeeIntent } from "$lib/stores/fee.svelte.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { uiStore } from "$lib/stores/ui.svelte.ts"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import { Batch, Token, TokenOrder, Ucs05, Utils, ZkgmClientRequest } from "@unionlabs/sdk"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms.ts"
import { graphqlQuoteTokenUnwrapQuery } from "@unionlabs/sdk/graphql/unwrapped-quote-token.js"
import type {
  Chain,
  Channel,
  ChannelId,
  TokenRawAmount,
  TokenRawDenom,
  UniversalChainId,
} from "@unionlabs/sdk/schema"
import { Effect, Match, Option, ParseResult, pipe } from "effect"
import * as A from "effect/Array"
import type { NoSuchElementException, UnknownException } from "effect/Cause"
import * as R from "effect/Record"
import * as S from "effect/Schema"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling.ts"

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
    console.debug("[createContext] args:", args)

    if (args.baseToken.address === "au") {
      args.quoteToken = Token.Erc20.make({ address: "0xba5eD44733953d79717F6269357C77718C8Ba5ed" })
    }

    const kind = args.sourceChain.addr_prefix === "union"
      ? TokenOrder.Kind.Solve
      : args.kind
    const metadata = args.sourceChain.addr_prefix === "union"
      ? "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
      : undefined

    const sendOrder = yield* TokenOrder.make({
      baseAmount: Option.getOrThrow(parseBaseAmount(args.baseAmount)),
      baseToken: args.baseToken,
      quoteToken: args.quoteToken,
      quoteAmount: Option.getOrThrow(parseBaseAmount(args.quoteAmount)),
      destination: args.destinationChain,
      receiver: args.receiver,
      sender: args.sender,
      kind,
      source: args.sourceChain,
      metadata,
    })

    const maybeFeeQuoteToken = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: Utils.ensureHex(args.fee.baseToken.address),
      sourceChainId: args.sourceChain.universal_chain_id,
      sourceChannelId: args.sourceChannelId,
    })

    // on destination chain tokens, find wrappings[] such that one exists where unwrapped_denom matches basetoken and unwrapped_chain and wrapped_chain universal ids match
    const feeQuoteToken = yield* maybeFeeQuoteToken.pipe(
      Option.orElse(() =>
        pipe(
          tokensStore.getData(args.destinationChain.universal_chain_id),
          Option.flatMap(
            A.findFirst((token) =>
              A.filter(token.wrapping, (x) =>
                x.unwrapped_denom === args.fee.baseToken.address
                && x.unwrapped_chain.universal_chain_id === args.sourceChain.universal_chain_id
                && x.wrapped_chain.universal_chain_id === args.destinationChain.universal_chain_id)
                .length
                === 1
            ),
          ),
          Option.map(x => x.denom),
        )
      ),
      Option.orElse(() => {
        if (args.baseToken.address === "au") {
          return Option.some(args.quoteToken)
        }
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
      kind,
      source: args.sourceChain,
      metadata,
    })

    const batch = Batch.make([sendOrder, feeOrder])

    console.log("[createContext]", { batch })

    const request = ZkgmClientRequest.make({
      channelId: args.sourceChannelId,
      destination: args.destinationChain,
      source: args.sourceChain,
      instruction: batch,
      ucs03Address: args.ucs03address,
    }).pipe(
      Option.some,
    )

    const ctx = yield* parseBaseAmount(args.baseAmount).pipe(
      Option.flatMap((baseAmount) => {
        const intents = createIntents(args, baseAmount)

        return intents.length > 0
          ? Option.some({
            intents,
            funds: calculateNativeValue(intents, args),
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
  return Match.value(edition).pipe(
    Match.when("btc", () => sourceChain.universal_chain_id === "babylon.bbn-1"),
    Match.orElse(() => true),
  )
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

const calculateNativeValue = (
  intents: Intent[],
  args: TransferArgs,
): Option.Option<Array<{ baseToken: TokenRawDenom | string; amount: TokenRawAmount }>> => {
  const nativeTokensMap = new Map<string, bigint>()

  const chainGasDenom = GAS_DENOMS[args.sourceChain.universal_chain_id]

  for (const intent of intents) {
    const normalizedToken = normalizeToken(intent.baseToken, args.sourceChain.rpc_type)

    const isNativeToken = args.sourceChain.rpc_type === "evm"
      ? chainGasDenom
        && normalizedToken === normalizeToken(chainGasDenom.address, args.sourceChain.rpc_type)
      : !isValidBech32ContractAddress(normalizedToken)

    if (isNativeToken) {
      const currentAmount = nativeTokensMap.get(normalizedToken) || 0n
      nativeTokensMap.set(normalizedToken, currentAmount + intent.baseAmount)
    }
  }

  // Convert map to array
  const nativeFunds = Array.from(nativeTokensMap.entries()).map(([token, amount]) => ({
    baseToken: token,
    amount: amount as TokenRawAmount,
  }))

  return nativeFunds.length > 0 ? Option.some(nativeFunds) : Option.none()
}
