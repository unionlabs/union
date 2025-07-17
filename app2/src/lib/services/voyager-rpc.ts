import { Effect, Option, Schema } from "effect"

const VOY_RUN_URL = "https://voy.run"

// Schema for JSON-RPC request
const JsonRpcRequest = Schema.Struct({
  jsonrpc: Schema.Literal("2.0"),
  method: Schema.String,
  params: Schema.Array(Schema.Unknown),
  id: Schema.Number
})

// Schema for JSON-RPC response
const JsonRpcResponse = Schema.Union(
  Schema.Struct({
    id: Schema.Number,
    jsonrpc: Schema.Literal("2.0"),
    result: Schema.String
  }),
  Schema.Struct({
    id: Schema.Number,
    jsonrpc: Schema.Literal("2.0"),
    error: Schema.Struct({
      code: Schema.Number,
      message: Schema.String
    })
  })
)

type JsonRpcResponse = Schema.Schema.Type<typeof JsonRpcResponse>

// Extract chain ID from universal chain ID (remove family prefix)
export function extractChainId(universalChainId: string): string {
  const parts = universalChainId.split(".")
  return parts.length > 1 ? parts[1] : universalChainId
}

// Fetch finalized heights for multiple chains
export const fetchFinalizedHeights = (universalChainIds: string[]) =>
  Effect.tryPromise({
    try: async () => {
      const requests = universalChainIds.map((ucid, index) => ({
        jsonrpc: "2.0" as const,
        method: "voyager_queryLatestHeight",
        params: [extractChainId(ucid), true],
        id: index + 1
      }))

      const response = await fetch(VOY_RUN_URL, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(requests)
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const json = await response.json()
      const decoded = Schema.decodeUnknownSync(Schema.Array(JsonRpcResponse))(json)

      // Map responses back to universal chain IDs
      const heightMap = new Map<string, Option.Option<string>>()
      
      decoded.forEach((res: JsonRpcResponse, index: number) => {
        const ucid = universalChainIds[index]
        if ("result" in res) {
          heightMap.set(ucid, Option.some(res.result))
        } else {
          heightMap.set(ucid, Option.none())
        }
      })

      return heightMap
    },
    catch: (error) => {
      console.error("Failed to fetch finalized heights:", error)
      // Return empty map on error
      return new Map<string, Option.Option<string>>()
    }
  })