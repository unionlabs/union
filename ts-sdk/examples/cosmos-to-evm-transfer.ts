import { Effect } from "effect"
import { ViemPublicClientDestination } from "../src/evm/client"
import { createPublicClient, http, parseEther } from "viem"
import { sepolia } from "viem/chains"
import { DestinationConfig } from "../src/evm/quote-token"
import { createCosmosToEvmFungibleAssetOrder } from "../src/evm/ucs03/fungible-asset-order"
import { CosmWasmClientContext, CosmWasmClientSource, createCosmWasmClient } from "../src/cosmos/client"
import { readCw20TokenInfo, readCw20Balance } from "../src/cosmos/cw20"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Cosmos to EVM transfer example
Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a CosmWasm client for the source chain (Stargaze testnet)
    const cosmosClient = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")
    
    // Create an EVM client for the destination chain (Sepolia)
    const evmClient = createPublicClient({
      chain: sepolia,
      transport: http()
    })
    
    // Define the token contract and addresses
    const cw20TokenAddress = "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr"
    const senderAddress = "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4"
    const receiverAddress = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    
    // Check the sender's balance before transfer
    const tokenInfo = yield* readCw20TokenInfo(cw20TokenAddress).pipe(
      Effect.provideService(CosmWasmClientContext, { client: cosmosClient })
    )
    
    const balance = yield* readCw20Balance(cw20TokenAddress, senderAddress).pipe(
      Effect.provideService(CosmWasmClientContext, { client: cosmosClient })
    )
    
    console.log(`Token: ${tokenInfo.name} (${tokenInfo.symbol})`)
    console.log(`Sender balance: ${balance} ${tokenInfo.symbol}`)
    
    // Create the fungible asset order
    const transferAmount = BigInt(1000000) // 1 token with 6 decimals
    const quoteAmount = parseEther("0.01") // Expected amount on destination chain
    
    const order = yield* createCosmosToEvmFungibleAssetOrder({
      sender: senderAddress,
      receiver: receiverAddress,
      baseToken: cw20TokenAddress,
      baseAmount: transferAmount,
      quoteAmount: quoteAmount
    }).pipe(
      Effect.provideService(CosmWasmClientSource, { client: cosmosClient }),
      Effect.provideService(ViemPublicClientDestination, { client: evmClient }),
      Effect.provideService(DestinationConfig, {
        ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
        channelId: 8
      })
    )
    
    return {
      tokenInfo,
      senderBalance: balance,
      order
    }
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
