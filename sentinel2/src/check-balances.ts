import { Effect, Logger, pipe, Schedule } from "effect"
import fetch from "node-fetch"
import { clearSignerIncident, getSignerIncident, markSignerIncident } from "./db-queries.js"
import { Config, resolveIncident, triggerIncident } from "./helpers.js"
import { db } from "./sentinel2.js"

process.on("uncaughtException", err => {
  console.error("❌ Uncaught Exception:", err.stack || err)
})
process.on("unhandledRejection", (reason, promise) => {
  console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
})

interface PostRequestInput {
  url: string
  port?: number
  headers: Record<string, string>
  payload: unknown
}

interface PostRequestError {
  readonly _tag: "PostRequestError"
  readonly message: string
  readonly status?: number
}

export const safePostRequest = ({ url, port, headers, payload }: PostRequestInput) => {
  const fullUrl = port ? `${url}:${port}` : url

  return Effect.tryPromise({
    try: () =>
      fetch(fullUrl, {
        method: "POST",
        headers,
        body: JSON.stringify(payload),
      }).then(async response => {
        if (response.status === 200) {
          return await response.json()
        }
        const text = await response.text().catch(() => "")
        // biome-ignore lint/style/useThrowOnlyError: <explanation>
        throw {
          _tag: "PostRequestError",
          message: `Non-200 status: ${response.status} body: ${text}`,
          status: response.status,
        }
      }),
    catch: error =>
      ({
        _tag: "PostRequestError",
        message: error instanceof Error
          ? error.message
          : typeof error === "object"
          ? JSON.stringify(error)
          : String(error),
        status: (error as any)?.status,
      }) satisfies PostRequestError,
  })
}

export const checkBalances = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Spawning per-plugin balance checks…")
    const { config } = yield* Config
    const sbConfig = config.signerBalances

    for (const [url, ports] of Object.entries(sbConfig)) {
      for (const [portStr, plugins] of Object.entries(ports)) {
        const port = Number(portStr)

        const portKey = `${url}:${port}`
        const existingPortIncident = getSignerIncident(db, portKey)

        const [probeJson, durationMs] = yield* Effect.gen(function*($) {
          const start = Date.now()

          const maybeResp = yield* pipe(
            Effect.tryPromise<Response, Error>({
              try: () =>
                (fetch(`${url}:${port}`, {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                }) as unknown) as PromiseLike<Response>,
              catch: e => new Error(`RPC probe connection failed: ${e}`),
            }),
            Effect.catchAllCause((err) => {
              console.error(`SIGNER_BALANCE_PROBE_FAILED @ ${portKey}: ${String(err)}`)
              return Effect.succeed(undefined)
            }),
          )

          if (!maybeResp) {
            return [undefined, 0] as const
          }

          const text = yield* Effect.tryPromise<string, Error>({
            try: () => maybeResp.text(),
            catch: e => new Error(`RPC probe read failed: ${e}`),
          })
          const took = Date.now() - start

          let json: any = null
          try {
            json = JSON.parse(text)
          } catch {
            /* ignore parse error */
          }

          return [json, took] as const
        })

        if (!probeJson || typeof probeJson.error !== "object") {
          yield* Effect.logError(`SIGNER_BALANCE_PORT_DOWN @ ${portKey}`)
          if (!existingPortIncident) {
            // const inc = yield* triggerIncident(
            //   `SIGNER_BALANCE_PORT_DOWN @ ${portKey}`,
            //   `no RPC response from ${url}:${port}`,
            //   config.betterstack_api_key,
            //   config.trigger_betterstack,
            //   "SENTINEL@union.build",
            //   "SIGNER_BALANCE_PORT_DOWN",
            //   "Union",
            //   config.isLocal,
            // )
            // markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        const errMsg = String(probeJson.error.message)
        if (errMsg !== "Parse error") {
          yield* Effect.logError(`SIGNER_BALANCE_RPC_ERROR @ ${portKey}: ${errMsg}`)
          if (!existingPortIncident) {
            // const inc = yield* triggerIncident(
            //   `SIGNER_BALANCE_RPC_ERROR @ ${portKey}`,
            //   `unexpected RPC error: ${errMsg}`,
            //   config.betterstack_api_key,
            //   config.trigger_betterstack,
            //   "SENTINEL@union.build",
            //   "SIGNER_BALANCE_RPC_ERROR",
            //   "Union",
            //   config.isLocal,
            // )
            // markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        yield* Effect.log(`SIGNER_BALANCE_RPC_OK @ ${portKey}`)
        if (existingPortIncident) {
          // const resolved = yield* resolveIncident(
          //   existingPortIncident,
          //   config.betterstack_api_key,
          //   config.trigger_betterstack,
          //   config.isLocal,
          //   "Sentinel: RPC back online",
          // )
          // if (resolved) {
          //   clearSignerIncident(db, portKey)
          // }
        }

        for (const [plugin, expectedThreshold] of Object.entries(plugins)) {
          const payload = [
            {
              jsonrpc: "2.0",
              id: 1,
              method: "voyager_pluginCustom",
              params: [plugin, "signerBalances", []] as const,
            },
          ]

          const callWithRetry = safePostRequest({
            url,
            port,
            headers: { "Content-Type": "application/json" },
            payload,
          })

          const worker = Effect.gen(function*(_) {
            const result = yield* callWithRetry
            if (result) {
              if (!Array.isArray(result) || result.length === 0) {
                yield* Effect.logError(
                  `Unexpected response shape for ${plugin} @ ${url}:${port}. Result: ${result}`,
                )
                return
              }

              const rpcObj = (result[0] as any).result
              if (typeof rpcObj !== "object" || rpcObj === null) {
                yield* Effect.logError(
                  `No 'result' object for ${plugin} @ ${url}:${port}. Result: ${
                    JSON.stringify(result)
                  }`,
                )
                return
              }

              for (const [wallet, balStr] of Object.entries(rpcObj)) {
                let bal = BigInt(balStr as string)

                const tags = {
                  plugin,
                  url,
                  port: portStr,
                  wallet,
                  balance: bal.toString(),
                  expected: expectedThreshold.toString(),
                }

                const key = `${url}:${port}:${plugin}:${wallet}`
                const existing = getSignerIncident(db, key)

                if (bal < expectedThreshold) {
                  const logEffect = Effect.annotateLogs(tags)(Effect.logError("SIGNER_BALANCE_LOW"))
                  Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                  if (!existing) {
                    const inc = yield* triggerIncident(
                      `SIGNER_BALANCE_LOW @ ${key}`,
                      JSON.stringify({
                        plugin,
                        url,
                        port: portStr,
                        wallet,
                        balance: bal.toString(),
                      }),
                      config.betterstack_api_key,
                      config.trigger_betterstack,
                      "SENTINEL@union.build",
                      "SIGNER_BALANCE_LOW",
                      "Union",
                      config.isLocal,
                    )
                    if (inc.data.id) {
                      markSignerIncident(db, key, inc.data.id)
                    }
                  }
                } else {
                  const logEffect = Effect.annotateLogs(tags)(Effect.logInfo("SIGNER_BALANCE_OK"))
                  Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                  if (existing) {
                    const didResolve = yield* resolveIncident(
                      existing,
                      config.betterstack_api_key,
                      config.trigger_betterstack,
                      config.isLocal,
                      "Sentinel-Automatically resolved.",
                    )
                    if (didResolve) {
                      clearSignerIncident(db, key)
                    }
                  }
                }
              }
            }
          })
          Effect.runFork(worker)
        }
      }
    }
  }),
  Schedule.spaced("30 minutes"),
)
