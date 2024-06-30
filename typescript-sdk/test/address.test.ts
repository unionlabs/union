import { raise } from "#utilities/index.ts"
import { expect, it, describe } from "vitest"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { bech32AddressToHex, hexAddressToBech32, hexStringToUint8Array } from "#convert.ts"

const PRIVATE_KEY = "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380"

const { address: evmAddress } = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const [unionAccount] = await cosmosAccount.getAccounts()
if (!unionAccount) raise("Account not found")
const { address: unionAddress } = unionAccount

describe("union to evm address converter", () => {
  it("should convert union address to evm address", () => {
    expect(bech32AddressToHex({ address: unionAddress })).toEqual(evmAddress)
  })
})

describe("evm to cosmos address converter", () => {
  it("should convert evm address to cosmos address", () => {
    expect(hexAddressToBech32({ address: evmAddress, bech32Prefix: "union" })).toEqual(unionAddress)
  })
})
