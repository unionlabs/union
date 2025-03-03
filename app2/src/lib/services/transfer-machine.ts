import { submitTransfer, switchChain, waitForReceipt } from "$lib/services/transfer"
import {
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "$lib/services/transfer-state"
import { Effect } from "effect"
import { sepolia } from "viem/chains"

export async function nextState(ts: TransferSubmission): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Pending: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },
    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          // TODO: don't hardcode
          const exit = await Effect.runPromiseExit(switchChain(sepolia.id))
          return TransferSubmission.SwitchChain({ state: SwitchChainState.Complete({ exit }) })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
          }
          return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
        }
      })
    },
    TransferSubmit: ({ state }) => {
      return TransferSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(
            // TODO: don't hardcode
            submitTransfer({
              chain: sepolia,
              account: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
              value: 1n,
              to: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
            })
          )
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
          }
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.InProgress({ hash: exit.value })
          })
        }
      })
    },
    TransferReceipt: ({ state }) => {
      return TransferReceiptState.$match(state, {
        InProgress: async ({ hash }) => {
          const exit = await Effect.runPromiseExit(waitForReceipt(hash))
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          //TODO: there is no real next state here
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit })
          })
        }
      })
    }
  })
}
