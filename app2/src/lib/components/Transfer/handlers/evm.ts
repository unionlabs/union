import {
  ApprovalReceiptState,
  ApprovalSubmitState as EvmApprovalSubmitState,
  hasFailedExit as hasEvmFailedExit,
  isComplete as isEvmComplete,
  nextState as evmNextState,
  SwitchChainState as EvmSwitchChainState,
  TransferReceiptState as EvmTransferReceiptState,
  TransferSubmission as EvmTransferSubmission,
  TransferSubmitState as EvmTransferSubmitState
} from "$lib/services/transfer-ucs03-evm"
import {TransferState, type TransferStateUnion} from "$lib/components/Transfer/validation.ts"
import type {EVMTransfer} from "@unionlabs/sdk/schema"

export async function handleEvmSubmit(
  currentState: TransferStateUnion,
  typedArgs: EVMTransfer,
  updateState: (state: TransferStateUnion) => void
) {
  let evmState: EvmTransferSubmission
  if (currentState._tag === "Evm") {
    if (hasEvmFailedExit(currentState.state)) {
      switch (currentState.state._tag) {
        case "SwitchChain":
          evmState = EvmTransferSubmission.SwitchChain({
            state: EvmSwitchChainState.InProgress()
          })
          break
        case "ApprovalSubmit":
          evmState = EvmTransferSubmission.ApprovalSubmit({
            state: EvmApprovalSubmitState.InProgress()
          })
          break
        case "ApprovalReceipt":
          evmState = EvmTransferSubmission.ApprovalReceipt({
            state: ApprovalReceiptState.InProgress({ hash: currentState.state.state.hash })
          })
          break
        case "TransferSubmit":
          evmState = EvmTransferSubmission.TransferSubmit({
            state: EvmTransferSubmitState.InProgress()
          })
          break
        case "TransferReceipt":
          evmState = EvmTransferSubmission.TransferReceipt({
            state: EvmTransferReceiptState.InProgress({ hash: currentState.state.state.hash })
          })
          break
        default:
          evmState = EvmTransferSubmission.Filling()
      }
      updateState(TransferState.Evm(evmState))
    } else {
      evmState = currentState.state
    }
  } else {
    evmState = EvmTransferSubmission.Filling()
  }

  const newState = await evmNextState(evmState, typedArgs)
  updateState(newState !== null ? TransferState.Evm(newState) : TransferState.Empty())

  let currentEvmState = newState
  while (
    currentEvmState !== null &&
    !hasEvmFailedExit(currentEvmState) &&
    !isEvmComplete(currentEvmState)
  ) {
    const nextEvmState = await evmNextState(currentEvmState, typedArgs)
    updateState(nextEvmState !== null ? TransferState.Evm(nextEvmState) : TransferState.Empty())
    currentEvmState = nextEvmState
  }
}
