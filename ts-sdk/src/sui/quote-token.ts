import { Transaction } from "@mysten/sui/transactions"
import { Effect } from "effect"
import { SuiChannelDestination } from "./channel.js"
import { SuiPublicClientDestination } from "./client.js"
import { readContract } from "./contract.js"
export type Hex = `0x${string}`

// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
}

function bytesToHex(bytes: number[]) {
  return "0x" + bytes.map(b => b.toString(16).padStart(2, "0")).join("")
}

export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function*() {
    const client = (yield* SuiPublicClientDestination).client
    const config = yield* SuiChannelDestination
    yield* Effect.log(
      `Predicting quote token for base token: ${baseToken} at channel: ${config.channelId} on ZKGM Address: ${config.ucs03address}`,
    )

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "compute_salt"
    const converted_base_token = baseToken

    const tx = new Transaction()
    const function_arguments = [
      tx.pure.u256(0),
      tx.pure.u32(config.channelId),
      tx.pure("vector<u8>", hexToBytes(converted_base_token)),
    ]

    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx,
    )

    if (!result || result.length === 0 || !result[0].returnValues || !result[0].returnValues[0]) {
      throw new Error("No return value from compute_salt")
    }
    const [rawBytes /*, _typeTag*/] = result[0].returnValues[0] as [number[], string]

    return bytesToHex(rawBytes.slice(1))
  })
