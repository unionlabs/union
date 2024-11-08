import { ResultAsync } from "neverthrow"
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { GasPrice, SigningStargateClient } from "@cosmjs/stargate"
import type { OfflineSigner as CosmosOfflineSigner } from "../types.ts"

/**
 * connect a stargate client with a signer
 * @example
 * ```ts
 * const client = await connectStargateWithSigner({
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 * })
 *
 * if (client.isOk()) {
 *   const tx = await client.value.getTx("A6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B")
 * }
 * ```
 */
export function connectStargateWithSigner({
  rpcUrl,
  account,
  gasPrice
}: {
  rpcUrl: string
  account: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}): ResultAsync<SigningStargateClient, Error> {
  return ResultAsync.fromPromise(
    SigningStargateClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => {
      console.error("@unionlabs/client-[connectStargateWithSigner]", error)
      return new Error("Failed to connect with stargate signer", { cause: error })
    }
  )
}

/**
 * connect a stargate client with a signer
 * @example
 * ```ts
 * const client = await connectCosmwasmWithSigner({
 *   account: cosmosAccount,
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   gasPrice: { amount: "0.0025", denom: "muno" },
 * })
 *
 * if (client.isOk()) {
 *   const tx = await client.value.getTx("A6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B")
 * }
 * ```
 */
export function connectCosmwasmWithSigner({
  rpcUrl,
  account,
  gasPrice
}: {
  rpcUrl: string
  account: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}): ResultAsync<SigningCosmWasmClient, Error> {
  return ResultAsync.fromPromise(
    SigningCosmWasmClient.connectWithSigner(rpcUrl, account, {
      gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
    }),
    error => {
      console.error("@unionlabs/client-[connectCosmwasmWithSigner]", error)
      return new Error("Failed to connect with cosmwasm signer", { cause: error })
    }
  )
}
