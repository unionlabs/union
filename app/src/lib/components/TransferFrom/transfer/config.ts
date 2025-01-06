import type {FormFields} from "$lib/components/TransferFrom/transfer/raw-intents.ts";

export const TRANSFER_DEBUG = true

export const defaultParams: FormFields = {
  source: "union-testnet-9",
  destination: "",
  asset: "",
  receiver: "",
  amount: ""
}
