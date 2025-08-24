import { runPromiseExit } from "$lib/runtime"

import type { ValidTransfer } from "@unionlabs/sdk/schema"
import { approveTransfer } from "./approval"
import { switchChain } from "./chain"
import {
  ApprovalSubmitState,
  SwitchChainState,
  TransferSubmission,
  TransferSubmitState,
} from "./state"
import { submitTransfer } from "./transactions"

export async function nextState(
  ts: TransferSubmission,
  params: ValidTransfer["args"],
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Filling: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },

    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await runPromiseExit(switchChain(params.sourceChain))
          return TransferSubmission.SwitchChain({
            state: SwitchChainState.Complete({ exit }),
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            // Stay in SwitchChain with the error in the Complete state
            return TransferSubmission.SwitchChain({
              state: SwitchChainState.Complete({ exit }),
            })
          }
          return TransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.InProgress(),
          })
        },
      })
    },

    ApprovalSubmit: ({ state }) => {
      return ApprovalSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await runPromiseExit(
            approveTransfer(params.sourceChain, params),
          )
          return TransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.Complete({ exit }),
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            // Stay in ApprovalSubmit with the error in the Complete state
            return TransferSubmission.ApprovalSubmit({
              state: ApprovalSubmitState.Complete({ exit }),
            })
          }
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.InProgress(),
          })
        },
      })
    },

    TransferSubmit: ({ state }) => {
      return TransferSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await runPromiseExit(submitTransfer(params))
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.Complete({ exit }),
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            // Stay in TransferSubmit with the error in the Complete state
            return TransferSubmission.TransferSubmit({
              state: TransferSubmitState.Complete({ exit }),
            })
          }
          return TransferSubmission.Filling()
        },
      })
    },
  })
}
