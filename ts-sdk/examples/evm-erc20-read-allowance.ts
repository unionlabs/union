import { Effect } from "effect"
import { ViemPublicClient } from "../src/evm/client.ts"
import { createPublicClient, http } from "viem"
import { sepolia } from "viem/chains"
import { readErc20Meta, readErc20Allowance } from "../src/evm/erc20.ts"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a client
    const client = createPublicClient({
      chain: sepolia,
      transport: http()
    })

    // Provide the client to the Effect context
    const withClient = Effect.provideService(ViemPublicClient, { client })

    // Token address, owner address, and spender address
    const tokenAddress = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
    const ownerAddress = "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
    const spenderAddress = "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa" // ucs03 on sepolia

    // Read ERC20 token metadata
    const metadata = yield* readErc20Meta(tokenAddress).pipe(withClient)

    // Read ERC20 token allowance
    const allowance = yield* readErc20Allowance(tokenAddress, ownerAddress, spenderAddress).pipe(
      withClient
    )

    // Combine the results
    return {
      ...metadata,
      allowance
    }
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
