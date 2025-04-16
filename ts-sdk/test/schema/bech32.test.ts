import { assert, describe, it } from "vitest"
import { Either, Schema as S } from "effect"
import {
  AddressCosmosZkgm,
  Bech32,
  Bech32FromAddressCanonicalBytesWithPrefix,
  HexFromString
} from "@unionlabs/sdk/schema"

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
    "?1ezyfcl"
  ])("decodes valid address %s", s => {
    assert.isTrue(Either.isRight(S.decodeUnknownEither(Bech32)(s)))
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
    Buffer.from("6465316c67377774ff", "hex").toString("binary")
  ])("fails invalid address %s", s => {
    assert.isTrue(Either.isLeft(S.decodeEither(Bech32)(s)))
  })

  it("Bech32FromAddressCanonicalBytesWithPrefix", () => {
    const addr = "0x52a648ef2157fd3bafa90bbac510b9a4870fdf36"
    const transform = Bech32FromAddressCanonicalBytesWithPrefix("bbn").pipe(
      S.compose(HexFromString),
      S.compose(AddressCosmosZkgm)
    )

    const result = S.decodeUnknownSync(transform)(addr)

    assert.isTrue(true)
  })
})
