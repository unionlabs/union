import { createQuery } from "@tanstack/svelte-query"

export function blockHeightQuery() {
  return createQuery({
    queryKey: ["block-height", "union"],
    queryFn: async () => {
      const response = await fetch("https://rpc.testnet.bonlulu.uno/block")
      const data = (await response.json()) as {
        jsonrpc: string
        id: number
        result: { block: { header: { height: string } } }
      }
      console.log(data)
      return data.result.block.header.height
    },
    enabled: true
  })
}
