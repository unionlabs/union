export const UNO = {
  COIN_TYPE: 118,
  NATIVE_DENOM: 'muno',
  ADDRESS_PREFIX: 'union',
  SYMBOL: 'UNO'
} as const

export const URLS = {
  UNION: {
    /**
     * TODO: add array of RPCs and pass to `viem`'s `fallback` array
     */
    RPC: 'https://union-testnet-rpc.polkachu.com',
    REST: import.meta.env.VITE_UNION_REST_URL,
    GRAPHQL: import.meta.env.VITE_UNION_GRAPHQL_URL
  },
  SEPOLIA: {
    RPC: import.meta.env.VITE_UCS01_EVM_ADDRESS
  }
} as const

export const CHAIN = {
  UNION: {
    ID: import.meta.env.VITE_UNION_CHAIN_ID || 'union-testnet-6',
    NAME: import.meta.env.VITE_UNION_CHAIN_NAME || 'union-testnet'
  }
} as const

export const CONTRACT = {
  UNION: {
    ADDRESS: import.meta.env.VITE_UCS01_UNION_ADDRESS,
    SOURCE_CHANNEL: import.meta.env.VITE_UCS01_UNION_SOURCE_CHANNEL
  },
  SEPOLIA: {
    ADDRESS: import.meta.env.VITE_UCS01_EVM_ADDRESS,
    PORT_ID: import.meta.env.VITE_UCS01_SEPOLIA_PORT_ID,
    SOURCE_CHANNEL: import.meta.env.VITE_UCS01_SEPOLIA_SOURCE_CHANNEL
  }
} as const
