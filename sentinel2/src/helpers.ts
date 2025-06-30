import { Context, Data, Effect } from "effect"
import fetch from "node-fetch"
import fs from "node:fs"
import tls from "node:tls"

export type Hex = `0x${string}`

export interface SignerBalanceThresholds {
  [plugin: string]: bigint
}

export type PortSignerBalances = Record<string, SignerBalanceThresholds>

export type SignerBalancesConfig = Record<string, PortSignerBalances>

export interface WrappedToken {
  chain: { universal_chain_id: string }
  denom: Hex
  wrapping: Array<{
    unwrapped_chain: { universal_chain_id: string }
    destination_channel_id: number
    unwrapped_denom: string
  }>
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

export function loadConfig(configPath: string) {
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

export function hexToUtf8(hex: string): string {
  // strip optional 0x
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex
  // build a Buffer from hex, then decode as UTF‑8
  return Buffer.from(clean, "hex").toString("utf8")
}

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
  call: boolean = false,
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
          call: call,
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
