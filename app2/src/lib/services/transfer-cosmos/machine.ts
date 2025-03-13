import {
  ApprovalSubmitState,
  SwitchChainState,
  TransferSubmission,
  TransferSubmitState
} from "./state.ts"
import { Effect } from "effect"
import { switchChain } from "./chain.ts"
import { submitTransfer } from "./transactions.ts"
import { approveTransfer } from "./approval.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import type { CosmosWalletId } from "$lib/wallet/cosmos"

export async function nextState(
  ts: TransferSubmission,
  params: ValidTransfer["args"],
  chain: Chain,
  connectedWallet: CosmosWalletId
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Filling: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },

    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(switchChain(chain))
          console.log(exit)
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
          const exit = await Effect.runPromiseExit(approveTransfer(chain, connectedWallet, params))
          console.log(exit)
          if (exit._tag === "Failure") {
            console.log("fail")
            // return TransferSubmission.ApprovalSubmit({
            //   state: ApprovalSubmitState.InProgress()
            // })
          }

          return TransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.Complete({ exit })
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
          return TransferSubmission.Filling() // Or another appropriate state
        }
      })
    }
  })
}
