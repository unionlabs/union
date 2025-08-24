/**
 * @title ERC20 Increase Allowance
 * @summary
 * Allowance on Sepolia for USDC spending is increased by `1` atomic unit.
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
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"

const account = privateKeyToAccount("0x...")

const wallet = Evm.WalletClient.Live({
  account,
  chain: sepolia,
  transport: http(),
})
const client = Evm.PublicClient.Live({
  chain: sepolia,
  transport: http(),
})

const TOKEN_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238" // USDC on Sepolia
const SPENDER_ADDRESS = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03" // UCS03 on Sepolia

const readAllowance = Evm.readErc20Allowance(
  TOKEN_ADDRESS,
  account.address,
  SPENDER_ADDRESS,
)

const readMeta = Evm.readErc20Meta(
  TOKEN_ADDRESS,
  UniversalChainId.make("ethereum.11155111"),
)

const increaseAllowance = Effect.fn(
  (amount: bigint) => Evm.increaseErc20Allowance(TOKEN_ADDRESS, SPENDER_ADDRESS, amount),
)

const program = Effect.gen(function*() {
  const currentAllowance = yield* readAllowance
  yield* Effect.log(`Current allowance: ${currentAllowance}`)

  yield* Effect.log(`Increasing allowance...`)
  const txHash = yield* increaseAllowance(currentAllowance + 1n)

  yield* Effect.log(`Waiting for transaction ${txHash}...`)
  const receipt = yield* Evm.waitForTransactionReceipt(txHash)
  yield* Effect.log(`Transaction confirmed in block: ${receipt.blockNumber}`)
  const newAllowance = yield* readAllowance

  return {
    ...(yield* readMeta),
    previousAllowance: currentAllowance,
    newAllowance,
    transactionHash: txHash,
    receipt: {
      blockNumber: receipt.blockNumber,
      status: receipt.status,
    },
  }
}).pipe(
  Effect.provide(client),
  Effect.provide(wallet),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
