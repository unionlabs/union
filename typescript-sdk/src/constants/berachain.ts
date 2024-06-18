import { defineChain } from "viem"

/**
 * create custom viem chain for berachain since 80084 is not supported yet
 */
export const berachainTestnetV2 = defineChain({
  id: 8_0084,
  testnet: true,
  name: "Berachain v2",
  rpcUrls: {
    default: {
      http: [
        "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
      ]
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
