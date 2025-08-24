/**
 * This module describes a superset of events during finite-state machine execution following {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
import { Brand, Data, Effect, Inspectable, Option, Predicate, Stream } from "effect"
import { constTrue } from "effect/Function"
import { Hex } from "./schema/hex.js"
import { PacketHash } from "./schema/packet.js"

/**
 * @since 2.0.0
 * @category type ids
 */
export const TypeId: unique symbol = Symbol.for("@unionlabs/sdk/ZkgmIncomingMessage")

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @category models
 * @since 2.0.0
 */
export type LifecycleEvent = Data.TaggedEnum<{
  // EVM
  EvmTransactionReceiptComplete: {
    transactionHash: Hex & Brand.Brand<"Hash">
    blockHash: Hex & Brand.Brand<"Hash">
    gasUsed: bigint
  }
  WaitForSafeWalletHash: {
    hash: Hex & Brand.Brand<"Hash">
  }
  // Cosmos
  // Agnostic
  Indexed: {
    packetHash: PacketHash
  }
}>

/**
 * @category utils
 * @since 2.0.0
 */
export const LifecycleEvent = Data.taggedEnum<LifecycleEvent>()

/**
 * @category models
 * @since 2.0.0
 */
export interface ZkgmIncomingMessage<E> extends Inspectable.Inspectable {
  [TypeId]: TypeId
  /** lifecycle and chain events in temporal order */
  readonly stream: Stream.Stream<LifecycleEvent, E>
  /**
   * - add default ucompletion handler (index)
   * - allow pred fn
   */
  readonly waitFor: <A extends LifecycleEvent>(
    refinement: Predicate.Refinement<NoInfer<LifecycleEvent>, A>,
  ) => Effect.Effect<Option.Option<A>, E>
}

/**
 * @category utils
 * @since 2.0.0
 */
export const isComplete = LifecycleEvent.$match({
  EvmTransactionReceiptComplete: constTrue,
  WaitForSafeWalletHash: constTrue,
  Indexed: constTrue,
})

/**
 * @category utils
 * @since 2.0.0
 */
export const inspect = <E>(self: ZkgmIncomingMessage<E>, that: object): object => {
  // TODO: fulfill
  return Object.create({})
}
