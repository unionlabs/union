import { Data, Schema } from "effect"
import { TransferT } from "@unionlabs/sdk/schema"
import type { TransferSubmission as EvmTransferSubmission } from "$lib/services/transfer-ucs03-evm"
import type { TransferSubmission as CosmosTransferSubmission } from "$lib/services/transfer-ucs03-cosmos"
import type { TransferSubmission as AptosTransferSubmission } from "$lib/services/transfer-ucs03-aptos"

export type TransferState = Data.TaggedEnum<{
  Empty: {}
  Evm: { state: EvmTransferSubmission }
  Cosmos: { state: CosmosTransferSubmission }
  Aptos: { state: AptosTransferSubmission }
}>

const TransferStateInternal = Data.taggedEnum<TransferState>()

export const TransferState = {
  Empty: () => TransferStateInternal.Empty(),
  Evm: (state: EvmTransferSubmission) => TransferStateInternal.Evm({ state }),
  Aptos: (state: AptosTransferSubmission) => TransferStateInternal.Aptos({ state }),
  Cosmos: (state: CosmosTransferSubmission) => TransferStateInternal.Cosmos({ state })
}

export type TransferStateUnion = TransferState

/**
 * NOTE: We need to provide full set of validation errors to user, thus `errors: "all"`.
 */
export const validateTransfer = Schema.decodeEither(TransferT, { errors: "all" })