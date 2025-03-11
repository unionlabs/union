import {
  ApprovalReceiptState,
  ApprovalSubmitState,
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "./state.ts"
import { Effect } from "effect"
import { switchChain } from "./chain.ts"
import { submitTransfer, waitForTransferReceipt } from "./transactions.ts"
import { approveTransfer, waitForApprovalReceipt } from "$lib/services/transfer-ucs03-evm/approval"
import type { Chain } from "$lib/schema/chain.ts"
import type {ValidTransferType} from "$lib/schema/transfer-args.ts";

export async function nextState(
  ts: TransferSubmission,
  params: ValidTransferType["args"],
  chain: Chain
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Filling: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },

    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          //@ts-ignore
          const exit = await Effect.runPromiseExit(switchChain(params.sourceChain.id))
          return TransferSubmission.SwitchChain({ state: SwitchChainState.Complete({ exit }) })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
          }
          return TransferSubmission.ApprovalSubmit({ state: ApprovalSubmitState.InProgress() })
        }
      })
    },

    ApprovalSubmit: ({ state }) => {
      return ApprovalSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(approveTransfer(chain, params))
          if (exit._tag === "Failure") {
            return TransferSubmission.ApprovalSubmit({
              state: ApprovalSubmitState.Complete({ exit })
            })
          }

          return TransferSubmission.ApprovalReceipt({
            state: ApprovalReceiptState.InProgress({ hash: exit.value })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.ApprovalSubmit({ state: ApprovalSubmitState.InProgress() })
          }
          return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
        }
      })
    },

    ApprovalReceipt: ({ state }) => {
      return ApprovalReceiptState.$match(state, {
        InProgress: async ({ hash }) => {
          const exit = await Effect.runPromiseExit(waitForApprovalReceipt(chain, hash))
          return TransferSubmission.ApprovalReceipt({
            state: ApprovalReceiptState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.ApprovalSubmit({ state: ApprovalSubmitState.InProgress() })
          }
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.InProgress()
          })
        }
      })
    },

    TransferSubmit: ({ state }) => {
      return TransferSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(submitTransfer(chain, params))
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
          const exit = await Effect.runPromiseExit(waitForTransferReceipt(chain, hash))
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
