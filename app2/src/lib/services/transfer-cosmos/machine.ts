import {
  ApprovalSubmitState,
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "./state.ts"
import { Effect } from "effect"
import { switchChain } from "./chain.ts"
import { submitTransfer } from "./transactions.ts"
import { approveTransfer } from "./approval"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import type {CosmosWalletId} from "$lib/wallet/cosmos";

export async function nextState(
  ts: TransferSubmission,
  params: ValidTransfer["args"],
  chain: Chain,
  connectedWallet: CosmosWalletId,
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    Filling: () => {
      return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
    },

    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(switchChain(chain))
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
            return TransferSubmission.ApprovalSubmit({
              state: ApprovalSubmitState.InProgress()
            })
          }

          return TransferSubmission.TransferSubmit({
            state: ApprovalSubmitState.Complete({exit})
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
