import { Effect, Inspectable, Match, Predicate, Schema } from "effect"
import * as B from "effect/Boolean"
import { dual, pipe } from "effect/Function"
import { ParseError } from "effect/ParseResult"
import { pipeArguments } from "effect/Pipeable"
import * as S from "effect/Schema"
import { Chain } from "../schema/chain.js"
import { Hex } from "../schema/hex.js"
import * as Token from "../Token.js"
import * as TokenOrder from "../TokenOrder.js"
import * as Ucs03 from "../Ucs03.js"
import * as Ucs05 from "../Ucs05.js"
import * as Utils from "../Utils.js"

/** @internal */
export const TypeId: TokenOrder.TypeId = Symbol.for(
  "@unionlabs/sdk/TokenOrder",
) as TokenOrder.TypeId

const Proto = {
  [TypeId]: TypeId,
  ...Inspectable.BaseProto,
  toJSON(this: TokenOrder.TokenOrder): unknown {
    return {
      _id: "@unionlabs/sdk/TokenOrder",
      _missing: Object.entries(this)
        .filter(([_k, v]) => typeof v === "undefined")
        .map(([k, _v]) => k),
      source: this.source,
      destination: this.destination,
      sender: this.sender,
      receiver: this.receiver,
      baseToken: this.baseToken,
      baseAmount: this.baseAmount.toString(),
      quoteToken: this.quoteToken,
      quoteAmount: this.quoteAmount.toString(),
      kind: this.kind,
      metadata: this.metadata,
      opcode: this.opcode,
      version: this.version,
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

function makeProto(
  source: Chain,
  destination: Chain,
  sender: Ucs05.AnyDisplay,
  receiver: Ucs05.AnyDisplay,
  baseToken: Token.Any,
  baseAmount: bigint,
  quoteToken: Token.Any,
  quoteAmount: bigint,
  kind: TokenOrder.Kind,
  metadata?: Hex | undefined,
  version?: 1 | 2 | undefined,
): TokenOrder.TokenOrder {
  const self = Object.create(Proto)
  self._tag = "TokenOrder"
  self.source = source
  self.destination = destination
  self.sender = sender
  self.receiver = receiver
  self.baseToken = baseToken
  self.baseAmount = baseAmount
  self.quoteToken = quoteToken
  self.quoteAmount = quoteAmount
  self.kind = kind
  self.metadata = metadata
  self.version = version ?? 2
  self.opcode = 3
  return self
}

/** @internal */
export const isTokenOrder = (u: unknown): u is TokenOrder.TokenOrder =>
  typeof u === "object" && u !== null && TypeId in u

/** @internal */
export const empty: TokenOrder.TokenOrder = makeProto(
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  void 0 as unknown as Ucs05.AnyDisplay,
  void 0 as unknown as Ucs05.AnyDisplay,
  void 0 as unknown as Token.Any,
  void 0 as unknown as bigint,
  void 0 as unknown as Token.Any,
  void 0 as unknown as bigint,
  void 0 as unknown as TokenOrder.Kind,
  undefined,
  2,
)

/** @internal */
export const make = (
  source: Chain,
  destination: Chain,
  sender: Ucs05.AnyDisplay | string,
  receiver: Ucs05.AnyDisplay | string,
  baseToken: Token.Any,
  baseAmount: bigint,
  quoteToken: Token.Any,
  quoteAmount: bigint,
  kind: TokenOrder.Kind,
  metadata?: Hex | undefined,
  version?: 1 | 2 | undefined,
) =>
  modify(empty, {
    source,
    destination,
    sender,
    receiver,
    baseToken,
    baseAmount,
    quoteToken,
    quoteAmount,
    kind,
    metadata,
    version,
  })

/** @internal */
export const modify = dual<
  (
    options: TokenOrder.Options.Complete,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    options: TokenOrder.Options.Complete,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError, never>
>(2, (self, options) =>
  Effect.gen(function*() {
    let result = self

    if (options.source) {
      const source = yield* Schema.decodeUnknown(TokenOrder.Input.fields.source)(options.source)
      result = setSource(result, source)
    }

    if (options.destination) {
      const destination = yield* Schema.decodeUnknown(TokenOrder.Input.fields.destination)(
        options.destination,
      )
      result = setDestination(result, destination)
    }

    if (options.sender) {
      // XXX: remove assertion
      result = yield* setSender(result, options.sender as any)
    }

    if (options.receiver) {
      // XXX: remove assertion
      result = yield* setReceiver(result, options.receiver as any)
    }

    if (options.baseToken) {
      result = yield* setBaseToken(result, options.baseToken as unknown as any)
    }

    if (Predicate.isBigInt(options.baseAmount)) {
      result = yield* setBaseAmount(result, options.baseAmount)
    }

    if (options.quoteToken) {
      result = yield* setQuoteToken(result, options.quoteToken as unknown as any)
    }

    if (Predicate.isBigInt(options.quoteAmount)) {
      result = yield* setQuoteAmount(result, options.quoteAmount)
    }

    if (options.kind !== undefined) {
      result = {
        ...result,
        kind: options.kind,
      }
    }

    if (options.metadata !== undefined) {
      result = {
        ...result,
        metadata: options.metadata,
      }
    }

    if (options.version !== undefined) {
      result = {
        ...result,
        version: options.version,
      }
    }

    // result.encode = encode(result)

    return result
  }))

/** @internal */
export const setSource = dual<
  (
    source: Chain,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, source: Chain) => TokenOrder.TokenOrder
>(2, (self, source) =>
  makeProto(
    source,
    self.destination,
    self.sender,
    self.receiver,
    self.baseToken,
    self.baseAmount,
    self.quoteToken,
    self.quoteAmount,
    self.kind,
    self.metadata,
  ))

/** @internal */
export const setDestination = dual<
  (
    destination: Chain,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, destination: Chain) => TokenOrder.TokenOrder
>(2, (self, destination) =>
  makeProto(
    self.source,
    destination,
    self.sender,
    self.receiver,
    self.baseToken,
    self.baseAmount,
    self.quoteToken,
    self.quoteAmount,
    self.kind,
    self.metadata,
  ))

/** @internal */
export const setSender = dual<
  (
    sender: Ucs05.AnyDisplay | string,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    sender: Ucs05.AnyDisplay | string,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, sender) =>
  pipe(
    sender,
    Schema.decode(TokenOrder.Input.fields.sender),
    Effect.map((sender) =>
      makeProto(
        self.source,
        self.destination,
        sender,
        self.receiver,
        self.baseToken,
        self.baseAmount,
        self.quoteToken,
        self.quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
export const setReceiver = dual<
  (
    receiver: Ucs05.AnyDisplay | string,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    receiver: Ucs05.AnyDisplay | string,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, receiver) =>
  pipe(
    receiver,
    Schema.decode(TokenOrder.Input.fields.receiver),
    Effect.map((receiver) =>
      makeProto(
        self.source,
        self.destination,
        self.sender,
        receiver,
        self.baseToken,
        self.baseAmount,
        self.quoteToken,
        self.quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
export const setBaseToken = dual<
  (
    baseToken: Token.Any | string,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    baseToken: Token.Any | string,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, baseToken) =>
  pipe(
    baseToken,
    Schema.decode(TokenOrder.Input.fields.baseToken),
    Effect.map((baseToken) =>
      makeProto(
        self.source,
        self.destination,
        self.sender,
        self.receiver,
        baseToken,
        self.baseAmount,
        self.quoteToken,
        self.quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
export const setBaseAmount = dual<
  (
    baseAmount: bigint,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    baseAmount: bigint,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, baseAmount) =>
  pipe(
    baseAmount,
    Schema.decode(TokenOrder.Input.fields.baseAmount),
    Effect.map((baseAmount) =>
      makeProto(
        self.source,
        self.destination,
        self.sender,
        self.receiver,
        self.baseToken,
        baseAmount,
        self.quoteToken,
        self.quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
export const setQuoteToken = dual<
  (
    quoteToken: string | Token.Any,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    quoteToken: string | Token.Any,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, quoteToken) =>
  pipe(
    quoteToken,
    Schema.decode(TokenOrder.Input.fields.quoteToken),
    Effect.map((quoteToken) =>
      makeProto(
        self.source,
        self.destination,
        self.sender,
        self.receiver,
        self.baseToken,
        self.baseAmount,
        quoteToken,
        self.quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
export const setQuoteAmount = dual<
  (
    quoteAmount: bigint,
  ) => (self: TokenOrder.TokenOrder) => Effect.Effect<TokenOrder.TokenOrder, ParseError>,
  (
    self: TokenOrder.TokenOrder,
    quoteAmount: bigint,
  ) => Effect.Effect<TokenOrder.TokenOrder, ParseError>
>(2, (self, quoteAmount) =>
  pipe(
    quoteAmount,
    Schema.decode(TokenOrder.Input.fields.quoteAmount),
    Effect.map((quoteAmount) =>
      makeProto(
        self.source,
        self.destination,
        self.sender,
        self.receiver,
        self.baseToken,
        self.baseAmount,
        self.quoteToken,
        quoteAmount,
        self.kind,
        self.metadata,
      )
    ),
  ))

/** @internal */
const kindToConst = Match.type<TokenOrder.Kind>().pipe(
  Match.when("initialize", () => 0 as const),
  Match.when("escrow", () => 1 as const),
  Match.when("unescrow", () => 2 as const),
  Match.when("solve", () => 3 as const),
  Match.exhaustive,
)

/** @internal */
export const encodeV1 = (self: TokenOrder.TokenOrder) =>
(meta: {
  name: string
  decimals: number
  symbol: string
  sourceChannelId: number
}): Effect.Effect<Ucs03.TokenOrderV1, ParseError, never> =>
  Effect.gen(function*() {
    const sender = yield* Ucs05.anyDisplayToZkgm(self.sender)
    const receiver = yield* Ucs05.anyDisplayToZkgm(self.receiver)
    return yield* S.decode(Ucs03.TokenOrderV1)({
      _tag: "@unionlabs/sdk/Ucs03/TokenOrder",
      opcode: 3,
      version: 1,
      operand: [
        sender,
        receiver,
        // XXX: remove in favor of pattern matching
        Utils.ensureHex(self.baseToken.address),
        self.baseAmount,
        meta.symbol,
        meta.name,
        meta.decimals, // decimals
        B.match(self.kind == "unescrow", {
          onTrue: () => BigInt(meta.sourceChannelId),
          onFalse: () => 0n,
        }), // path
        Utils.ensureHex(self.quoteToken.address),
        self.quoteAmount,
      ],
    })
  })

/** @internal */
export const encodeV2 = (
  self: TokenOrder.TokenOrder,
): Effect.Effect<Ucs03.TokenOrderV2, ParseError, never> =>
  Effect.gen(function*() {
    const sender = yield* Ucs05.anyDisplayToZkgm(self.sender)
    const receiver = yield* Ucs05.anyDisplayToZkgm(self.receiver)
    return yield* S.decode(Ucs03.TokenOrderV2)({
      _tag: "@unionlabs/sdk/Ucs03/TokenOrder",
      opcode: 3,
      version: 2,
      operand: [
        sender,
        receiver,
        Utils.ensureHex(self.baseToken.address),
        self.baseAmount,
        Utils.ensureHex(self.quoteToken.address),
        self.quoteAmount,
        kindToConst(self.kind),
        self.metadata || "0x",
      ],
    })
  })
