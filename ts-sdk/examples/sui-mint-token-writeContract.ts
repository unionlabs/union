import { getFullnodeUrl } from "@mysten/sui/client"
import { Effect } from "effect"
import { createSuiPublicClient } from "../src/sui/client.js"
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { Transaction } from '@mysten/sui/transactions';
import { writeContract } from "../src/sui/contract.js"
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }

    const publicClient = yield* createSuiPublicClient(config)

    const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC);

    const pkg = "0xacc51178ffc547cdfa36a8ab4a6ae3823edaa8f07ff9177d9d520aad080b28fd";
    const module = "fungible_token"
    const function_name = "mint"

    const tx = new Transaction()
    const function_arguments = [
      tx.object("0x634d588b8da56aec3256a09092425fcff6fdb2146b495c54773fa497c29fd8fd"),
      tx.pure.u64(4444),
      tx.pure.address("0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"),
    ]
    

    const result = yield* writeContract(
      publicClient,
      keypair,
      pkg,
      module,
      function_name,
      [],
      function_arguments,
      tx,
    )
    console.info("Transaction result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
