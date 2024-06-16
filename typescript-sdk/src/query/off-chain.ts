export interface Rpc {
  enabled: boolean
  url: string
  type: "rpc" | "rest" | "grpc"
}

export interface Chain {
  id: number
  chain_id: string
  rpc_type: "evm" | "cosmos"
  addr_prefix: string
  display_name: string
  enabled: boolean
  logo_uri: string | null
  rpcs: Array<Rpc>
}

const HUBBLE_REST_URL = "https://noble-pika-27.hasura.app/api/rest"

const queryHeaders = new Headers({
  Accept: "application/json",
  "Content-Type": "application/json",
  "User-Agent": "typescript-sdk"
})

export const offchainQuery = {
  chains: async ({
    includeEndpoints = false
  }: { includeEndpoints?: boolean } = {}): Promise<Array<Chain>> => {
    const response = await fetch(`${HUBBLE_REST_URL}/chains?include_rpcs=${includeEndpoints}`, {
      method: "GET",
      headers: queryHeaders
    })
    return response.json() as Promise<Array<Chain>>
  },
  chain: async ({
    chainId,
    includeEndpoints = false
  }: { chainId: string; includeEndpoints?: boolean }): Promise<Chain> => {
    const response = await fetch(
      `${HUBBLE_REST_URL}/chains/${chainId}?include_rpcs=${includeEndpoints}`,
      { method: "GET", headers: queryHeaders }
    )
    return response.json() as Promise<Chain>
  }
}
