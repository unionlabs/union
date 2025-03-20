import { Effect } from "effect"
import { ViemPublicClient, ViemWalletClient } from "../src/evm/client.ts"
import { createPublicClient, createWalletClient, http, parseEther } from "viem"
import { sepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
import { increaseErc20Allowance, readErc20Allowance, readErc20Meta } from "../src/evm/erc20.ts"
import { waitForTransactionReceipt } from "../src/evm/receipts.ts"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Replace with your private key
const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create account from private key
    const chain = sepolia
    const account = privateKeyToAccount(PRIVATE_KEY as `0x${string}`)

    const publicClient = createPublicClient({ chain: sepolia, transport: http() })

    // Create a wallet client
    const client = createWalletClient({
      account,
      chain,
      transport: http()
    })

    // Token address and spender address
    const tokenAddress = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238" // USDC on sepolia
    const spenderAddress = "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa" // ucs03 on sepolia

    // Read ERC20 token metadata
    const metadata = yield* readErc20Meta(tokenAddress).pipe(
      Effect.provideService(ViemPublicClient, { client: publicClient })
    )

    // Read current allowance
    const currentAllowance = yield* readErc20Allowance(
      tokenAddress,
      account.address,
      spenderAddress
    ).pipe(
      Effect.provideService(ViemPublicClient, {
        client: publicClient
      })
    )

    console.log("Current allowance:", currentAllowance.toString())

    // Increase allowance
    const txHash = yield* increaseErc20Allowance(tokenAddress, spenderAddress, 420n).pipe(
      Effect.provideService(ViemWalletClient, { client, account, chain })
    )

    // Wait for transaction receipt
    const receipt = yield* waitForTransactionReceipt(txHash).pipe(
      Effect.provideService(ViemPublicClient, { client: publicClient })
    )

    console.log("Transaction confirmed in block:", receipt.blockNumber)

    // Read new allowance
    const newAllowance = yield* readErc20Allowance(
      tokenAddress,
      account.address,
      spenderAddress
    ).pipe(
      Effect.provideService(ViemPublicClient, {
        client: publicClient
      })
    )

    return {
      ...metadata,
      previousAllowance: currentAllowance,
      newAllowance,
      transactionHash: txHash,
      receipt: {
        blockNumber: receipt.blockNumber,
        status: receipt.status
      }
    }
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
