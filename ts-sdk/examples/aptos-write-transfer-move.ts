import { Effect } from "effect"
import { AptosPublicClient, createAptosPublicClient } from "../src/aptos/client.js"
import { writeContract } from "../src/aptos/contract.js"
import { waitForTransactionReceipt } from "../src/aptos/receipts.js"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { AptosConfig, Network } from "@aptos-labs/ts-sdk"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Replace with your private key
const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create account from private key
    const privateKey = new Ed25519PrivateKey(PRIVATE_KEY)
    const account = Account.fromPrivateKey({ privateKey })

    const rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"

    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    const publicClient = yield* createAptosPublicClient(config)

    const contract_address = "0x1"
    const transfer_module_name = "aptos_account"
    const transfer_function_name = "transfer"
    const receiver_address = "0x9ec0ea9b728dd1aa4f0b9b779e7f885099bcea7d28f88f357982d7de746183c9"
    const transfer_amount = 100
    const transfer_functionArguments = [receiver_address, transfer_amount]

    yield* Effect.log("transfer_functionArguments:", transfer_functionArguments)

    const result_execute = yield* writeContract(
      publicClient,
      account,
      contract_address,
      transfer_module_name,
      transfer_function_name,
      [], // type arguments
      transfer_functionArguments
    ).pipe(Effect.provideService(AptosPublicClient, { client: publicClient }))

    const txHash = yield* waitForTransactionReceipt(result_execute.hash).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("transaction receipt:", txHash)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
