import { Effect } from "effect"
import { AptosPublicClient, createAptosPublicClient } from "../src/aptos/client.ts"
import { readContract, writeContract } from "../src/aptos/contract.ts"
import { waitForTransactionReceipt } from "../src/aptos/receipts.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { AptosConfig, Network, MoveVector } from "@aptos-labs/ts-sdk"

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

    const zkgm_address = "0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84"
    const real_token_address = "0x188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c"
    const token_address = "0x6d756e6f"
    const module_name = "ibc_app"

    const function_name = "predict_wrapped_token"
    const destination_channel_id = "2"
    const base_token = MoveVector.U8(token_address)
    const functionArguments = [0, destination_channel_id, base_token]

    const result = yield* readContract(
      publicClient,
      zkgm_address,
      module_name,
      function_name,
      [], // type arguments
      functionArguments
    ).pipe(Effect.provideService(AptosPublicClient, { client: publicClient }))
    yield* Effect.log("Result:", result)

    if (result[0] === real_token_address) {
      yield* Effect.log("Success")
    } else {
      yield* Effect.logError("Failure")
    }

    const contract_address = "0x1"
    const balance_module_name = "primary_fungible_store"
    const balance_function_name = "balance"
    const balance_typeArguments = ["0x1::fungible_asset::Metadata"]
    const balance_functionArguments = [account.accountAddress.toString(), result[0]]

    const result_balance = yield* readContract(
      publicClient,
      contract_address,
      balance_module_name,
      balance_function_name,
      balance_typeArguments,
      balance_functionArguments
    ).pipe(Effect.provideService(AptosPublicClient, { client: publicClient }))
    yield* Effect.log("Result Balance:", result_balance)

    const transfer_module_name = "primary_fungible_store"
    const transfer_function_name = "transfer"
    const transfer_typeArguments = ["0x1::fungible_asset::Metadata"]
    const receiver_address = "0x9ec0ea9b728dd1aa4f0b9b779e7f885099bcea7d28f88f357982d7de746183c9"
    const transfer_amount = 1
    const transfer_functionArguments = [result[0], receiver_address, transfer_amount]

    yield* Effect.log("transfer_functionArguments:", transfer_functionArguments)

    const result_execute = yield* writeContract(
      publicClient,
      account,
      contract_address,
      transfer_module_name,
      transfer_function_name,
      transfer_typeArguments,
      transfer_functionArguments
    ).pipe(Effect.provideService(AptosPublicClient, { client: publicClient }))

    const txHash = yield* waitForTransactionReceipt(result_execute.hash).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("transaction receipt:", txHash)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
