import { bcs } from "@mysten/sui/bcs"
import { Transaction } from "@mysten/sui/transactions"
import { Effect } from "effect"
import type { Hex } from "viem"
import { SuiChannelDestination } from "./channel.js"
import { SuiPublicClientDestination } from "./client.js"
import { readContract } from "./contract.js"

// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
}

export const channelBalance = (path: number, token: Hex, relayStore: Hex) =>
  Effect.gen(function*() {
    const client = (yield* SuiPublicClientDestination).client
    const config = yield* SuiChannelDestination

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "channel_balance"

    const tx = new Transaction()
    const function_arguments = [
      tx.object(relayStore),
      tx.pure.u32(config.channelId),
      tx.pure.u256(path),
      tx.pure("vector<u8>", hexToBytes(token)),
    ]

    yield* Effect.log("Getting channel_balance for token:", token)

    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779", // TODO:
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx,
    )
    const [bytesArray] = result[0].returnValues[0] as [number[], string]
    const data = new Uint8Array(bytesArray)
    const decoded = bcs.U256.parse(data)

    return decoded
  })
