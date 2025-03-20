import { Effect } from "effect"
import { createSigningCosmWasmClient } from "../src/cosmos/client.ts"
import { executeContract } from "../src/cosmos/contract.ts"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a wallet from mnemonic (in a real app, use a secure method to get this)
    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic("memo memo memo", { prefix: "stars" })
    )

    // Get the first account address
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    // Create a signing client
    const client = yield* createSigningCosmWasmClient(
      "https://rpc.elgafar-1.stargaze-apis.com",
      wallet
    )

    // Execute a CW20 token transfer
    const result = yield* executeContract(
      client,
      firstAccount.address,
      "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr", // Example CW20 contract address
      {
        transfer: {
          recipient: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
          amount: "1"
        }
      }
    )

    return result
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
