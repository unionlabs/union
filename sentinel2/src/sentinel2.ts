import { Cause, Effect, pipe } from "effect"

import Database from "better-sqlite3"
import type { Database as BetterSqlite3Database } from "better-sqlite3"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import { checkBalances } from "./check-balances.js"
import { checkSSLCertificates } from "./check-ssl-certificates.js"
import { dbPrepeare } from "./db-queries.js"
import { escrowSupplyControlLoop } from "./escrow-supply-control-loop.js"
import { fundBabylonAccounts } from "./fund-babylon-accounts.js"
import { Config, loadConfig } from "./helpers.js"
import { runIbcChecksForever } from "./run-ibc-checks.js"

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

  yield* Effect.log("Loading config from", argv.config)
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
