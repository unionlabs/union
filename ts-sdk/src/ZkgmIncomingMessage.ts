/**
 * This module describes a superset of events during finite-state machine execution following {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
import { Data, Effect, Inspectable, Stream } from "effect"
import { constFalse, constTrue } from "effect/Function"

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
  // | { _tag: "SwitchChainStart" ; target: UniversalChainId }
  // | { _tag: "SwitchChainDone"  ; success: boolean }
  // | { _tag: "WriteTxStart"     ; payload: { to: string; data: string } }
  // | { _tag: "WriteTxDone"      ; txHash: string }
  // | { _tag: "Broadcasted"      ; txHash: string }
  // | { _tag: "Log"              ; chainFamily: "ethereum"; log: EvmLog }
  // | { _tag: "Receipt"          ; chainFamily: "cosmos"  ; receipt: CosmosTx }
  // | { _tag: "Confirmed"        ; block: bigint }
  // | { _tag: "Finalised"        ; height: bigint ; success: boolean }
  // | { _tag: "Failed"           ; reason: string }
  // evm
  EvmWriteContractInProgress: {}
  EvmWriteContractComplete: {}
  EvmWaitForSafeWalletHash: {}
  EvmTransactionReceiptInProgress: {}
  EvmTransactionReceiptComplete: {}

  // cosmos
  CosmosWriteContractInProgress: {}
  CosmosWriteContractComplete: {}
  // agnostic
  // Broadcast: {}
  // Receipt: {}
  // Indexed: {}
  // Finalized: {}
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
  readonly waitFor: (
    pred: (a: LifecycleEvent) => boolean,
  ) => Effect.Effect<{ readonly txHash: string }, E>
}

/**
 * @category utils
 * @since 2.0.0
 */
export const isBroadcast = LifecycleEvent.$match({
  CosmosWriteContractComplete: constFalse,
  CosmosWriteContractInProgress: constTrue,
  EvmTransactionReceiptComplete: constFalse,
  EvmTransactionReceiptInProgress: constFalse,
  EvmWaitForSafeWalletHash: constFalse,
  EvmWriteContractComplete: constTrue,
  EvmWriteContractInProgress: constFalse,
})

/**
 * @category utils
 * @since 2.0.0
 */
export const isComplete = LifecycleEvent.$match({
  CosmosWriteContractComplete: constTrue,
  CosmosWriteContractInProgress: constFalse,
  EvmTransactionReceiptComplete: constTrue,
  EvmTransactionReceiptInProgress: constFalse,
  EvmWaitForSafeWalletHash: constFalse,
  EvmWriteContractComplete: constFalse,
  EvmWriteContractInProgress: constFalse,
})

/**
 * @category utils
 * @since 2.0.0
 */
export const inspect = <E>(self: ZkgmIncomingMessage<E>, that: object): object => {
  // TODO: fulfill
  return Object.create({})
}
