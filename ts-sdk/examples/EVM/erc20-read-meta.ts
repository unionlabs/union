/**
 * @title ERC20 Read Meta
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
import { Evm } from "@unionlabs/sdk"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Effect } from "effect"
import { http } from "viem"
import { sepolia } from "viem/chains"

const client = Evm.PublicClient.Live({
  chain: sepolia,
  transport: http(),
})

const program = Evm.readErc20Meta(
  "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238", // USDC on Sepolia
  UniversalChainId.make("ethereum.11155111"),
).pipe(
  Effect.provide(client),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
