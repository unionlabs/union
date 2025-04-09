import { describe, it, assert } from "@effect/vitest"
import { Chain } from "@unionlabs/sdk/schema/chain"
import { BaseTransfer, TransferT } from "@unionlabs/sdk/schema/transfer-args"
import { Arbitrary, Either, FastCheck as fc, Schema as S, Option } from "effect"

const chainArb = Arbitrary.make(Chain)
const cosmosChainArb = chainArb.filter(x => x.rpc_type === "cosmos")
const evmChainArb = chainArb.filter(x => x.rpc_type === "evm")

const baseTransferArb = Arbitrary.make(BaseTransfer)

describe("Transfer Args", () => {
  it("produces a chain", () => {
    const chain = fc.sample(chainArb, 1)[0]
    console.log({ chain })
    assert.isTrue(chain instanceof Chain)
  })

  it.only.each([
    [evmChainArb, cosmosChainArb],
    // [evmChainArb, evmChainArb],
  ])("does thing from evm -> cosmos", (sender, receiver) => {
    const sourceChain = fc.sample(sender, 1)[0]
    const destinationChain = fc.sample(receiver, 1)[0]
    const baseTransfer = fc.sample(
      baseTransferArb
      .map(x => ({
        ...x,
        sourceChain,
        destinationChain,
      })),
      1
    )[0]
    const result = S.decodeEither(TransferT)(baseTransfer)
    console.log({ baseTransfer, result })
    assert.isTrue(Either.isRight(result))

    assert.notStrictEqual(
      baseTransfer.receiver,
      // @ts-expect-error
      result.right.receiver
    )
  })
})
