import { Inspectable } from "effect"
import { dual } from "effect/Function"
import { pipeArguments } from "effect/Pipeable"
import type * as ClientRequest from "../ClientRequest.js"
import { Chain } from "../schema/chain.js"

/** @internal */
export const TypeId: ClientRequest.TypeId = Symbol.for(
  "@effect/platform/HttpClientRequest",
) as ClientRequest.TypeId

const Proto = {
  [TypeId]: TypeId,
  ...Inspectable.BaseProto,
  toJSON(this: ClientRequest.ClientRequest): unknown {
    return {
      _id: "@unionlabs/sdk/ClientRequest",
      source: this.source,
      destination: this.destination,
      sender: this.sender,
      receiver: this.receiver,
      amount: this.amount,
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

function makeProto(
  method: ClientRequest.Method,
  source: Chain,
  destination: Chain,
  sender: string,
  receiver: string,
  amount: bigint,
): ClientRequest.ClientRequest {
  const self = Object.create(Proto)
  self.method = method
  self.source = source
  self.destination = destination
  self.sender = sender
  self.receiver = receiver
  self.amount = amount
  return self
}

/** @internal */
export const isClientRequest = (u: unknown): u is ClientRequest.ClientRequest =>
  typeof u === "object" && u !== null && TypeId in u

/** @internal */
export const empty: ClientRequest.ClientRequest = makeProto(
  "SEND",
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  "",
  "",
  0n,
)

/** @internal */
export const make = <M extends ClientRequest.Method>(method: M) =>
(
  sender: string,
  receiver: string,
  options?: M extends "SEND" ? ClientRequest.Options.Send : ClientRequest.Options.NoUrl,
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
    options: ClientRequest.Options,
  ) => (self: ClientRequest.ClientRequest) => ClientRequest.ClientRequest,
  (self: ClientRequest.ClientRequest, options: ClientRequest.Options) => ClientRequest.ClientRequest
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
  ) => (self: ClientRequest.ClientRequest) => ClientRequest.ClientRequest,
  (self: ClientRequest.ClientRequest, sender: string) => ClientRequest.ClientRequest
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
  ) => (self: ClientRequest.ClientRequest) => ClientRequest.ClientRequest,
  (self: ClientRequest.ClientRequest, receiver: string) => ClientRequest.ClientRequest
>(2, (self, receiver) =>
  makeProto(
    self.method,
    self.sender,
    receiver,
  ))
