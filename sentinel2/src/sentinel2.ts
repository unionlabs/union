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
import { fundBabylonAccounts } from "./fund_babylon_accounts.js"
import { checkBalances } from "./check_balances.js"
import { checkSSLCertificates } from "./check_ssl_certificates.js"
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
export function getCertExpiry(endpoint: string): Promise<Date> {
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
      fundBabylonAccounts,
      checkBalances,
      checkSSLCertificates,
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
