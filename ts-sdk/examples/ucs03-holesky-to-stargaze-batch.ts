import { Effect } from "effect"
import { ViemPublicClientSource } from "../src/evm/client.js"
import { createPublicClient, http } from "viem"
import { holesky } from "viem/chains"
import { CosmosDestinationConfig } from "../src/cosmos/quote-token.js"
import { createEvmToCosmosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import { CosmWasmClientDestination, createCosmWasmClient } from "../src/cosmos/client.js"
import { Instruction } from "../src/ucs03.js"

const createBatch = Effect.gen(function* () {
  const mainTransfer = yield* createEvmToCosmosFungibleAssetOrder({
    sender: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
    receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
    baseToken: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b", // LINK on holesky
    baseAmount: 100n,
    quoteAmount: 100n
  })

  const feeTransfer = yield* createEvmToCosmosFungibleAssetOrder({
    sender: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
    receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
    baseToken: "0x94373a4919B3240D86eA41593D5eBa789FEF3848", // WETH on holesky
    baseAmount: 50n,
    quoteAmount: 0n
  })

  return new Instruction.Batch({ operand: [mainTransfer, feeTransfer] })
})

Effect.runPromiseExit(
  Effect.gen(function* () {
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")

    const batch = yield* createBatch.pipe(
      Effect.provideService(ViemPublicClientSource, {
        client: createPublicClient({
          chain: holesky,
          transport: http()
        })
      }),
      Effect.provideService(CosmWasmClientDestination, { client }),
      Effect.provideService(CosmosDestinationConfig, {
        ucs03address: "stars1x2jzeup7uwfxjxxrtfna2ktcugltntgu6kvc0eeayk0d82l247cqsnqksg",
        channelId: 3
      })
    )

    console.log(batch)
    console.log(Instruction.encodeAbi(batch))
  })
)
