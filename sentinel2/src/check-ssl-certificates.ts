import { Effect, Schedule } from "effect"
import { clearSslIncident, getSslIncident, markSslIncident } from "./db-queries.js"
import { getCertExpiry } from "./helpers.js"
import { Config, resolveIncident, triggerIncident } from "./helpers.js"
import { db } from "./sentinel2.js"

process.on("uncaughtException", err => {
  console.error("❌ Uncaught Exception:", err.stack || err)
})
process.on("unhandledRejection", (reason, promise) => {
  console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
})

export const safeGetRequest = ({
  url,
  port,
  headers = {} as Record<string, string>,
}: {
  url: string
  port?: number
  headers?: Record<string, string>
}) =>
  Effect.tryPromise({
    try: async () => {
      const fullUrl = port ? `${url}:${port}` : url
      const res = await fetch(fullUrl, { method: "GET", headers }) // headers is now always a Record<string,string>
      const text = await res.text()
      if (!res.ok) {
        throw { _tag: "GetRequestError", message: `GET ${res.status}`, status: res.status }
      }
      return text
    },
    catch: error =>
      ({
        _tag: "GetRequestError",
        message: error instanceof Error ? error.message : String(error),
        status: (error as any)?.status,
      }) as GetRequestError,
  })

interface GetRequestError {
  readonly _tag: "GetRequestError"
  readonly message: string
  readonly status?: number
}

export const checkSSLCertificates = Effect.repeat(
  Effect.gen(function*() {
    yield* Effect.log("Spawning checkSSLCertificates loop")
    const { config } = yield* Config
    const rpchostEndpoints = config.rpcHostEndpoints

    for (const rpchostEndpoint of rpchostEndpoints) {
      const pageHtml: string = yield* safeGetRequest({
        url: rpchostEndpoint,
        port: 443,
        headers: {},
      }).pipe(Effect.retry(Schedule.spaced("2 minutes")))
      const endpointAnchorRegex = /<a\s+href="([^"]+)">https?:\/\/[^<]+<\/a>/gi
      const links: string[] = []
      let m: RegExpExecArray | null
      while ((m = endpointAnchorRegex.exec(pageHtml))) {
        const href = m[1]
        if (href) {
          links.push(href)
        }
      }
      const uniqueEndpoints = Array.from(new Set(links))
      yield* Effect.log(`Found ${uniqueEndpoints.length} endpoints}`)

      const now = Date.now()
      const fourDaysMs = 4 * 24 * 60 * 60 * 1000
      for (const url of uniqueEndpoints) {
        const existingIncident = getSslIncident(db, url)
        const expiry: Date = yield* Effect.tryPromise({
          try: () => getCertExpiry(url),
          catch: e => new Error(`SSL check failed for ${url}: ${String(e)}`),
        })

        const msLeft = expiry.getTime() - now
        const description = JSON.stringify({ endpoint: url, expiresAt: expiry.toISOString() })

        if (msLeft <= fourDaysMs) {
          if (!existingIncident) {
            const inc = yield* triggerIncident(
              `SSL expiring soon @ ${url}`,
              description,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SSL_CERT_EXPIRY",
              "Union",
              config.isLocal,
            )
            if (inc.data.id) {
              markSslIncident(db, url, inc.data.id)
            }
          }
          yield* Effect.logError(`SSL expiring in ${(msLeft / 86400000).toFixed(1)} day. @ ${url}`)
        } else {
          if (existingIncident) {
            const didResolve = yield* resolveIncident(
              existingIncident,
              config.betterstack_api_key,
              config.trigger_betterstack,
              config.isLocal,
              "Sentinel: SSL renewed",
            )
            if (didResolve) {
              clearSslIncident(db, url)
            }
          }
          yield* Effect.log(`SSL ok @ ${url}, expires in ${(msLeft / 86400000).toFixed(1)} days`)
        }
      }
    }
  }),
  Schedule.spaced("6 hours"),
)
