import { submitTransfer, switchChain, waitForReceipt } from "."
import {
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "./state"
import { Effect } from "effect"
import type { Chain, Address } from "viem"

export type TransactionParams = {
  chain: Chain
  account: Address
  value: bigint
  to: Address
}

export async function nextState(
  ts: TransferSubmission,
  params: TransactionParams
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Pending: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },
    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(switchChain(params.chain.id))
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
            submitTransfer(params)
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
