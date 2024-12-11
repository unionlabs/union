import type { RawTransferIntents } from "$lib/components/TransferFrom/transfer/intents.ts"

export const TRANSFER_DEBUG = true

export const defaultParams: RawTransferIntents = {
  source: "union-testnet-8",
  destination: "11155111",
  asset: "",
  receiver: "",
  amount: "",
  isValid: false
}
