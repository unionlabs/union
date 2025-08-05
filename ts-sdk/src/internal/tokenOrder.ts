import { Inspectable } from "effect"
import { dual } from "effect/Function"
import { pipeArguments } from "effect/Pipeable"
import { Chain } from "../schema/chain.js"
import * as Token from "../Token.js"
import type * as TokenOrder from "../TokenOrder.js"

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
  baseAmount: bigint,
  baseToken: Token.Any,
): TokenOrder.TokenOrder {
  const self = Object.create(Proto)
  self.source = source
  self.destination = destination
  self.sender = sender
  self.receiver = receiver
  self.baseAmount = baseAmount
  return self
}

/** @internal */
export const isTokenOrder = (u: unknown): u is TokenOrder.TokenOrder =>
  typeof u === "object" && u !== null && TypeId in u

/** @internal */
export const empty: TokenOrder.TokenOrder = makeProto(
  "SEND",
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  void 0 as unknown as,
  "",
  0n,
)

/** @internal */
export const make = <M extends TokenOrder.Method>(method: M) =>
(
  sender: string,
  receiver: string,
  options?: M extends "SEND" ? TokenOrder.Options.Send : TokenOrder.Options.NoUrl,
) =>
  modify(empty, {
    method,
    sender,
    receiver,
    ...(options ?? undefined),
  })

/** @internal */
export const send = make("SEND")

/** @internal */
export const modify = dual<
  (
    options: TokenOrder.Options,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, options: TokenOrder.Options) => TokenOrder.TokenOrder
>(2, (self, options) => {
  let result = self

  if (options.sender) {
    result = setSender(result, options.sender)
  }
  if (options.receiver) {
    result = setReceiver(result, options.receiver)
  }

  return result
})

/** @internal */
export const setSender = dual<
  (
    sender: string,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, sender: string) => TokenOrder.TokenOrder
>(2, (self, sender) =>
  makeProto(
    self.method,
    sender,
    self.receiver,
  ))

/** @internal */
export const setReceiver = dual<
  (
    receiver: string,
  ) => (self: TokenOrder.TokenOrder) => TokenOrder.TokenOrder,
  (self: TokenOrder.TokenOrder, receiver: string) => TokenOrder.TokenOrder
>(2, (self, receiver) =>
  makeProto(
    self.method,
    self.sender,
    receiver,
  ))
