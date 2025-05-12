import { ENV } from "$lib/constants"
import * as Datadog from "$lib/logging/datadog"
import { Context, Layer, Logger, LogLevel, Match } from "effect"

const A = Context.GenericTag<"A">("A")

const minimumLogLevel = Logger.minimumLogLevel(
  Match.value(ENV()).pipe(
    Match.when("DEVELOPMENT", () => LogLevel.Trace),
    Match.when("STAGING", () => LogLevel.Debug),
    Match.when("PRODUCTION", () => LogLevel.Warning),
    Match.exhaustive,
  ),
)

export default Layer.mergeAll(
  Layer.succeed(A, "A"),
  Logger.replace(
    Logger.defaultLogger,
    // Logger.zip(
    Logger.prettyLogger({
      colors: true,
      mode: "browser",
      stderr: true,
    }),
    // Datadog.Logger,
    // ),
  ),
  minimumLogLevel,
  // Datadog.Resource,
)
