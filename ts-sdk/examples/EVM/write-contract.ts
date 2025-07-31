/**
 * @title Write Contract
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
import { Evm, Ucs03 } from "@unionlabs/sdk"
import { Effect } from "effect"
import { http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"

const account = privateKeyToAccount("0x...")

const wallet = Evm.WalletClient.Live({
  account,
  chain: sepolia,
  transport: http(),
})

const program = Evm.writeContract({
  account,
  chain: sepolia,
  address: "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3", // token address
  abi: Ucs03.Abi,
  functionName: "pause",
}).pipe(
  Effect.flip,
  Effect.provide(wallet),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
