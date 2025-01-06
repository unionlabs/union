import type { FormFields } from "$lib/components/TransferFrom/transfer/intents.ts"

export const TRANSFER_DEBUG = true

export const defaultParams: FormFields = {
  source: "union-testnet-8",
  destination: "",
  asset: "",
  receiver: "",
  amount: ""
}
