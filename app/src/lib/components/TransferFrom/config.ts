import type { RawTransferIntents } from "$lib/components/TransferFrom/transfer/intents.ts"

export const TRANSFER_DEBUG = true

export const defaultParams: RawTransferIntents = {
  source: "union-testnet-8",
  destination: "",
  asset: "muno",
  receiver: "",
  amount: "",
  isValid: false
}
