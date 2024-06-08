import { createQuery } from "@tanstack/svelte-query"

export const blockHeightQuery = () =>
  createQuery({
    queryKey: ["block-height", "union"],
    queryFn: async () => {
      // const response = await fetch("https://rpc.testnet.bonlulu.uno/block")
      const response = await fetch("https://union-testnet-rpc.polkachu.com/block")
      const data = (await response.json()) as {
        jsonrpc: string
        id: number
        result: { block: { header: { height: string } } }
      }
      return data.result.block.header.height
    },
    enabled: true,
    // Union default block time
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
