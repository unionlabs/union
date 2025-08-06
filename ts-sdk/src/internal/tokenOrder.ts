import { Effect, Inspectable, Schema } from "effect"
import { dual, pipe } from "effect/Function"
import { ParseError } from "effect/ParseResult"
import { pipeArguments } from "effect/Pipeable"
import { Chain } from "../schema/chain.js"
import * as Token from "../Token.js"
import * as TokenOrder from "../TokenOrder.js"
import type * as Ucs05 from "../Ucs05.js"

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
        .filter(([_k, v]) => v === "undefined")
        .map(([k, _v]) => k),
      source: this.source,
      destination: this.destination,
      sender: this.sender,
      receiver: this.receiver,
      baseToken: this.baseToken,
      baseAmount: this.baseAmount,
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

function makeProto(
  source: Chain,
  destination: Chain,
  sender: string,
  receiver: string,
  baseToken: Token.Any,
  baseAmount: bigint,
  kind: TokenOrder.Kind,
): TokenOrder.TokenOrder {
  const self = Object.create(Proto)
  self.source = source
  self.destination = destination
  self.sender = sender
  self.receiver = receiver
  self.baseToken = baseToken
  self.baseAmount = baseAmount
  self.kind = kind
  return self
}

/** @internal */
export const isTokenOrder = (u: unknown): u is TokenOrder.TokenOrder =>
  typeof u === "object" && u !== null && TypeId in u

/** @internal */
export const empty: TokenOrder.TokenOrder = makeProto(
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  void 0 as unknown as string,
  void 0 as unknown as string,
  void 0 as unknown as Token.Any,
  void 0 as unknown as bigint,
  -1 as unknown as TokenOrder.Kind,
)

/** @internal */
export const make = (
  source: Chain,
  destination: Chain,
  sender: string,
  receiver: string,
  baseToken: Token.Any,
  baseAmount: bigint,
  kind: TokenOrder.Kind,
) =>
  modify(empty, {
    source,
    destination,
    sender,
    receiver,
    baseToken,
    baseAmount,
    kind,
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
      result = yield* setSender(result, options.sender)
    }

    if (options.receiver) {
      result = yield* setReceiver(result, options.receiver)
    }

    if (options.baseToken) {
      result = yield* setBaseToken(result, options.baseToken)
    }

    if (options.baseToken) {
      result = yield* setBaseAmount(result, options.baseAmount)
    }

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
    self.kind,
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
    self.kind,
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
        self.kind,
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
    receiver: string,
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
        self.kind,
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
        self.kind,
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
        self.kind,
      )
    ),
  ))
