// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function () {
    return this.toString()
  }
}
import { Effect, Logger } from "effect"
import { getFullnodeUrl } from "@mysten/sui/client"
import { PublicClient, WalletClient, writeContract, readCoinMetadata, readCoinBalances } from "../src/Sui.js"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"

const MNEMONIC = process.env.MNEMONIC ?? "fix auto gallery heart practice drip joke nice decline lift attend bread"
const RECIPIENT = process.env.RECIPIENT ?? "0x03ff9dd9e093387bdd4432c6a3eb6a1bd5a8f39a530042ac7efe576f18d3232b"

const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)


const program = Effect.gen(function* () {
  const { client } = yield* PublicClient
  yield* Effect.log("Sui public client initialized", client.network )
  const meta = yield* readCoinMetadata("0x2::sui::SUI" as any)
  yield* Effect.log("SUI metadata", meta)

  yield* Effect.log("keypair.getPublicKey().toSuiAddress()", keypair.getPublicKey().toSuiAddress())
  const balances = yield* readCoinBalances("0x2::sui::SUI" as any, keypair.getPublicKey().toSuiAddress() as any)
  yield* Effect.log("SUI balances", balances)


  const wallet = yield* WalletClient
  const amountMist = 10_000_000n // 0.01 SUI

  const tx = new Transaction()
  const coin = tx.splitCoins(tx.gas, [tx.pure.u64(amountMist)])
  const recipient = tx.pure.address(RECIPIENT)

  const res = yield* writeContract(
    client,
    keypair,
    "0x2",                                // packageId: Sui framework
    "transfer",                           // module: sui::transfer
    "public_transfer",                    // function
    ["0x2::coin::Coin<0x2::sui::SUI>"],   // type arg T
    [coin, recipient],                    // (obj: T, recipient: address)
    tx,
  )

  yield* Effect.log("Transfer submitted", res)

  
}).pipe(
  Effect.provide(PublicClient.Live({ url: getFullnodeUrl("testnet") })),
  Effect.provide(
    WalletClient.Live({
      url: getFullnodeUrl("testnet"),
      account: keypair,              // signer
      chain: "sui-testnet" as any,   // placeholder; not used internally
    }),
  ),

  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program).catch(console.error)
