import {
  hasFailedExit as hasAptosFailedExit,
  isComplete as isAptosComplete,
  nextState as aptosNextState,
  TransferSubmitState as AptosTransferSubmitState,
  TransferSubmission as AptosTransferSubmission,
  TransferReceiptState as AptosTransferReceiptState
} from "$lib/services/transfer-ucs03-aptos"
import { TransferState, type TransferStateUnion } from "$lib/components/Transfer/validation.ts"
import type { AptosTransfer } from "$lib/schema/transfer-args.ts"
import type { Chain } from "$lib/schema/chain.ts"

export async function handleAptosSubmit(
  currentState: TransferStateUnion,
  typedArgs: AptosTransfer,
  sourceChainValue: Chain
) {
  let aptosState: AptosTransferSubmission

  if (currentState._tag === "Aptos") {
    if (hasAptosFailedExit(currentState.state)) {
      switch (currentState.state._tag) {
        case "SwitchChain":
          aptosState = AptosTransferSubmission.SwitchChain({
            state: AptosTransferSubmitState.InProgress()
          })
          break
        case "TransferSubmit":
          aptosState = AptosTransferSubmission.TransferSubmit({
            state: AptosTransferSubmitState.InProgress()
          })
          break
        case "TransferReceipt":
          aptosState = AptosTransferSubmission.TransferReceipt({
            state: AptosTransferReceiptState.InProgress({ hash: currentState.state.state.hash })
          })
          break
        default:
          aptosState = AptosTransferSubmission.Filling()
      }
    } else {
      aptosState = currentState.state
    }
  } else {
    aptosState = AptosTransferSubmission.Filling()
  }

  const newState = await aptosNextState(aptosState, typedArgs, sourceChainValue)
  let result = newState !== null ? TransferState.Aptos(newState) : TransferState.Empty()

  let currentAptosState = newState
  while (currentAptosState !== null && !hasAptosFailedExit(currentAptosState)) {
    const nextAptosState = await aptosNextState(currentAptosState, typedArgs, sourceChainValue)
    result = nextAptosState !== null ? TransferState.Aptos(nextAptosState) : TransferState.Empty()

    currentAptosState = nextAptosState
    if (currentAptosState !== null && isAptosComplete(currentAptosState)) break
  }

  return result
}
