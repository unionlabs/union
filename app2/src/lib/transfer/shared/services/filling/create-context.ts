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
import { Effect, Either, flow, Match, Option, ParseResult, pipe } from "effect"
import * as A from "effect/Array"
import type { NoSuchElementException, UnknownException } from "effect/Cause"
import * as S from "effect/Schema"
import { fromHex, isHex } from "viem"
import { GenericFlowError } from "../../errors"
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
  ParseResult.ParseError | GenericFlowError | UnknownException | NoSuchElementException,
  never
> =>
  Effect.gen(function*() {
    console.log("[createContext] args:", args)

    const baseAmount = yield* parseBaseAmount(args.baseAmount).pipe(
      Effect.mapError((cause) =>
        new GenericFlowError({
          message: "Could not parse base amount",
          cause,
        })
      ),
    )

    const quoteAmount = yield* parseBaseAmount(args.quoteAmount).pipe(
      Effect.mapError((cause) =>
        new GenericFlowError({
          message: "Could not parse quote amount",
          cause,
        })
      ),
    )

    const sendOrder = yield* TokenOrder.make({
      baseAmount,
      baseToken: args.baseToken,
      quoteToken: args.quoteToken,
      quoteAmount,
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
    const encodedFeeBaseToken = yield* pipe(
      args.fee.baseToken,
      S.encode(Token.AnyFromEncoded(args.sourceChain.rpc_type)),
      Effect.mapError((cause) =>
        new GenericFlowError({
          message: "Could not base token",
          cause,
        })
      ),
    )

    const shouldIncludeFees = shouldChargeFees(
      args.fee,
      uiStore.edition,
      args.sourceChain,
      args.destinationChain,
    )

    console.log({ shouldIncludeFees })

    const produceBatch = Effect.gen(function*() {
      if (shouldIncludeFees) {
        const feeQuoteToken = yield* Effect.if(args.baseToken.address === "au", {
          onTrue: () => Effect.succeed(args.quoteToken),
          onFalse: () =>
            pipe(
              maybeFeeQuoteToken,
              Either.fromOption(() => "No fee quote token"),
              Either.orElse(() =>
                pipe(
                  tokensStore.getData(args.destinationChain.universal_chain_id),
                  Either.fromOption(() => "No matching token in token store"),
                  Either.flatMap(flow(
                    A.findFirst((token) =>
                      A.filter(token.wrapping, (x) =>
                        x.unwrapped_denom === encodedFeeBaseToken
                        && x.unwrapped_chain.universal_chain_id
                          === args.sourceChain.universal_chain_id
                        && x.wrapped_chain.universal_chain_id
                          === args.destinationChain.universal_chain_id)
                        .length
                        === 1
                    ),
                    Either.fromOption(() =>
                      `No quote token wrapping found for ${args.destinationChain.universal_chain_id} given ${args.fee.baseToken}`
                    ),
                  )),
                  Either.map(x => x.denom),
                  Either.flatMap((raw) =>
                    S.decodeEither(Token.AnyFromEncoded(args.destinationChain.rpc_type))(raw)
                  ),
                )
              ),
              Effect.mapError((cause) =>
                new GenericFlowError({
                  message: "Could not determine fee quote token",
                  cause,
                })
              ),
            ),
        })

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

        return pipe(
          Batch.make([sendOrder, feeOrder]),
          Batch.optimize,
        )
      } else {
        return sendOrder
      }
    })

    const batch = yield* produceBatch

    // TODO: vaultId & ibcStoreId will be fetched from hubble probably
    const SUI_HARDCODED_TRANSPORT = {
      sui: {
        vaultId: "0xc3ac8618f622fc70ea30eaec5b45d504e239af668033d07e396be44d45f8f45d",
        ibcStoreId: "0xdc5f20df5f143a06772c073e9c30dacd30e31f6788885cf478d0fd40f92766c4",
        coins: [
          {
            typeArg: args.baseToken.address,
            baseAmount: Option.getOrThrow(parseBaseAmount(args.baseAmount)),
          },
        ],
      },
    }

    // const maybeTransport =
    //   args.transport?.sui && args.sourceChain.rpc_type === "sui"
    //     ? { sui: args.transport.sui }
    //     : undefined
    const maybeTransport = args.sourceChain.rpc_type === "sui" ? SUI_HARDCODED_TRANSPORT : undefined

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

    const ctx = yield* pipe(
      createIntents(args, baseAmount),
      (intents) =>
        intents.length > 0
          ? Option.some({
            intents,
            allowances: Option.none(),
            request: Option.none(),
            message: Option.none(),
          })
          : Option.none(),
    )

    return {
      ...ctx,
      request,
    } as const
  })
)

const createIntents = (args: TransferArgs, baseAmount: TokenRawAmount): Intent[] => {
  const shouldIncludeFees = shouldChargeFees(
    args.fee,
    uiStore.edition,
    args.sourceChain,
    args.destinationChain,
  )
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
const shouldChargeFees = (
  fee: FeeIntent,
  edition: string,
  sourceChain: Chain,
  destinationChain: Chain,
): boolean => {
  if (fee.baseAmount === 0n) {
    return false
  }
  if (sourceChain.testnet) {
    return true
  }
  // skip fees babylon to bsc for now
  if (
    destinationChain.universal_chain_id === "bsc.56"
    && sourceChain.universal_chain_id === "babylon.bbn-1"
  ) {
    return false
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
