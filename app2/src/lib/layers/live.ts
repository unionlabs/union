import { ENV } from "$lib/constants"
import { GraphQL } from "$lib/graphql/service"
import * as Datadog from "$lib/logging/datadog"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
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
  PriceOracle.Test, // TODO: replace with live service
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
