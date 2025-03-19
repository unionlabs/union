import { Effect } from "effect"
import { ViemPublicClientDestination, ViemPublicClientSource } from "../src/evm/client"
import { createPublicClient, http, parseEther, toHex } from "viem"
import { sepolia, holesky } from "viem/chains"
import { DestinationConfig } from "../src/evm/quote-token"
import { createFungibleAssetOrder } from "../src/evm/ucs03/fungible-asset-order"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

Effect.runPromiseExit(
  createFungibleAssetOrder({
    sender: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    receiver: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    baseToken: "0x94373a4919B3240D86eA41593D5eBa789FEF3848", // WETH on holesky
    baseAmount: parseEther("100"), // 100 tokens
    quoteAmount: parseEther("0.05") // 0.05 quote tokens
  }).pipe(
    Effect.provideService(ViemPublicClientSource, {
      client: createPublicClient({
        chain: holesky,
        transport: http()
      })
    }),
    Effect.provideService(ViemPublicClientDestination, {
      client: createPublicClient({
        chain: sepolia,
        transport: http()
      })
    }),
    Effect.provideService(DestinationConfig, {
      ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
      channelId: 8
    })
  )
).then(exit => console.log(JSON.stringify(exit, null, 2)))
