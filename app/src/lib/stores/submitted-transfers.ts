import { persisted } from "svelte-persisted-store"

export const submittedTransfers = persisted<
  Record<
    string,
    {
      // type marker hack
      _is_submitted_transfer: true
      source_chain_id: string
      destination_chain_id: string
      packet_send_transaction_hash: string
      sender_normalized: string
      transfer_day: string
      receiver_normalized: string
      base_token: string
      base_amount: bigint
    }
  >
>("submittedTransfers", {})
