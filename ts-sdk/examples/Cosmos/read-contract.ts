/**
 * @title Read Contract
 * @description Example read contract call for token balance.
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
import { Cosmos } from "@unionlabs/sdk"
import { Effect, pipe } from "effect"

// Create a CosmWasm client
const client = Cosmos.Client.Live("https://rpc.elgafar-1.stargaze-apis.com")

// Query a CW20 token contract for a balance
const program = pipe(
  Cosmos.queryContract<{ balance: string }>(
    // Example CW20 contract address
    "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr",
    {
      balance: {
        // The address for which to check balance
        address: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
      },
    },
  ),
  Effect.provide(client),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
