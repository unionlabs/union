import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { coins } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { FetchHttpClient } from "@effect/platform"
import {
  channelBalanceAtBlock as EthereumChannelBalanceAtBlock,
  EvmChannelDestination,
  readErc20BalanceAtBlock,
  readErc20TotalSupplyAtBlock,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
} from "@unionlabs/sdk/evm"
import { Context, Data, Effect, Logger, Schedule } from "effect"
import { pipe } from "effect"
import * as Cause from "effect/Cause"
import tls from "node:tls"
import { createPublicClient, http } from "viem"

import {
  channelBalanceAtHeight,
  channelBalanceAtHeight as CosmosChannelBalanceAtHeight,
  CosmosChannelDestination,
  CosmWasmClientContext,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  getBalanceAtHeight,
  getChainHeight,
  readCw20BalanceAtHeight,
  readCw20TokenInfo,
  readCw20TotalSupplyAtHeight,
} from "@unionlabs/sdk/cosmos"

import Database from "better-sqlite3"
import type { Database as BetterSqlite3Database } from "better-sqlite3"
import { gql, request } from "graphql-request"
import fetch from "node-fetch"
import fs from "node:fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import { runIbcChecksForever } from "./run_ibc_checks.js"
import { escrowSupplyControlLoop } from "./escrow_supply_control_loop.js"
import { dbPrepeare } from "./db_queries.js"

process.on("uncaughtException", err => {
  console.error("❌ Uncaught Exception:", err.stack || err)
})
process.on("unhandledRejection", (reason, promise) => {
  console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
})

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}



/**
 * Checks whether the TLS cert for `host` is valid and
 * doesn’t expire within 1 week.
 *
 * @param host  hostname (e.g. "example.com")
 * @returns     Effect<never, Error, boolean>
 */
export const checkSslCertificate = (host: string, isExpiringInDays: number) =>
  Effect.tryPromise<boolean, Error>({
    try: () =>
      new Promise((resolve, reject) => {
        // connect on port 443, SNI = host
        const socket = tls.connect(443, host, { servername: host }, () => {
          const cert = socket.getPeerCertificate()
          socket.end()

          if (!cert || typeof cert.valid_to !== "string") {
            return reject(new Error("No certificate retrieved"))
          }

          const expiry = new Date(cert.valid_to)
          const oneWeekFromNow = Date.now() + isExpiringInDays * 24 * 60 * 60 * 1_000

          resolve(expiry.getTime() > oneWeekFromNow)
        })

        socket.on("error", reject)
      }),
    catch: e => new Error(`SSL certificate check failed: ${e}`),
  })
// helper to pull the cert expiry date out of a host:port
function getCertExpiry(endpoint: string): Promise<Date> {
  const { hostname, port } = new URL(endpoint)
  const portNum = port ? Number(port) : 443
  return new Promise((resolve, reject) => {
    const socket = tls.connect({ host: hostname, port: portNum, servername: hostname }, () => {
      const cert = socket.getPeerCertificate()
      socket.end()
      if (!cert || !cert.valid_to) {
        return reject(new Error(`no valid_to on cert for ${endpoint}`))
      }
      resolve(new Date(cert.valid_to))
    })
    socket.on("error", reject)
  })
}

type Hex = `0x${string}`
export let db: BetterSqlite3Database

/**
 * Effect to trigger a BetterStack incident via the Uptime API
 */
export const triggerIncident = (
  summary: string,
  description: string,
  apiKey: string,
  trigger_betterstack: boolean,
  requesterEmail: string,
  incidentName: string,
  teamName: string,
  isLocal: boolean,
) => {
  const remote = Effect.tryPromise<{ data: { id: string } }, Error>({
    try: () =>
      fetch("https://uptime.betterstack.com/api/v3/incidents", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          summary,
          description,
          requester_email: requesterEmail,
          ...(teamName ? { team_name: teamName } : {}),
          call: false,
          sms: false,
          email: false,
          name: incidentName,
        }),
      }).then(async res => {
        const text = await res.text()
        if (!res.ok) {
          throw new Error(`Trigger failed: ${text}`)
        }
        return JSON.parse(text)
      }),
    catch: e => new Error(`Incident trigger error: ${e}`),
  })
    .pipe(
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("⚠️ triggerIncident failed:", cause)
          return { data: { id: "" } }
        })
      ),
    )

  if (isLocal) {
    return Effect.sync(() => {
      console.info("Local mode: skipping triggerIncident")
      return { data: { id: "" } }
    })
  }
  if (!trigger_betterstack) {
    return Effect.sync(() => {
      return { data: { id: "" } }
    })
  }
  return remote
}

/**
 * Effect to resolve an existing BetterStack incident via the Uptime API
 */
export const resolveIncident = (
  incidentId: string,
  apiKey: string,
  trigger_betterstack: boolean,
  isLocal: boolean,
  resolvedBy = "SENTINEL@union.build",
) => {
  if (!trigger_betterstack) {
    return Effect.sync(() => {
      return false
    })
  }
  if (isLocal) {
    return Effect.sync(() => {
      console.info("Local mode: skipping resolveIncident")
      return true
    })
  }

  return Effect.tryPromise<unknown, Error>({
    try: () =>
      fetch(`https://uptime.betterstack.com/api/v3/incidents/${incidentId}/resolve`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${apiKey}`,
        },
        body: JSON.stringify({ resolved_by: resolvedBy }),
      }).then(async res => {
        const text = await res.text()
        if (!res.ok) {
          throw new Error(`Resolve failed: ${text}`)
        }
        return JSON.parse(text)
      }),
    catch: e => new Error(`Incident resolve error: ${e}`),
  }).pipe(
    // if we parse successfully we consider it resolved
    Effect.map(() => true),
    // swallow any error and return `false`
    Effect.catchAllCause(err =>
      Effect.sync(() => {
        const message = String(err)
        if (message.includes("Incident was already resolved")) {
          console.info("Incident was already resolved, treating as success.")
          return true
        }

        console.error("⚠️ resolveIncident failed:", err)
        return false
      })
    ),
  )
}


function hexToUtf8(hex: string): string {
  // strip optional 0x
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex
  // build a Buffer from hex, then decode as UTF‑8
  return Buffer.from(clean, "hex").toString("utf8")
}

// Chain pair configuration
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
  enabled: boolean
}

export interface WrappedToken {
  chain: { universal_chain_id: string }
  denom: Hex
  wrapping: Array<{
    unwrapped_chain: { universal_chain_id: string }
    destination_channel_id: number
    unwrapped_denom: string
  }>
}

interface FundableAccounts {
  receiver_display: string
  traces: Array<{
    type: string
    transaction_hash: string
  }>
}

interface V2Channels {
  source_channel_id: string
}



export interface Packet {
  source_chain: {
    universal_chain_id: string
  }
  destination_chain: {
    universal_chain_id: string
  }
  packet_send_timestamp: string
  packet_hash: string
  status: string
  sort_order: string
}

type ChainType = "evm" | "cosmos"

interface ChainConfigEntry {
  zkgmAddress: string
  rpc: string
  restUrl: string
  chainType: ChainType
  minter: string
}

type ChainConfig = Record<string, ChainConfigEntry>

export interface SignerBalanceThresholds {
  [plugin: string]: bigint
}

export type PortSignerBalances = Record<string, SignerBalanceThresholds>

export type SignerBalancesConfig = Record<string, PortSignerBalances>

// Combined configuration shape
interface ConfigFile {
  cycleIntervalMs: number
  hasuraEndpoint: string
  rpcHostEndpoints: string[]
  signerBalances: SignerBalancesConfig
  chainConfig: ChainConfig
  signer_account_mnemonic: string
  betterstack_api_key: string
  trigger_betterstack: boolean
  dbPath: string
  isLocal: boolean
}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}


const fetchFundableAccounts = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
    const query = gql`
      query {
        v2_transfers(args: { p_destination_universal_chain_id: "babylon.bbn-1" }) {
          receiver_display
          traces {
            type
            transaction_hash
          }
        }
      }
    `

    const response: any = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, query),
      catch: error => {
        console.error("fetchFundableAccounts failed:", error)
        throw error
      },
    })

    const tokens: Array<FundableAccounts> = response?.v2_transfers || []
    const filtered: Array<FundableAccounts> = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(
            trace =>
              trace.type === "WRITE_ACK"
              && trace.transaction_hash != null
              && !isFunded(db, trace.transaction_hash),
          )
          // biome-ignore lint/style/noNonNullAssertion: <explanation>
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! })),
      }))
      .filter(acc => acc.traces.length > 0)

    return filtered
  })


function loadConfig(configPath: string) {
  return Effect.tryPromise({
    // biome-ignore lint/suspicious/useAwait: <explanation>
    try: async () => {
      if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.")
      }
      const rawData = fs.readFileSync(configPath, "utf-8")
      const config: ConfigFile = JSON.parse(rawData)

      return config
    },
    catch: error =>
      new FilesystemError({
        message: "Config file is invalid.",
        cause: error,
      }),
  })
}
const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config
    if (config.isLocal) {
      yield* Effect.log("Local mode: skipping funding babylon accounts")
      return
    }

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn"),
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options,
    )

    if (!senderAccount?.address) {
      yield* Effect.logError("Sender account couldnt found!")
      return
    }
    const balance = yield* Effect.tryPromise(() => client.getBalance(senderAccount.address, "ubbn"))

    if (Number.parseInt(balance.amount) < 1_000_000) {
      const errLog = Effect.annotateLogs({
        issueType: "SPENDER_BALANCE_LOW",
        balance: balance.amount,
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
      })(Effect.logError("SPENDER_BALANCE_LOW"))

      Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
      return
    }

    const fee = {
      amount: coins(500, "ubbn"),
      gas: "200000",
    }

    const accs = yield* fetchFundableAccounts(config.hasuraEndpoint)
    for (const acc of accs) {
      const receiver = acc.receiver_display
      const result = yield* Effect.tryPromise({
        try: () =>
          client.sendTokens(
            senderAccount.address,
            receiver,
            coins(10000, "ubbn"), // send 0.01 bbn
            fee,
          ),
        catch: err => {
          console.error("raw sendTokens error:", err)
          throw err
        },
      })

      addFunded(db, result.transactionHash)

      const okLog = Effect.annotateLogs({
        sentAmount: "0.01",
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
        receiver,
        transactionHash: result.transactionHash,
      })(Effect.logInfo("SENT_OK"))
      Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
    }
  }),
  Schedule.spaced("1 minutes"),
)

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
interface GetRequestError {
  readonly _tag: "GetRequestError"
  readonly message: string
  readonly status?: number
}

// 1) make headers always defined by giving it a default
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

export const checkSSLCertificates = Effect.repeat(
  Effect.gen(function*(_) {
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
          const resp = yield* Effect.tryPromise<Response, Error>({
            try: () =>
              (fetch(`${url}:${port}`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
              }) as unknown) as PromiseLike<Response>,
            catch: e => new Error(`RPC probe connection failed: ${e}`),
          })

          const text = yield* Effect.tryPromise<string, Error>({
            try: () => resp.text(),
            catch: e => new Error(`RPC probe read failed: ${e}`),
          })
          const took = Date.now() - start

          let json: any = null
          try {
            json = JSON.parse(text)
          } catch { /* leave json=null */ }

          return [json, took] as const
        })

        if (!probeJson || typeof probeJson.error !== "object") {
          yield* Effect.logError(`SIGNER_BALANCE_PORT_DOWN @ ${portKey}`)
          if (!existingPortIncident) {
            const inc = yield* triggerIncident(
              `SIGNER_BALANCE_PORT_DOWN @ ${portKey}`,
              `no RPC response from ${url}:${port}`,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SIGNER_BALANCE_PORT_DOWN",
              "Union",
              config.isLocal,
            )
            markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        const errMsg = String(probeJson.error.message)
        if (errMsg !== "Parse error") {
          yield* Effect.logError(`SIGNER_BALANCE_RPC_ERROR @ ${portKey}: ${errMsg}`)
          if (!existingPortIncident) {
            const inc = yield* triggerIncident(
              `SIGNER_BALANCE_RPC_ERROR @ ${portKey}`,
              `unexpected RPC error: ${errMsg}`,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SIGNER_BALANCE_RPC_ERROR",
              "Union",
              config.isLocal,
            )
            markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        yield* Effect.log(`SIGNER_BALANCE_RPC_OK @ ${portKey} in ${durationMs}ms`)
        if (existingPortIncident) {
          const resolved = yield* resolveIncident(
            existingPortIncident,
            config.betterstack_api_key,
            config.trigger_betterstack,
            config.isLocal,
            "Sentinel: RPC back online",
          )
          if (resolved) {
            clearSignerIncident(db, portKey)
          }
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

const mainEffect = Effect.gen(function*(_) {
  const argv = yield* Effect.sync(() =>
    yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file",
      })
      .help()
      .alias("help", "h")
      .parseSync()
  )

  console.info("Loading config from", argv.config)
  const config = yield* loadConfig(argv.config)

  try {
    db = new Database(config.dbPath)
  } catch (err) {
    console.error("Error opening database:", err)
    throw err
  }

  yield* dbPrepeare(db)

  yield* Effect.log("Database opened at", config.dbPath, "hasuraEndpoint:", config.hasuraEndpoint)

  yield* Effect.all(
    [
      runIbcChecksForever,
      escrowSupplyControlLoop,
      // fundBabylonAccounts,
      // checkBalances,
      // checkSSLCertificates,
    ],
    {
      concurrency: "unbounded",
    },
  ).pipe(Effect.provideService(Config, { config }))
})

pipe(
  mainEffect,
  Effect.catchAllCause(cause =>
    Effect.sync(() => {
      console.error("💥 mainEffect failed:\n", Cause.pretty(cause))
    })
  ),
  Effect.runPromise,
).catch(err => {
  console.error("🔥 runPromise threw:", err.stack || err)
})
