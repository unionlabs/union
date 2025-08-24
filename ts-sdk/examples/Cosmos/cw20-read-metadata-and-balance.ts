/**
 * @title Read Metadata & Balance
 * @summary
 * Concurrently read token metadata, ablance, and allowance.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="@cosmjs/stargate" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
import { Cosmos, Ucs05 } from "@unionlabs/sdk"
import { Effect } from "effect"

// Contract address and wallet address
const contractAddress = Ucs05.AddressCosmosDisplay.make(
  "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr",
) // WETH on stargaze
const walletAddress = Ucs05.AddressCosmosDisplay.make(
  "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
) // The address to which to check balance
const spender = Ucs05.AddressCosmosDisplay.make(
  "stars14qemq0vw6y3gc3u3e0aty2e764u4gs5lddqqxv",
)
const client = Cosmos.Client.Live("https://rpc.elgafar-1.stargaze-apis.com")

const program = Effect.all({
  // Read CW20 token info
  tokenInfo: Cosmos.readCw20TokenInfo(contractAddress),
  // Read CW20 token balance
  balance: Cosmos.readCw20Balance(contractAddress, walletAddress),
  // Read CW20 token allowance
  allowance: Cosmos.readCw20Allowance(contractAddress, walletAddress, spender),
  // Combine the results
}, { concurrency: "unbounded" }).pipe(
  Effect.provide(client),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
