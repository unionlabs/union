export const UNO = {
  COIN_TYPE: 118,
  NATIVE_DENOM: "muno",
  ADDRESS_PREFIX: "union",
  SYMBOL: "UNO"
} as const

export const URLS = {
  GRAPHQL: "https://introspect.unionlabs.workers.dev",
  UNION: {
    /**
     * TODO: add array of RPCs and pass to `viem`'s `fallback` array
     */
    RPC: "https://union-testnet-rpc.polkachu.com",
    REST: "https://union-testnet-api.polkachu.com"
  },
  SEPOLIA: {
    RPC: "https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a",
    REST: null
  }
} as const

export const CHAIN = {
  UNION: {
    ID: import.meta.env.VITE_UNION_CHAIN_ID || "union-testnet-6",
    NAME: import.meta.env.VITE_UNION_CHAIN_NAME || "union-testnet"
  }
} as const

export const CONTRACT = {
  UNION: {
    ADDRESS:
      import.meta.env.VITE_UCS01_UNION_ADDRESS ||
      "union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy",
    SOURCE_CHANNEL: import.meta.env.VITE_UCS01_UNION_SOURCE_CHANNEL || "channel-0"
  },
  SEPOLIA: {
    ADDRESS: import.meta.env.VITE_UCS01_EVM_ADDRESS || "0x7f7AC7d5a1a2bD54dBA53a22209C3f96699Ed63c",
    PORT_ID: import.meta.env.VITE_UCS01_SEPOLIA_PORT_ID || "ucs01-relay",
    SOURCE_CHANNEL: import.meta.env.VITE_UCS01_SEPOLIA_SOURCE_CHANNEL || "channel-0"
  }
} as const
