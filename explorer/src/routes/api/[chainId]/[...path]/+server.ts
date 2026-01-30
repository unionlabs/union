import type { RequestHandler } from "./$types"
import { getChain, DEFAULT_CHAIN } from "$lib/chains/config"

const FETCH_TIMEOUT = 12_000

// Simple in-memory cache
const cache = new Map<string, { data: unknown; expires: number }>()

function getCached(key: string): unknown | null {
  const entry = cache.get(key)
  if (!entry) return null
  if (Date.now() > entry.expires) {
    cache.delete(key)
    return null
  }
  return entry.data
}

function setCache(key: string, data: unknown, ttlMs: number) {
  cache.set(key, { data, expires: Date.now() + ttlMs })
  // Basic cleanup - remove old entries periodically
  if (cache.size > 1000) {
    const now = Date.now()
    for (const [k, v] of cache) {
      if (now > v.expires) cache.delete(k)
    }
  }
}

// Race endpoints, return first success
async function raceEndpoints(endpoints: string[], path: string): Promise<unknown> {
  const controllers = endpoints.map(() => new AbortController())

  const promises = endpoints.map(async (base, i) => {
    const url = `${base}${path}`
    const controller = controllers[i]
    const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

    try {
      const res = await fetch(url, {
        headers: { Accept: "application/json" },
        signal: controller.signal,
      })
      clearTimeout(timeoutId)

      if (!res.ok) throw new Error(`HTTP ${res.status}`)

      const data = await res.json()
      // Cancel other requests
      controllers.forEach((c, j) => j !== i && c.abort())
      return data
    } catch (e) {
      clearTimeout(timeoutId)
      throw e
    }
  })

  return Promise.any(promises)
}

// Determine cache TTL based on path
function getCacheTTL(path: string): number {
  if (path.includes("/blocks/latest")) return 2_000      // 2s for latest block
  if (path.includes("/blockchain")) return 2_000          // 2s for block range
  if (path.includes("/blocks/")) return 60_000            // 1min for specific block
  if (path.includes("/tx_search")) return 3_000           // 3s for tx search
  if (path.includes("/txs/")) return 60_000               // 1min for specific tx
  if (path.includes("/validators")) return 30_000         // 30s for validators
  if (path.includes("/proposals")) return 30_000          // 30s for proposals
  if (path.includes("/params")) return 300_000            // 5min for params
  if (path.includes("/status")) return 5_000              // 5s for status
  return 10_000                                           // 10s default
}

// Determine if this is an RPC or REST request
function isRpcPath(path: string): boolean {
  const rpcPaths = ["/blockchain", "/block", "/tx_search", "/status", "/validators"]
  return rpcPaths.some(p => path.startsWith(p))
}

export const GET: RequestHandler = async ({ params, url }) => {
  const chainId = params.chainId
  const path = `/${params.path}${url.search}`

  const chain = getChain(chainId) ?? getChain(DEFAULT_CHAIN)
  if (!chain) {
    return new Response(JSON.stringify({ error: "Unknown chain" }), {
      status: 400,
      headers: { "Content-Type": "application/json" },
    })
  }

  // Check cache
  const cacheKey = `${chainId}:${path}`
  const cached = getCached(cacheKey)
  if (cached) {
    return new Response(JSON.stringify(cached), {
      headers: {
        "Content-Type": "application/json",
        "X-Cache": "HIT",
      },
    })
  }

  // Pick endpoints based on path type
  const endpoints = isRpcPath(path) ? chain.rpc : chain.api

  if (endpoints.length === 0) {
    return new Response(JSON.stringify({ error: "No endpoints configured" }), {
      status: 503,
      headers: { "Content-Type": "application/json" },
    })
  }

  try {
    const data = await raceEndpoints(endpoints, path)
    const ttl = getCacheTTL(path)
    setCache(cacheKey, data, ttl)

    return new Response(JSON.stringify(data), {
      headers: {
        "Content-Type": "application/json",
        "X-Cache": "MISS",
        "Cache-Control": `public, max-age=${Math.floor(ttl / 1000)}`,
      },
    })
  } catch (error) {
    const message = error instanceof AggregateError
      ? error.errors.map(e => e.message).join("; ")
      : error instanceof Error ? error.message : String(error)

    return new Response(JSON.stringify({ error: `All endpoints failed: ${message}` }), {
      status: 503,
      headers: { "Content-Type": "application/json" },
    })
  }
}
