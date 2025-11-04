import { runPromiseExit } from "$lib/runtime"
import { approveTransfer, waitForApprovalReceipt } from "$lib/services/transfer-ucs03-evm/approval"
import { EvmSwitchChainError } from "$lib/services/transfer-ucs03-evm/errors"
import type { ValidTransfer } from "@unionlabs/sdk/schema"
import { Effect, Option } from "effect"
import type { SwitchChainErrorType } from "viem"
import { SwitchChain } from "./chain"
import {
  SwitchChainState,
  TransferReceiptState,
  TransferSubmission,
  TransferSubmitState,
} from "./state"
import { submitTransfer, waitForTransferReceipt } from "./transactions"

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
          const exit = await runPromiseExit(SwitchChain(params.sourceChain, {}))
          return TransferSubmission.SwitchChain({ state: SwitchChainState.Complete({ exit }) })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.SwitchChain({ state: SwitchChainState.InProgress() })
          }
          return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
        },
      })
    },

    TransferSubmit: ({ state }) => {
      return TransferSubmitState.$match(state, {
        InProgress: async () => {
          const exit = await runPromiseExit(submitTransfer(params.sourceChain, params))
          return TransferSubmission.TransferSubmit({
            state: TransferSubmitState.Complete({ exit }),
          })
        },
        Complete: ({ exit }) => {
          if (exit._tag === "Failure") {
            return TransferSubmission.TransferSubmit({ state: TransferSubmitState.InProgress() })
          }
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.InProgress({
              // @ts-ignore-error
              hash: exit.value,
            }),
          })
        },
      })
    },

    TransferReceipt: ({ state }) => {
      return TransferReceiptState.$match(state, {
        InProgress: async ({ digest }) => {
          const exit = await runPromiseExit(waitForTransferReceipt(params.sourceChain, digest))
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit }),
          })
        },
        Complete: ({ exit }) => {
          // TODO: there is no real next state here
          return TransferSubmission.TransferReceipt({
            state: TransferReceiptState.Complete({ exit }),
          })
        },
      })
    },
  })
}
