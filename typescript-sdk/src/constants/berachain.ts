import { defineChain } from "viem"

/**
 * create custom viem chain for berachain since 80084 is not supported yet
 */
export const berachainTestnetV2 = defineChain({
  id: 8_0084,
  testnet: true,
  name: "Berachain bArtio",
  rpcUrls: {
    default: {
      http: ["https://bartio.rpc.berachain.com/"]
    }
  },
  nativeCurrency: {
    name: "BERA",
    symbol: "BERA",
    decimals: 18
  },
  blockExplorers: {
    default: { name: "routescan berachain", url: "https://testnet.routescan.io" }
  }
})
