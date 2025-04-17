import { describe, it } from "@effect/vitest"
import * as Address from "@unionlabs/sdk/schema/address"
import * as S from "effect/Schema"

const validErc55s = [
  // All caps
  "0x52908400098527886E0F7030069857D2E4169EE7",
  "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
  // All Lower
  "0xde709f2102306220921060314715629080e2fb77",
  "0x27b1fdb04752bbc536007a920d24acb045561c26",
  // Normal
  "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
  "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
  "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
  "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"
]

const vaildBech32s = [
  "A12UEL5L",
  "a12uel5l",
  "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1tt5tgs",
  "abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw",
  "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
  "split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w",
  "?1ezyfcl"
]

describe("Address", () => {
  describe("ERC-55", () => {
    it.effect.each(validErc55s)("passes for %s", x => S.decode(Address.ERC55)(x))
  })

  describe("union", () => {
    it.effect.each([...validErc55s, ...vaildBech32s])("passes for %s", x =>
      S.decode(Address.MyUnion)(x)
    )
  })
})
