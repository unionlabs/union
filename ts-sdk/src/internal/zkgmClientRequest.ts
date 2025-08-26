import { HashMap, Inspectable, Match } from "effect"
import * as A from "effect/Array"
import { dual, pipe } from "effect/Function"
import * as O from "effect/Option"
import { pipeArguments } from "effect/Pipeable"
import { Chain } from "../schema/chain.js"
import { ChannelId } from "../schema/channel.js"
import { Hex } from "../schema/hex.js"
import type * as Token from "../Token.js"
import type * as ClientRequest from "../ZkgmClientRequest.js"
import { ZkgmInstruction } from "../ZkgmInstruction.js"

/** @internal */
export const TypeId: ClientRequest.TypeId = Symbol.for(
  "@unionlabs/sdk/ZkgmClientRequest",
) as ClientRequest.TypeId

const Proto = {
  [TypeId]: TypeId,
  ...Inspectable.BaseProto,
  toJSON(this: ClientRequest.ZkgmClientRequest): unknown {
    return {
      _id: "@unionlabs/sdk/ZkgmClientRequest",
      source: this.source,
      destination: this.destination,
      channelId: this.channelId,
      ucs03Address: this.ucs03Address,
      instruction: this.instruction.toJSON(),
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

function makeProto(
  source: Chain,
  destination: Chain,
  channelId: ChannelId,
  ucs03Address: string,
  instruction: ZkgmInstruction,
): ClientRequest.ZkgmClientRequest {
  const self = Object.create(Proto)
  self.source = source
  self.destination = destination
  self.channelId = channelId
  self.ucs03Address = ucs03Address
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
  void 0 as unknown as ChannelId,
  void 0 as unknown as Hex,
  void 0 as unknown as ZkgmInstruction,
)

/** @internal */
export const make = (options: {
  source: Chain
  destination: Chain
  channelId: ChannelId
  ucs03Address: string
  instruction: ZkgmInstruction
}) => modify(empty, options)

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
  if (options.channelId) {
    result = setChannelId(result, options.channelId)
  }
  if (options.ucs03Address) {
    result = setUcs03Address(result, options.ucs03Address)
  }
  if (options.instruction) {
    result = setInstruction(result, options.instruction)
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
    self.channelId,
    self.ucs03Address,
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
    self.channelId,
    self.ucs03Address,
    self.instruction,
  ))

/** @internal */
export const setChannelId = dual<
  (
    channelId: ChannelId,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (self: ClientRequest.ZkgmClientRequest, channelId: ChannelId) => ClientRequest.ZkgmClientRequest
>(2, (self, channelId) =>
  makeProto(
    self.source,
    self.destination,
    channelId,
    self.ucs03Address,
    self.instruction,
  ))
/** @internal */
export const setUcs03Address = dual<
  (
    ucs03Address: string,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (self: ClientRequest.ZkgmClientRequest, ucs03Address: string) => ClientRequest.ZkgmClientRequest
>(2, (self, ucs03Address) =>
  makeProto(
    self.source,
    self.destination,
    self.channelId,
    ucs03Address,
    self.instruction,
  ))

/** @internal */
export const setInstruction = dual<
  (
    instruction: ZkgmInstruction,
  ) => (self: ClientRequest.ZkgmClientRequest) => ClientRequest.ZkgmClientRequest,
  (
    self: ClientRequest.ZkgmClientRequest,
    instruction: ZkgmInstruction,
  ) => ClientRequest.ZkgmClientRequest
>(2, (self, instruction) =>
  makeProto(
    self.source,
    self.destination,
    self.channelId,
    self.ucs03Address,
    instruction,
  ))

/** @internal */
export const requiredFunds = (
  self: ClientRequest.ZkgmClientRequest,
): O.Option<A.NonEmptyReadonlyArray<readonly [Token.Any, bigint]>> => {
  const reduceToFunds = (
    instr: ZkgmInstruction,
  ): ReadonlyArray<O.Option<readonly [Token.Any, bigint]>> =>
    Match.value(instr).pipe(
      Match.tagsExhaustive({
        Batch: (batch) =>
          pipe(
            A.fromIterable(batch),
            A.reduce([] as ReadonlyArray<O.Option<readonly [Token.Any, bigint]>>, (acc, child) => [
              ...acc,
              ...reduceToFunds(child),
            ]),
          ),

        TokenOrder: (x) => [O.some([x.baseToken, x.baseAmount] as const)],
      }),
    )

  return pipe(
    reduceToFunds(self.instruction),
    A.getSomes,
    A.reduce(
      HashMap.empty<Token.Any, bigint>(),
      (acc, [k, v]) => {
        const curr = pipe(HashMap.get(acc, k), O.getOrElse(() => 0n))
        return HashMap.set(acc, k, curr + v)
      },
    ),
    HashMap.toEntries,
    O.liftPredicate(A.isNonEmptyReadonlyArray<readonly [Token.Any, bigint]>),
  )
}
