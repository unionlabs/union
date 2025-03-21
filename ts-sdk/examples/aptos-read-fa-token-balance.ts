import { Effect } from "effect"
import { AptosPublicClient, createAptosPublicClient } from "../src/aptos/client.ts"
import { readFaBalance } from "../src/aptos/fa.ts"
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

    const real_token_address = "0x188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c"

    const result_balance = yield* readFaBalance(
      real_token_address,
      account.accountAddress.toString()
    ).pipe(Effect.provideService(AptosPublicClient, { client: publicClient }))
    yield* Effect.log("Result Balance:", result_balance)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
