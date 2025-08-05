import { Effect, Inspectable, Schema } from "effect"
import { dual } from "effect/Function"
import { ParseError } from "effect/ParseResult"
import { pipeArguments } from "effect/Pipeable"
import { Chain } from "../schema/chain.js"
import * as Token from "../Token.js"
import * as TokenOrder from "../TokenOrder.js"

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
        .filter(([k, v]) => v === "undefined")
        .map(([k, v]) => k),
      source: this.source,
      destination: this.destination,
      sender: this.sender,
      receiver: this.receiver,
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
  baseAmount: bigint,
  baseToken: Token.Any,
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
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
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
      const sender = yield* Schema.decodeUnknown(TokenOrder.Input.fields.sender)(options.sender)
      result = setSender(result, sender)
    }

    if (options.receiver) {
      result = setReceiver(result, options.receiver)
    }

    if (options.baseToken) {
      const baseToken = yield* Schema.decodeUnknown(TokenOrder.Input.fields.baseToken)(
        options.baseToken,
      )
      result = setBaseToken(result, baseToken)
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
    sender: string,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, sender: string) => TokenOrder.TokenOrder
>(2, (self, sender) =>
  makeProto(
    self.source,
    self.destination,
    sender,
    self.receiver,
    self.baseToken,
    self.baseAmount,
    self.kind,
  ))

/** @internal */
export const setReceiver = dual<
  (
    receiver: string,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, receiver: string) => TokenOrder.TokenOrder
>(2, (self, receiver) =>
  makeProto(
    self.source,
    self.destination,
    self.sender,
    receiver,
    self.baseToken,
    self.baseAmount,
    self.kind,
  ))

/** @internal */
export const setBaseToken = dual<
  (
    baseToken: Token.Any | string,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, baseToken: Token.Any | string) => TokenOrder.TokenOrder
>(2, (self, baseToken) =>
  makeProto(
    self.source,
    self.destination,
    self.sender,
    self.receiver,
    baseToken,
    self.baseAmount,
    self.kind,
  ))
