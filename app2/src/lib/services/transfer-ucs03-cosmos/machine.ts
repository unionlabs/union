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
          return TransferSubmission.SwitchChain({
            state: SwitchChainState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            // Stay in SwitchChain with the error in the Complete state
            return TransferSubmission.SwitchChain({
              state: SwitchChainState.Complete({ exit })
            })
          }
          return TransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.InProgress()
          })
        }
      })
    },

    ApprovalSubmit: ({ state }) => {
      return ApprovalSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(approveTransfer(chain, connectedWallet, params))
          return TransferSubmission.ApprovalSubmit({
            state: ApprovalSubmitState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            // Stay in ApprovalSubmit with the error in the Complete state
            return TransferSubmission.ApprovalSubmit({
              state: ApprovalSubmitState.Complete({ exit })
            })
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
            // Stay in TransferSubmit with the error in the Complete state
            return TransferSubmission.TransferSubmit({
              state: TransferSubmitState.Complete({ exit })
            })
          }
          return TransferSubmission.Filling()
        }
      })
    }
  })
}
