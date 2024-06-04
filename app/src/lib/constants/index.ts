import { sepolia } from "viem/chains"

export const UNO = {
  COIN_TYPE: 118,
  NATIVE_DENOM: "muno",
  ADDRESS_PREFIX: "union",
  SYMBOL: "UNO"
} as const

export const URLS = {
  GRAPHQL: "https://graphql.union.build/v1/graphql",
  GRAPHQL_WSS: "wss://noble-pika-27.hasura.app/v1/graphql",
  GRAPHQL_REST: "https://graphql.union.build/api/rest",
  UNION: {
    /**
     * TODO: add array of RPCs and pass to `viem`'s `fallback` array
     */
    RPC: "https://rpc.testnet.bonlulu.uno",
    // REST: "https://api.testnet.bonlulu.uno"
    REST: "https://union-testnet-api.polkachu.com"
  },
  SEPOLIA: {
    RPC: "https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a",
    REST: null
  }
} as const

export const CHAINS = ["SEPOLIA", "UNION"] as const
export type Chain = (typeof CHAINS)[number]

export const CHAIN = {
  UNION: {
    ID: "union-testnet-8",
    NAME: "Union Testnet"
  },
  SEPOLIA: {
    ID: sepolia.id.toString(),
    NAME: sepolia.name
  }
} satisfies Record<Chain, { ID: string; NAME: string }>

export const CHAIN_URLS = {
    [CHAIN.UNION.ID]: URLS.UNION
}
