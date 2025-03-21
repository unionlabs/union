import { Effect } from "effect"
import { AptosPublicClient, createAptosPublicClient } from "../src/aptos/client.ts"
import { queryContract } from "../src/aptos/contract.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { Aptos, AptosConfig, Network, AptosApiError, MoveVector } from "@aptos-labs/ts-sdk"

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

    yield* Effect.log("Aptos publicclient:", publicClient)


    const zkgm_address = "0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84"
    const real_token_address = "0x188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c"
    const token_address = "0x6d756e6f"
    const function_name = "ibc_app::predict_wrapped_token"
    const typeArguments = []
    const destination_channel_id = "2"
    const base_token = MoveVector.U8(token_address)
    const functionArguments = [0,  destination_channel_id, base_token]

    const result = yield* queryContract(publicClient, zkgm_address, function_name, typeArguments, functionArguments).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield * Effect.log("Result:", result)

    if (result[0] === real_token_address){
      yield * Effect.log("Success")
    } else {
      yield * Effect.logError("Failure")
    }

    const contract_address = "0x1"
    const balance_function_name = "primary_fungible_store::balance"
    const balance_typeArguments = ["0x1::fungible_asset::Metadata"]
    const balance_functionArguments = [account.accountAddress.toString(), result[0]]

    const result_balance = yield* queryContract(publicClient, contract_address, balance_function_name, balance_typeArguments, balance_functionArguments).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield * Effect.log("Result Balance:", result_balance)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
