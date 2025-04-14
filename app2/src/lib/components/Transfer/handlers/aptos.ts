import {
  hasFailedExit as hasAptosFailedExit,
  isComplete as isAptosComplete,
  nextState as aptosNextState,
  TransferReceiptState as AptosTransferReceiptState,
  TransferSubmission as AptosTransferSubmission,
  TransferSubmitState as AptosTransferSubmitState
} from "$lib/services/transfer-ucs03-aptos"
import {SwitchChainState as EvmSwitchChainState} from "$lib/services/transfer-ucs03-evm"
import {TransferState, type TransferStateUnion} from "$lib/components/Transfer/validation.ts"
import type {AptosTransfer, Chain} from "@unionlabs/sdk/schema"

export async function handleAptosSubmit(
  currentState: TransferStateUnion,
  typedArgs: AptosTransfer,
  sourceChainValue: Chain,
  updateState: (state: TransferStateUnion) => void
) {
  let aptosState: AptosTransferSubmission

  if (currentState._tag === "Aptos") {
    if (hasAptosFailedExit(currentState.state)) {
      switch (currentState.state._tag) {
        case "SwitchChain":
          aptosState = AptosTransferSubmission.SwitchChain({
            state: EvmSwitchChainState.InProgress()
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
      updateState(TransferState.Aptos(aptosState))
    } else {
      aptosState = currentState.state
    }
  } else {
    aptosState = AptosTransferSubmission.Filling()
  }

  const newState = await aptosNextState(aptosState, typedArgs, sourceChainValue)
  updateState(newState !== null ? TransferState.Aptos(newState) : TransferState.Empty())

  let currentAptosState = newState
  while (
    currentAptosState !== null &&
    !hasAptosFailedExit(currentAptosState) &&
    !isAptosComplete(currentAptosState)
  ) {
    const nextAptosState = await aptosNextState(currentAptosState, typedArgs, sourceChainValue)
    updateState(
      nextAptosState !== null ? TransferState.Aptos(nextAptosState) : TransferState.Empty()
    )
    currentAptosState = nextAptosState
  }
}
