import type { RequestHandler } from "./$types"
import { getChain, DEFAULT_CHAIN } from "$lib/chains/config"

const FETCH_TIMEOUT = 15_000

// Race all RPC endpoints, return first successful response
async function raceEndpoints(
  endpoints: string[],
  path: string,
  queryString: string
): Promise<{ data: unknown } | { error: string }> {
  const controllers = endpoints.map(() => new AbortController())

  const fetchPromises = endpoints.map(async (baseUrl, index) => {
    const targetUrl = `${baseUrl}/${path}${queryString}`
    const controller = controllers[index]

    const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

    try {
      const response = await fetch(targetUrl, {
        headers: { Accept: "application/json" },
        signal: controller.signal,
      })

      clearTimeout(timeoutId)

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      const data = await response.json()
      // Abort other requests since we got a success
      controllers.forEach((c, i) => i !== index && c.abort())
      return { success: true as const, data }
    } catch (error) {
      clearTimeout(timeoutId)
      const message = error instanceof Error ? error.message : String(error)
      return { success: false as const, error: `${baseUrl}: ${message}` }
    }
  })

  const results = await Promise.all(fetchPromises)
  const success = results.find((r) => r.success)
  if (success && success.success) {
    return { data: success.data }
  }

  const errors = results.filter((r) => !r.success).map((r) => (r as { error: string }).error)
  return { error: errors.join("; ") }
}

export const GET: RequestHandler = async ({ params, url, request }) => {
  const path = params.path
  const queryString = url.search

  // Get chain from header or default
  const chainName = request.headers.get("x-chain") || DEFAULT_CHAIN
  const chain = getChain(chainName) ?? getChain(DEFAULT_CHAIN)!
  const endpoints = chain.rpc

  if (endpoints.length === 0) {
    return new Response(JSON.stringify({ error: "No RPC endpoints configured" }), {
      status: 503,
      headers: { "Content-Type": "application/json" },
    })
  }

  const result = await raceEndpoints(endpoints, path, queryString)

  if ("data" in result) {
    return new Response(JSON.stringify(result.data), {
      headers: {
        "Content-Type": "application/json",
        "Cache-Control": "public, max-age=2",
      },
    })
  }

  return new Response(JSON.stringify({ error: `All RPC endpoints failed: ${result.error}` }), {
    status: 503,
    headers: { "Content-Type": "application/json" },
  })
}
