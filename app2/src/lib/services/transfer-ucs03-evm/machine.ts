import {
  ApprovalReceiptState,
  ApprovalSubmitState,
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "./state.ts"
import { Effect, Option } from "effect"
import { switchChain } from "./chain.ts"
import { submitTransfer, waitForTransferReceipt } from "./transactions.ts"
import { approveTransfer, waitForApprovalReceipt } from "$lib/services/transfer-ucs03-evm/approval"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import { SwitchChainError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import type { SwitchChainErrorType } from "viem"

export async function nextState(
  ts: TransferSubmission,
  params: ValidTransfer["args"]
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Filling: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },

    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const viemChainOption = params.sourceChain.toViemChain()
          const exit = await Effect.runPromiseExit(
            Option.match(viemChainOption, {
              onNone: () =>
                Effect.fail(
                  new SwitchChainError({
                    cause: {
                      name: "UserRejectedRequestError",
                      message: "Could not convert to viem chain for chain switch."
                    } as SwitchChainErrorType
                  })
                ),
              onSome: switchChain
            })
          )
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
          const exit = await Effect.runPromiseExit(approveTransfer(params))
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
          const exit = await Effect.runPromiseExit(waitForApprovalReceipt(params.sourceChain, hash))
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
          const exit = await Effect.runPromiseExit(submitTransfer(params.sourceChain, params))
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
          const exit = await Effect.runPromiseExit(waitForTransferReceipt(params.sourceChain, hash))
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
