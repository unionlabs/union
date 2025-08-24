/**
 * @title Read & Write Allowance
 * @summary
 * Concurrently read token metadata, ablance, and allowance.
 * @badge WIP:caution
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
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { Cosmos, Ucs05 } from "@unionlabs/sdk"
import { Effect } from "effect"

const RPC_URL = "https://rpc.union-testnet-10.union.chain.kitchen"

const wallet = await DirectSecp256k1HdWallet.fromMnemonic("memo memo memo", { prefix: "union" })
const [account] = await wallet.getAccounts()

const client = Cosmos.Client.Live(RPC_URL)
const signingClient = Cosmos.SigningClient.Live(
  RPC_URL,
  wallet,
  { gasPrice: GasPrice.fromString("0.025muno") },
)

const program = Effect.gen(function*() {
  const contractAddress = Ucs05.AddressCosmosDisplay.make(
    "union13pxktu2hk8pseksaaka54ngxyfmpjljrleh3cc8sxvq4dxalvttqdmdgv5",
  )
  const spender = Ucs05.AddressCosmosDisplay.make(
    "union1x2jzeup7uwfxjxxrtfna2ktcugltntgu6kvc0eeayk0d82l247cqz669ee",
  )

  const allowance = yield* Cosmos.readCw20Allowance(
    contractAddress,
    account.address as unknown as any,
    spender,
  )
  console.info("Current allowance:", allowance.toString())

  yield* Cosmos.writeCw20IncreaseAllowance(contractAddress, account.address, spender, "1")

  const allowance_after = yield* Cosmos.readCw20Allowance(
    contractAddress,
    Ucs05.AddressCosmosDisplay.make(account.address as unknown as any),
    spender,
  )

  console.info("allowance after increasing:", allowance_after.toString())
}).pipe(
  Effect.provide(client),
  Effect.provide(signingClient),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
