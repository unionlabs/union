import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Effect } from "effect";

export const getCosmWasmClient = (rpc: string) =>
  Effect.tryPromise({
    try: () => CosmWasmClient.connect(rpc),
    catch: (err) => new Error(`Failed to create CosmWasm client with RPC ${rpc} ${err}`, { cause: err })
  });