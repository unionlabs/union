import { Effect, pipe } from "effect"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as Evm from "./Evm.js"
import { graphqlQuoteTokenUnwrapQuery } from "./graphql/unwrapped-quote-token.js"
import { UniversalChainId } from "./schema/chain.js"
import { ChannelId } from "./schema/channel.js"
import { Hex } from "./schema/hex.js"
import { TokenRawDenom } from "./schema/token.js"
import * as Ucs03 from "./Ucs03.js"
import * as Ucs05 from "./Ucs05.js"
import * as Utils from "./Utils.js"

const BaseIntent = S.Struct({
  baseAmount: S.BigIntFromSelf.pipe(
    S.greaterThanBigInt(0n),
  ),
  quoteAmount: S.BigIntFromSelf,
  sourceChainId: UniversalChainId,
  sourceChannelId: ChannelId,
})
type BaseIntent = typeof BaseIntent.Type

export const XAS = S.Struct({
  sender: Ucs05.AddressEvmZkgm,
  baseToken: TokenRawDenom,
  sourceChainId: UniversalChainId,
  sourceChannelId: ChannelId,
})
export type XAS = typeof XAS.Type

export const YAS = S.Struct({
  receiver: Ucs05.AddressEvmZkgm,
})
export type YAS = typeof YAS.Type

const XA = (xas: XAS) => (base: BaseIntent) =>
  Effect.gen(function*() {
    const sourceClient = (yield* Evm.PublicClientSource).client
    const tokenMeta = yield* Evm.readErc20Meta(
      xas.baseToken as unknown as any,
      xas.sourceChainId,
    ).pipe(
      Effect.provideService(Evm.PublicClient, { client: sourceClient }),
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: Utils.ensureHex(xas.baseToken),
      sourceChainId: xas.sourceChainId,
      sourceChannelId: xas.sourceChannelId,
    })

    const finalQuoteToken = yield* O.match(graphqlDenom, {
      onNone: () => Evm.predictQuoteToken(xas.baseToken),
      onSome: Effect.succeed,
    })

    const path = O.match(graphqlDenom, {
      onNone: () => 0n,
      onSome: () => xas.sourceChannelId,
    })

    return [
      O.some(xas.sender),
      O.none(), // receiver
      O.some(Utils.ensureHex(xas.baseToken)),
      O.some(base.baseAmount),
      O.some(tokenMeta.symbol),
      O.some(tokenMeta.name),
      O.some(tokenMeta.decimals),
      O.some(path), // path is source channel when unwrapping, else 0
      O.some(finalQuoteToken),
      O.some(base.quoteAmount),
    ] as const
  })

const YA = (yas: YAS) => (base: BaseIntent) =>
  Effect.succeed(
    [
      O.none(),
      O.some(yas.receiver),
      O.none(),
      O.none(),
      O.none(),
      O.none(),
      O.none(),
      O.none(),
      O.none(),
      O.none(),
    ] as const,
  )

const combine = (xas: XAS, yas: YAS) => (base: BaseIntent) =>
  pipe(
    Effect.all([
      XA(xas)(base),
      YA(yas)(base),
    ]),
    Effect.map([x, y] => O.all),
    x => 
  )
