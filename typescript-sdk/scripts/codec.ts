import { bech32AddressToHex, hexAddressToBech32 } from "#convert.ts"

console.info(
  hexAddressToBech32({
    bech32Prefix: "union",
    address: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"
  })
)
