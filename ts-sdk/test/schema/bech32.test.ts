import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { HexFromString } from "@unionlabs/sdk/schema/hex"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Either, Schema as S } from "effect"
import { isHex } from "viem"
import { assert, describe, it } from "vitest"

/**
 * This test suite implements a subset of tests from
 * @see https://github.com/paulmillr/scure-base/blob/main/test/bech32.test.js
 */
describe("Bech32", () => {
  it.each([
    "A12UEL5L",
    "a12uel5l",
    "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1tt5tgs",
    "abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw",
    "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
    "split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w",
    "?1ezyfcl",
  ])("decodes valid address %s", s => {
    assert.isTrue(Either.isRight(S.decodeUnknownEither(Ucs05.Bech32)(s)))
  })

  it.each([
    "A12Uel5l",
    " 1nwldj5",
    "abc1rzg",
    "an84characterslonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1569pvx",
    "x1b4n0q5v",
    "1pzry9x0s0muk",
    "pzry9x0s0muk",
    "abc1rzgt4",
    "s1vcsyn",
    "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
    "li1dgmt3",
    Buffer.from("6465316c67377774ff", "hex").toString("binary"),
  ])("fails invalid address %s", s => {
    assert.isTrue(Either.isLeft(S.decodeUnknownEither(Ucs05.Bech32)(s)))
  })

  describe("Bech32FromAddressCanonicalBytesWithPrefix", () => {
    it("happy", () => {
      const addr = "0x52a648ef2157fd3bafa90bbac510b9a4870fdf36"
      const transform = Ucs05.Bech32FromCanonicalBytesWithPrefix("bbn").pipe(
        S.compose(HexFromString),
        S.compose(AddressCosmosZkgm),
      )

      const result = S.decodeUnknownSync(transform)(addr)

      assert.isTrue(isHex(result))
    })
    it.fails("guarantees given HRP matches", () => {
      const addr = "bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"
      const transform = Ucs05.Bech32FromCanonicalBytesWithPrefix("osmosis")

      const result = S.encodeUnknownSync(transform)(addr)
      console.log(result)

      assert.isTrue(isHex(result))
    })
  })
})
