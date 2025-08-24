import { ENV } from "$lib/constants"
import { SupabaseClient } from "$lib/dashboard/client"
import { GasPriceMap } from "$lib/gasprice"
import { GraphQL } from "$lib/graphql/service"
import * as Datadog from "$lib/logging/datadog"
import { PriceOracleExecutor } from "@unionlabs/sdk/PriceOracle"
import { Layer, Logger, LogLevel, Match } from "effect"

const minimumLogLevel = Logger.minimumLogLevel(
  Match.value(ENV()).pipe(
    Match.when("DEVELOPMENT", () => LogLevel.Trace),
    Match.when("STAGING", () => LogLevel.Debug),
    Match.when("PRODUCTION", () => LogLevel.Info),
    Match.exhaustive,
  ),
)

export default Layer.mergeAll(
  GraphQL.Default,
  GasPriceMap.Default,
  PriceOracleExecutor.Default,
  SupabaseClient.Default({ auth: { autoRefreshToken: true } }),
  Logger.replace(
    Logger.defaultLogger,
    Logger.zip(
      Logger.prettyLogger({
        colors: true,
        mode: "browser",
        stderr: true,
      }),
      Datadog.Logger,
    ),
  ),
  minimumLogLevel,
)
