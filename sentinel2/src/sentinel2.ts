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
import { Config, loadConfig } from "./helpers.js"
import type { Hex } from "./helpers.js"

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



export let db: BetterSqlite3Database



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
