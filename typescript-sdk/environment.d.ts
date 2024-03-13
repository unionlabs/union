interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly CLIENT_MODE: "browser" | "node"
  readonly SEPOLIA_RPC_URL: string
  readonly CHAIN_ID: "1" | "11155111" | "6"
  readonly ANVIL_RPC_URL: string
  readonly ANVIL_ACCOUNT_PRIVATE_KEY: `0x${string}`

  readonly UNION_RPC_URL: string
  readonly UNION_REST_URL: string
  readonly UNION_GRAPHQL_API: string
  readonly UNION_CHAIN_ID: string
  readonly MUNO_ERC20_ADDRESS: `0x${string}`
  readonly UCS01_EVM_ADDRESS: `0x${string}`
  readonly UCS01_UNION_ADDRESS: string
  readonly UCS01_UNION_SOURCE_CHANNEL: string
  readonly UCS01_SEPOLIA_SOURCE_CHANNEL: string
  readonly UCS01_SEPOLIA_PORT_ID: string

  readonly ADDRESS_PREFIX: string
  readonly UNION_COIN_TYPE: string
  readonly UNION_NATIVE_DENOM: string
}

declare module "bun" {
  interface Env extends EnvironmentVariables {}
}

declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
