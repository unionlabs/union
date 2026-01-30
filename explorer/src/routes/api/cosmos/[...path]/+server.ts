import type { RequestHandler } from "./$types"
import { getChain, DEFAULT_CHAIN } from "$lib/chains/config"

const FETCH_TIMEOUT = 15_000 // 15 second timeout
const INDEXER_URL = process.env.INDEXER_URL || "http://localhost:3002"

export const GET: RequestHandler = async ({ params, url, request }) => {
  const path = params.path
  const queryString = url.search

  // Get chain from header or default
  const chainId = request.headers.get("x-chain") || DEFAULT_CHAIN
  const chain = getChain(chainId)

  // Try indexer first
  const indexerUrl = `${INDEXER_URL}/${chainId}/rest/${path}${queryString}`

  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

  try {
    const response = await fetch(indexerUrl, {
      headers: { Accept: "application/json" },
      signal: controller.signal,
    })

    clearTimeout(timeoutId)

    if (!response.ok) {
      const body = await response.text()
      // If indexer fails, try direct chain API as fallback
      if (chain && chain.api.length > 0) {
        return await fetchDirect(chain.api[0], path, queryString)
      }
      return new Response(JSON.stringify({ error: `Indexer error: ${response.status} ${body}` }), {
        status: response.status,
        headers: { "Content-Type": "application/json" },
      })
    }

    const data = await response.json()
    return new Response(JSON.stringify(data), {
      headers: {
        "Content-Type": "application/json",
        "Cache-Control": "public, max-age=5",
      },
    })
  } catch (error) {
    clearTimeout(timeoutId)
    // Indexer unreachable - try direct chain API as fallback
    if (chain && chain.api.length > 0) {
      try {
        return await fetchDirect(chain.api[0], path, queryString)
      } catch (directError) {
        const message = directError instanceof Error ? directError.message : String(directError)
        return new Response(JSON.stringify({ error: `Both indexer and chain API failed: ${message}` }), {
          status: 503,
          headers: { "Content-Type": "application/json" },
        })
      }
    }
    const message = error instanceof Error ? error.message : String(error)
    return new Response(JSON.stringify({ error: `Indexer request failed: ${message}` }), {
      status: 503,
      headers: { "Content-Type": "application/json" },
    })
  }
}

// Direct fetch to chain API (fallback when indexer is unavailable)
async function fetchDirect(baseUrl: string, path: string, queryString: string): Promise<Response> {
  const url = `${baseUrl}/${path}${queryString}`
  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

  try {
    const response = await fetch(url, {
      headers: { Accept: "application/json" },
      signal: controller.signal,
    })

    clearTimeout(timeoutId)

    if (!response.ok) {
      const body = await response.text()
      return new Response(JSON.stringify({ error: `Chain API error: ${response.status} ${body}` }), {
        status: response.status,
        headers: { "Content-Type": "application/json" },
      })
    }

    const data = await response.json()
    return new Response(JSON.stringify(data), {
      headers: {
        "Content-Type": "application/json",
        "Cache-Control": "public, max-age=5",
      },
    })
  } finally {
    clearTimeout(timeoutId)
  }
}
