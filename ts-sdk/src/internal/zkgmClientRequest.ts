import { Inspectable } from "effect"
import { dual } from "effect/Function"
import { pipeArguments } from "effect/Pipeable"
import { Chain } from "../schema/chain.js"
import type * as ClientRequest from "../ZkgmClientRequest.js"
import { ZkgmInstruction } from "../ZkgmInstruction.js"

/** @internal */
export const TypeId: ClientRequest.TypeId = Symbol.for(
  "@effect/platform/ZkgmClientRequest",
) as ClientRequest.TypeId

const Proto = {
  [TypeId]: TypeId,
  ...Inspectable.BaseProto,
  toJSON(this: ClientRequest.ZkgmClientRequest): unknown {
    return {
      _id: "@unionlabs/sdk/ClientRequest",
      source: this.source,
      destination: this.destination,
      instruction: this.instruction,
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

function makeProto(
  source: Chain,
  destination: Chain,
  instruction: ZkgmInstruction,
): ClientRequest.ZkgmClientRequest {
  const self = Object.create(Proto)
  self.source = source
  self.destination = destination
  self.instruction = instruction
  return self
}

/** @internal */
export const isZkgmClientRequest = (u: unknown): u is ClientRequest.ZkgmClientRequest =>
  typeof u === "object" && u !== null && TypeId in u

/** @internal */
export const empty: ClientRequest.ZkgmClientRequest = makeProto(
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  void 0 as unknown as ZkgmInstruction,
)

/** @internal */
export const make = (
  source: Chain,
  destination: Chain,
  instruction: ZkgmInstruction,
) =>
  modify(empty, {
    source,
    destination,
    instruction,
  })

/** @internal */
export const modify = dual<
  (
    options: ClientRequest.Options,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (
    self: ClientRequest.ZkgmClientRequest,
    options: ClientRequest.Options,
  ) => ClientRequest.ZkgmClientRequest
>(2, (self, options) => {
  let result = self

  if (options.source) {
    result = setSource(result, options.source)
  }
  if (options.destination) {
    result = setDestination(result, options.destination)
  }

  return result
})

/** @internal */
export const setSource = dual<
  (
    source: Chain,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (self: ClientRequest.ZkgmClientRequest, source: Chain) => ClientRequest.ZkgmClientRequest
>(2, (self, source) =>
  makeProto(
    source,
    self.destination,
    self.instruction,
  ))

/** @internal */
export const setDestination = dual<
  (
    destination: Chain,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (self: ClientRequest.ZkgmClientRequest, destination: Chain) => ClientRequest.ZkgmClientRequest
>(2, (self, destination) =>
  makeProto(
    self.source,
    destination,
    self.instruction,
  ))
