import { Either, Schema as S } from "effect"
import { assert, describe, it } from "vitest"
import { PacketHashFromLenient } from "../../src/schema/packet.js"

describe("LenientPacketHash", () => {
  it.each([
    "0x072c786ad66c49fe4e8c46c06defd91772d893b4ad5c0f7eb0ac4522fde5e94b", // prefixed lower ("canonical")
    "0x072C786AD66C49FE4E8C46C06DEFD91772D893B4AD5C0F7EB0AC4522FDE5E94B", // prefixed upper
    "0x072C786AD66C49fe4e8c46c06DEFd91772D893b4AD5C0f7EB0ac4522Fde5E94B", // prefixed mixed
    "072c786ad66c49fe4e8c46c06defd91772d893b4ad5c0f7eb0ac4522fde5e94b", // unprefixed lower
    "072C786AD66C49FE4E8C46C06DEFD91772D893B4AD5C0F7EB0AC4522FDE5E94B", // unprefixed upper
    "072C786AD66C49fe4e8c46c06DEFd91772D893b4AD5C0f7EB0ac4522Fde5E94B", // unprefixed mixed
  ])("decodes valid packet hash %s", s => {
    const hash = S.decodeUnknownEither(PacketHashFromLenient)(s)
    console.log({ hash })
    assert.isTrue(Either.isRight(hash))
  })
})
