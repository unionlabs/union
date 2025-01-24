import type { FormFields } from "$lib/components/TransferFrom/transfer/raw-intents.ts"

export const TRANSFER_DEBUG = true

export const defaultParams: FormFields = {
  source: "17000",
  destination: "11155111",
  asset: "",
  receiver: "",
  amount: ""
}
