import { describe, it } from "@effect/vitest"
// import * as Batch from "@unionlabs/sdk/Batch"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
// import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
// import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
// import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import { Effect, Layer, Logger } from "effect"
// import * as S from "effect/Schema"

const Live = Layer.mergeAll(
  ChainRegistry.Default,
  Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault),
)

describe("Batch", () => {
  it.layer(Live)("name", (it) =>
    it.effect("live", () =>
      Effect.gen(function*() {
        // const source = yield* ChainRegistry.byUniversalId(
        //   UniversalChainId.make("ethereum.17000"),
        // )
        // const destination = yield* ChainRegistry.byUniversalId(
        //   UniversalChainId.make("ethereum.11155111"),
        // )
        // const order = yield* TokenOrder.make({
        //   source,
        //   destination,
        //   sender: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
        //   receiver: "0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302",
        //   // LINK on Holesky
        //   baseToken: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b",
        //   baseAmount: 10n,
        //   quoteToken: "0x80fdbf104ec58a527ec40f7b03f88c404ef4ba63",
        //   quoteAmount: 10n,
        //   kind: TokenOrder.Kind.Escrow,
        //   metadata: undefined,
        // })

        // const batch = Batch.make([order, order])
        // console.log(batch.toString())
        // const encoded = yield* batch.encode
        // console.log({ encoded })
        // const decodedInstruction = yield* S.decode(Ucs03.Ucs03FromHex)(encoded)
        // console.log({ decodedInstruction })
      })))
})
