import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { hexStringToUint8Array, uint8ArrayToHexString } from "../src/mod.ts"

const PRIVATE_KEY = "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380"

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)
const { address: evmAddress, publicKey: evmHexPublicKey } = evmAccount
const evmUint8ArrayPublicKey = hexStringToUint8Array(evmHexPublicKey)
//
const cosmosWallet = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const [cosmosAccount] = await cosmosWallet.getAccounts()
const cosmosAddress = cosmosAccount?.address
const cosmosUint8ArrayPublicKey = cosmosAccount?.pubkey as Uint8Array
const cosmosHexPublicKey = uint8ArrayToHexString(cosmosUint8ArrayPublicKey)

console.info(
  { evmAddress, evmHexPublicKey, evmUint8ArrayPublicKey },
  { cosmosAddress, cosmosHexPublicKey, cosmosUint8ArrayPublicKey }
)
