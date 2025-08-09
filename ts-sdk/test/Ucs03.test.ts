import { assert, describe, it } from "@effect/vitest"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Effect, Schema as S } from "effect"
import { toHex } from "viem"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

describe("UCS03", () => {
  describe("FungibleAssetOrder (0x03)", () => {
    it.effect("encodes V1", () =>
      Effect.gen(function*() {
        const fao = Ucs03.TokenOrderV1.fromOperand([
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          0n,
          "muno",
          "muno",
          6,
          100n,
          "0x16628cB81ffDA9B8470e16299eFa5F76bF45A579",
          100n,
        ])
        assert.equal(fao.version, 1)
      }))
    it.effect("encodes V2", () =>
      Effect.gen(function*() {
        /**
         * @see https://github.com/unionlabs/union/blob/f0c86086cd4110d4173a9138537925e163ef7220/evm/tests/src/05-app/Zkgm.t.sol#L6267-L6307
         */
        const fao = Ucs03.TokenOrderV2.fromOperand([
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          100n,
          0,
          "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
          "0x49aCf968c7E8807B39e980b2a924E97C8ead3a22",
          100n,
        ])
        assert.equal(fao.version, 2)
      }))
    it.effect("derives from operand", () =>
      Effect.gen(function*() {
        const v1 = [
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          0n,
          "muno",
          "muno",
          6,
          100n,
          "0x16628cB81ffDA9B8470e16299eFa5F76bF45A579",
          100n,
        ] as const
        const r1 = yield* S.decode(Ucs03.TokenOrder)({
          _tag: "FungibleAssetOrder",
          operand: v1,
        })
        assert.equal(r1.version, 1)

        const v2 = [
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          100n,
          0,
          "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
          "0x49aCf968c7E8807B39e980b2a924E97C8ead3a22",
          100n,
        ] as const
        const r2 = yield* S.decode(Ucs03.TokenOrder)({
          _tag: "FungibleAssetOrder",
          operand: v2,
        })
        assert.equal(r2.version, 2)
      }))

    it.effect("encodes", () =>
      Effect.gen(function*() {
        const v1 = Ucs03.TokenOrderV1.fromOperand([
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          0n,
          "muno",
          "muno",
          6,
          100n,
          "0x16628cB81ffDA9B8470e16299eFa5F76bF45A579",
          100n,
        ])
        const v2 = Ucs03.TokenOrderV2.fromOperand([
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          100n,
          0,
          "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
          "0x49aCf968c7E8807B39e980b2a924E97C8ead3a22",
          100n,
        ])

        const batch = Ucs03.Batch.fromOperand([v1, v2])

        const encoded = Ucs03.encode(batch)

        console.log(JSON.stringify(
          {
            batch,
            encoded,
          },
          null,
          2,
        ))
      }))
    it.effect("decodes", () =>
      Effect.gen(function*() {
        const fao = Ucs03.TokenOrderV1.fromOperand([
          toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
          toHex("0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD"),
          "0x6d756e6f",
          0n,
          "muno",
          "muno",
          6,
          100n,
          "0x16628cB81ffDA9B8470e16299eFa5F76bF45A579", // XXX: loses casing in decode
          100n,
        ])

        const encoded = Ucs03.encode(fao)

        const decoded = yield* S.decode(Ucs03.FungibleAssetOrderFromHex)(encoded)

        assert.deepStrictEqual(
          fao,
          decoded,
        )
      }))

    it.effect("decodes batch", () =>
      Effect.gen(function*() {
        const instruction =
          "0x00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000004400000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000149c968b805a625303ad43fce99ae72306256fe5f9000000000000000000000000000000000000000000000000000000000000000000000000000000000000018499f0385300000000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba000000000000000000000000" as const

        const decoded = yield* S.decode(Ucs03.Ucs03FromHex)(instruction)

        console.log({ decoded })
      }))

    it.effect.only("sol", () =>
      Effect.gen(function*() {
        const SENDER = Ucs05.AddressEvmZkgm.make(
          "0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d",
        )
        const RECEIVER = Ucs05.AddressCosmosZkgm.make(
          toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"),
        )

        const instruction = Ucs03.TokenOrderV2.fromOperand([
          SENDER,
          RECEIVER,
          "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
          10n,
          1,
          "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
          toHex("bbn1gm8473g2vszxepfyn884trrxtgkyf8572wa4csev5t8hjumja7csnllkkr"),
          10n,
        ])

        const encoded = yield* S.encode(Ucs03.Ucs03FromHex)(instruction)

        console.log("scratch", { encoded })
      }))
  })
})
