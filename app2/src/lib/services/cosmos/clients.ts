import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { Effect } from "effect"

export const getCosmWasmClient = (rpc: URL | string) =>
  Effect.tryPromise({
    try: () => {
      const rpcString = typeof rpc === "string" ? rpc : rpc.toString()
      return CosmWasmClient.connect(rpcString)
    },
    catch: err =>
      new Error(`Failed to create CosmWasm client with RPC ${rpc} ${err}`, { cause: err })
  })
