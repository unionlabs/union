import {
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState
} from "./state.ts"
import { Effect } from "effect"
import { switchChainAptos } from "./chain.ts"
import { submitTransferAptos, waitForTransferReceiptAptos } from "./transactions-aptos.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"

/**
 * This state machine is dedicated to Aptos transfers.
 * It goes from Filling -> SwitchChain -> TransferSubmit -> TransferReceipt.
 * No approval steps are required.
 */
export async function nextStateAptos(
  ts: TransferSubmission,
  params: ValidTransfer["args"],
  chain: Chain
): Promise<TransferSubmission> {
  return TransferSubmission.$match(ts, {
    // Initially, we’re in the Filling state.
    Filling: () => {
      return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
      // TODO: skip switch chain state for now
    },

    // SwitchChain state: Use the Aptos-specific switch function.
    SwitchChain: ({ state }) => {
      return SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(switchChainAptos(chain))
          return TransferSubmission.SwitchChain({ state: SwitchChainState.Complete({ exit }) })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
          }
          // Once switched, move directly to TransferSubmit.
          return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
        }
      })
    },

    // TransferSubmit state: Submit the transfer using Aptos-specific logic.
    TransferSubmit: ({ state }) => {
      return TransferSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(submitTransferAptos(chain, params))
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
          }
          // After successful submission, move to waiting for receipt.
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.InProgress({ hash: exit.value.hash })
          })
        }
      })
    },

    // TransferReceipt state: Wait for the transfer receipt.
    TransferReceipt: ({ state }) => {
      return TransferReceiptState.$match(state, {
        InProgress: async ({ hash }) => {
          const exit = await Effect.runPromiseExit(waitForTransferReceiptAptos(chain, hash))
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit })
          })
        },
        Complete: ({ exit }) => {
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit })
          })
        }
      })
    }
  })
}
