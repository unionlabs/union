import { ENV } from "$lib/constants"
import { SupabaseClient } from "$lib/dashboard/client"
import { GasPriceMap } from "$lib/gasprice"
import { GraphQL } from "$lib/graphql/service"
import * as Tracing from "$lib/logging/tracing"
import * as SvelteConfigProvider from "$lib/services/SvelteConfigProvider.js"
import { PriceOracle } from "@unionlabs/sdk"
import { Indexer } from "@unionlabs/sdk/Indexer"
import { Layer, Logger, LogLevel, Match } from "effect"

const minimumLogLevel = Logger.minimumLogLevel(
  Match.value(ENV()).pipe(
    Match.when("DEVELOPMENT", () => LogLevel.Trace),
    Match.when("STAGING", () => LogLevel.Debug),
    Match.when("PRODUCTION", () => LogLevel.Info),
    Match.exhaustive,
  ),
)

const IndexerLive = Layer.mergeAll(
  Indexer.Default,
  SvelteConfigProvider.StaticPublic,
)

export default Layer.mergeAll(
  GraphQL.Default,
  GasPriceMap.Default,
  PriceOracle.layerExecutor,
  SupabaseClient.Default({ auth: { autoRefreshToken: true } }),
  IndexerLive,
  Tracing.TracingLive,
  Logger.replace(
    Logger.defaultLogger,
    Logger.prettyLogger({
      colors: true,
      mode: "browser",
      stderr: true,
    }),
  ),
  minimumLogLevel,
)
