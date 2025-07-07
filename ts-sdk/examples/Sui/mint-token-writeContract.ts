/**
 * @title Mint Token Write
 * @badge WIP:caution
 */
/// <reference types="effect" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
/*
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { Effect } from "effect"
import { createSuiPublicClient } from "../src/sui/client.js"
import { writeContract } from "../src/sui/contract.js"
const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }

    const publicClient = yield* createSuiPublicClient(config)

    const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)

    const pkg = "0xd32f121aec92e5179398e21ab9beb366d854b6f985bb326266228271c3697c95"
    const module = "fungible_token"
    const function_name = "mint"

    const tx = new Transaction()
    const function_arguments = [
      tx.object("0xa030755da2b5fed0a54c8eb7ef5383ac2ad9991d79e47fb9de7af77507124159"),
      tx.pure.u64(100),
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
*/
