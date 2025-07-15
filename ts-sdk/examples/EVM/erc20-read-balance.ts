/**
 * @title ERC20 Read Balance
 * @summary
 * Concurrently read balance and token metadata.
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

const TOKEN_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const client = Evm.PublicClient.Live({
  chain: sepolia,
  transport: http(),
})

const program = Effect.all({
  metadata: Evm.readErc20Meta(
    TOKEN_ADDRESS,
    UniversalChainId.make("ethereum.11155111"),
  ),
  balance: Evm.readErc20Balance(
    TOKEN_ADDRESS,
    "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
  ),
}, { concurrency: 2 }).pipe(
  Effect.provide(client),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
