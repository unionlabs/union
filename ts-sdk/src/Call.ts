/**
 * This module provides a high-level API for UCS03 `Call` instruction construction.
 *
 * @since 2.0.0
 */
import { Effect, Inspectable } from "effect"
import { Pipeable, pipeArguments } from "effect/Pipeable"
import * as S from "effect/Schema"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/call.js"
import { Hex } from "./schema/hex.js"
import * as Ucs03 from "./Ucs03.js"
import * as Ucs05 from "./Ucs05.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = internal.TypeId

/**
 * @category type ids
 * @since 2.0.0
 */
export type TypeId = typeof TypeId

/**
 * @category models
 * @since 2.0.0
 */
export interface Call
  extends Inspectable.Inspectable, Pipeable, Iterable<ZkgmInstruction.ZkgmInstruction>
{
  readonly [TypeId]: TypeId
  readonly _tag: "Call"
  readonly sender: Ucs05.AnyDisplay
  readonly eureka: boolean
  readonly contractAddress: Ucs05.AnyDisplay
  readonly contractCalldata: Hex
  readonly opcode: 1
  readonly version: 0
}

const CallProto: Omit<
  Call,
  "sender" | "eureka" | "contractAddress" | "contractCalldata" | "opcode" | "version" | "_tag"
> = {
  [TypeId]: TypeId,
  ...Inspectable.BaseProto,
  *[Symbol.iterator](): IterableIterator<Call> {
    yield this as Call
  },
  toJSON(this: Call): unknown {
    return {
      _id: "@unionlabs/sdk/Call",
      sender: this.sender,
      eureka: this.eureka,
      contractAddress: this.contractAddress,
      contractCalldata: this.contractCalldata,
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
} as const

/**
 * @category utils
 * @since 2.0.0
 */
export const make = (options: {
  sender: Ucs05.AnyDisplay
  eureka: boolean
  contractAddress: Ucs05.AnyDisplay
  contractCalldata: Hex
}): Call => {
  const self = Object.create(CallProto)

  self.sender = options.sender
  self.eureka = options.eureka
  self.contractAddress = options.contractAddress
  self.contractCalldata = options.contractCalldata
  self.version = 0
  self.opcode = 1
  self._tag = "Call"

  return self
}

/**
 * @category utils
 * @since 2.0.0
 */
export const encode = (self: Call) =>
  Effect.gen(function*() {
    const sender = yield* Ucs05.anyDisplayToZkgm(self.sender)
    const contractAddress = yield* Ucs05.anyDisplayToZkgm(self.contractAddress)
    return yield* S.decode(Ucs03.Call)({
      _tag: "@unionlabs/sdk/Ucs03/Call",
      opcode: 1,
      version: 0,
      operand: [
        sender,
        self.eureka,
        contractAddress,
        self.contractCalldata,
      ],
    })
  })
