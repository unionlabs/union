import { expect, it, describe } from "vitest"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { unionToEvmAddress, evmToCosmosAddress, hexStringToUint8Array } from "#/convert.ts"

const PRIVATE_KEY = "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380"

const _evmAddress = "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"
const { address: evmAddress, publicKey: evmPublicKey } = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const unionDirectWallet = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)
const [unionAccount] = await unionDirectWallet.getAccounts()
if (!unionAccount) throw new Error("No account found")
const _unionAddress = "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
const { address: unionAddress, pubkey: unionPublicKey } = unionAccount

describe("union to evm address converter", () => {
  it("should convert union address to evm address", () => {
    expect(unionToEvmAddress(unionAddress)).toEqual(evmAddress)
  })
})

describe("evm to cosmos address converter", () => {
  it("should convert evm address to cosmos address", () => {
    expect(evmToCosmosAddress(evmAddress)).toEqual(unionAddress)
  })
})
