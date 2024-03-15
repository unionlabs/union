interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly PORT: string
  readonly VERSION: string // from package.json#version
  readonly COMMIT_SHA: string
  readonly VITE_UNION_RPC_URL: string
  readonly VITE_UNION_REST_URL: string
  readonly VITE_UNION_GRAPHQL_URL: string
  readonly VITE_UNION_CHAIN_ID: string
  readonly VITE_UNION_CHAIN_NAME: string
  readonly VITE_UCS01_EVM_ADDRESS: string
  readonly VITE_UCS01_UNION_ADDRESS: string
  readonly VITE_UCS01_SEPOLIA_PORT_ID: string
  readonly VITE_UCS01_UNION_SOURCE_CHANNEL: string
  readonly VITE_UCS01_SEPOLIA_SOURCE_CHANNEL: string
}
// Node.js environment variables types
declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
// Vite environment variables types
interface ImportMetaEnv extends EnvironmentVariables {}
interface ImportMeta {
  readonly env: ImportMetaEnv
}
// Cloudflare Pages/Workers
interface Env extends EnvironmentVariables {}
