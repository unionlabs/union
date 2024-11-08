import { expect, it, describe } from "vitest"
import { bech32AddressToHex, hexAddressToBech32 } from "../src/convert.ts"

describe("union to evm address converter", () => {
  it("should convert union address to evm address", () => {
    expect(bech32AddressToHex({ address: "union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md" })).toEqual(
      "0xf2d614f14686d26e6a17ffa9adecc407231fd3f2"
    )
  })
})

describe("evm to cosmos address converter", () => {
  it("should convert evm address to cosmos address", () => {
    expect(
      hexAddressToBech32({
        bech32Prefix: "union",
        address: "0xf2d614f14686d26e6a17ffa9adecc407231fd3f2"
      })
    ).toEqual("union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md")
  })
})
