import { Data, Effect, Stream } from "effect"

/**
 * NOTE: don't assume exhaustion
 * - consider documenting per ecosystem states ? 
 */
type LifecycleEvent = Data.TaggedEnum<{
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
  Broadcast: {}
  Receipt: {}
  Indexed: {}
  Finalized: {}
}>

export const LifecycleEvent = Data.taggedEnum<LifecycleEvent>()

export interface TxIncomingMessage<E> {
  /** lifecycle and chain events in temporal order */
  readonly stream: Stream.Stream<LifecycleEvent, E>
  /**
   * - add default ucompletion handler (index)
   * - allow pred fn
   */
  readonly waitFor: (pred: (a: LifecycleEvent['_tag']) => Effect.Effect<{ readonly txHash: string }, E>

}
