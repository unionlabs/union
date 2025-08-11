import { Inspectable } from "effect"
import * as A from "effect/Array"
import { pipeArguments } from "effect/Pipeable"
import * as Batch from "../Batch.js"
import { ZkgmInstruction } from "../ZkgmInstruction.js"

/** @internal */
export const TypeId: Batch.TypeId = Symbol.for(
  "@unionlabs/sdk/Batch",
) as Batch.TypeId

const Proto = {
  [TypeId]: TypeId,
  _tag: "Batch",
  ...Inspectable.BaseProto,
  toJSON(this: Batch.Batch): unknown {
    return {
      _id: "@unionlabs/sdk/Batch",
      instructions: A.map(this.instructions, (x) => x.toJSON()),
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

export const make = <
  A extends ZkgmInstruction,
>(iterable: Iterable<A>): Batch.Batch =>
  Object.assign(Object.create(Proto), {
    _tag: "Batch",
    instructions: iterable,
  })

/** @internal */
export const fromIterable = <A extends ZkgmInstruction>(iterable: Iterable<A>): Batch.Batch =>
  make(iterable)
