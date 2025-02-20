import type { FormFields } from "$lib/components/TransferCube/transfer/raw-intents.ts"

export const TRANSFER_DEBUG = true

export const defaultParams: FormFields = {
  source: "union-testnet-9",
  destination: "17000",
  asset: "0x6d756e6f",
  receiver: "",
  amount: ""
}
