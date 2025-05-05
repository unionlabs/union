import { Effect } from "effect"
import { createWalletClient, erc20Abi, http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"
import { ViemWalletClient } from "../src/evm/client.js"
import { writeContract } from "../src/evm/contract.js"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

// Example private key
const privateKey = "0xC0ffee"
const account = privateKeyToAccount(privateKey)

// Create a wallet client
const walletClient = createWalletClient({
  account,
  chain: sepolia,
  transport: http(),
})

const tokenAddress = "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3"

// Example transfer
writeContract(walletClient, {
  account,
  chain: sepolia,
  address: tokenAddress,
  abi: erc20Abi,
  functionName: "transfer",
  args: ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 2n],
})
  .pipe(
    Effect.provideService(ViemWalletClient, { client: walletClient }),
    Effect.mapError(e => e.cause.message),
    Effect.runPromiseExit,
  )
  .then(exit => console.log(JSON.stringify(exit, null, 2)))
