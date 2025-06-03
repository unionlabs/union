import type { SuiClient } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { Effect } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { SuiReadContractError, SuiWriteContractError } from "./client.js"




export const readContract = <T>(
  client: SuiClient,
  sender: string,
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
  tx: Transaction,
) =>
  Effect.tryPromise({
    try: async () => {
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      // dev-inspect it
      const result = await client.devInspectTransactionBlock({
        transactionBlock: tx,
        sender,
      })
      return result.results // result as unknown as T
    },
    catch: e => new SuiReadContractError({ cause: extractErrorDetails(e as Error) }),
  }).pipe(
    // optional: e.g. timeout & retry like your Aptos wrapper
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )

export const writeContract = (
  client: SuiClient,
  signer: Ed25519Keypair,
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
  tx: Transaction,
) =>
  Effect.tryPromise({
    try: async () => {
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      // sign & execute
      const res = await client.signAndExecuteTransaction({
        signer,
        transaction: tx,
      })
      return res
    },
    catch: e => new SuiWriteContractError({ cause: extractErrorDetails(e as Error) }),
  })
