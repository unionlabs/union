import {
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete,
  nextState as cosmosNextState,
  TransferSubmission as CosmosTransferSubmission,
  SwitchChainState,
  ApprovalSubmitState,
  TransferSubmitState
} from "$lib/services/transfer-ucs03-cosmos"
import { TransferState, type TransferStateUnion } from "$lib/components/Transfer/validation.ts"
import type { CosmosTransfer } from "$lib/schema/transfer-args.ts"

export async function handleCosmosSubmit(
  currentState: TransferStateUnion,
  typedArgs: CosmosTransfer,
  connectedWallet: "leap" | "keplr"
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
    } else {
      cosmosState = currentState.state
    }
  } else {
    cosmosState = CosmosTransferSubmission.Filling()
  }

  const newState = await cosmosNextState(cosmosState, typedArgs, connectedWallet)

  let result = newState !== null ? TransferState.Cosmos(newState) : TransferState.Empty()

  let currentCosmosState = newState
  while (currentCosmosState !== null && !hasCosmosFailedExit(currentCosmosState)) {
    const nextCosmosState = await cosmosNextState(currentCosmosState, typedArgs, connectedWallet)

    result =
      nextCosmosState !== null ? TransferState.Cosmos(nextCosmosState) : TransferState.Empty()

    currentCosmosState = nextCosmosState
    if (currentCosmosState !== null && isCosmosComplete(currentCosmosState)) break
  }

  return result
}
