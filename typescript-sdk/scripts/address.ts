import { hexToBytes, bytesToHex } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { Account as AptosAccount, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"

const PRIVATE_KEY = "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380"

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)
const { address: evmAddress, publicKey: evmHexPublicKey } = evmAccount
const evmUint8ArrayPublicKey = hexToBytes(evmHexPublicKey)

const cosmosWallet = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexToBytes(PRIVATE_KEY)),
  "union"
)
const [cosmosAccount] = await cosmosWallet.getAccounts()
const cosmosAddress = cosmosAccount?.address
const cosmosUint8ArrayPublicKey = cosmosAccount?.pubkey as Uint8Array
const cosmosHexPublicKey = bytesToHex(cosmosUint8ArrayPublicKey)

const aptosAccount = AptosAccount.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(hexToBytes(PRIVATE_KEY))
})
const aptosAddress = aptosAccount.accountAddress.toString()
const aptosPublicKey = aptosAccount.publicKey.toString()
const aptosUint8ArrayPublicKey = aptosAccount.publicKey.toUint8Array()

console.info(
  { evmAddress, evmHexPublicKey, evmUint8ArrayPublicKey },
  { cosmosAddress, cosmosHexPublicKey, cosmosUint8ArrayPublicKey },
  { aptosAddress, aptosPublicKey, aptosUint8ArrayPublicKey }
)
