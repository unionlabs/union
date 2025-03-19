import { Effect } from "effect"
import { readErc20Meta } from "../src/evm/erc20.js"
import { ViemPublicClient } from "../src/evm/client.js"
import { createPublicClient, http } from "viem"
import { sepolia } from "viem/chains"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Create a public client for Sepolia
const client = createPublicClient({
  chain: sepolia,
  transport: http()
})

// USDC on Sepolia
const tokenAddress = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

// Read ERC20 metadata
readErc20Meta(tokenAddress)
  .pipe(Effect.provideService(ViemPublicClient, { client }), Effect.runPromiseExit)
  .then(exit => console.log(JSON.stringify(exit, null, 2)))
