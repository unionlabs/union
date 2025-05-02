import { Effect } from "effect"
import type { Hex } from "viem"
import { ucs03abi } from "./abi/ucs03.js"
import { EvmChannelDestination } from "./channel.js"
import { ViemPublicClientDestination } from "./client.js"
import { readContract } from "./contract.js"

export const channelBalance = (path: bigint, token: Hex) =>
  Effect.gen(function*() {
    const client = (yield* ViemPublicClientDestination).client
    const config = yield* EvmChannelDestination

    const result = yield* readContract(client, {
      address: config.ucs03address,
      abi: ucs03abi,
      functionName: "channelBalance",
      args: [config.channelId, path, token],
    })

    return result
  })
