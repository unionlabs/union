export interface CosmosSdkRpcProvider {
  rpc: string
  rest: string
  grpc?: string
}

export const COSMOS_SDK_RPC_PROVIDERS = [
  {
    rpc: "https://union-testnet-rpc.polkachu.com",
    rest: "https://union-testnet-api.polkachu.com"
  },
  {
    rpc: "https://rpc.testnet.bonlulu.uno",
    rest: "https://api.testnet.bonlulu.uno"
  }
] as const satisfies Array<CosmosSdkRpcProvider>
