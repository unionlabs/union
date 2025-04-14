import {
  ApprovalSubmitState,
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete,
  nextState as cosmosNextState,
  SwitchChainState,
  TransferSubmission as CosmosTransferSubmission,
  TransferSubmitState
} from "$lib/services/transfer-ucs03-cosmos"
import {TransferState, type TransferStateUnion} from "$lib/components/Transfer/validation.ts"
import type {CosmosTransfer} from "@unionlabs/sdk/schema"

export async function handleCosmosSubmit(
  currentState: TransferStateUnion,
  typedArgs: CosmosTransfer,
  connectedWallet: "leap" | "keplr",
  updateState: (state: TransferStateUnion) => void
) {
  let cosmosState: CosmosTransferSubmission

  if (currentState._tag === "Cosmos") {
    if (hasCosmosFailedExit(currentState.state)) {
      switch (currentState.state._tag) {
        case "SwitchChain":
          cosmosState = CosmosTransferSubmission.SwitchChain({
            state: SwitchChainState.InProgress()
          })
          break
        case "ApprovalSubmit":
          cosmosState = CosmosTransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.InProgress()
          })
          break
        case "TransferSubmit":
          cosmosState = CosmosTransferSubmission.TransferSubmit({
            state: TransferSubmitState.InProgress()
          })
          break
        default:
          cosmosState = CosmosTransferSubmission.Filling()
      }
      updateState(TransferState.Cosmos(cosmosState))
    } else {
      cosmosState = currentState.state
    }
  } else {
    cosmosState = CosmosTransferSubmission.Filling()
  }

  const newState = await cosmosNextState(cosmosState, typedArgs, connectedWallet)

  updateState(newState !== null ? TransferState.Cosmos(newState) : TransferState.Empty())

  let currentCosmosState = newState
  while (
    currentCosmosState !== null &&
    !hasCosmosFailedExit(currentCosmosState) &&
    !isCosmosComplete(currentCosmosState)
  ) {
    const nextCosmosState = await cosmosNextState(currentCosmosState, typedArgs, connectedWallet)

    updateState(
      nextCosmosState !== null ? TransferState.Cosmos(nextCosmosState) : TransferState.Empty()
    )
    currentCosmosState = nextCosmosState
  }
}
