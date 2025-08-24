import { PUBLIC_DATADOG_CLIENT_TOKEN, PUBLIC_GIT_REV } from "$env/static/public"
import { ENV, SERVICE_NAME } from "$lib/constants"
import { flattenObject } from "$lib/utils/flattenObject"
import { datadogLogs, StatusType } from "@datadog/browser-logs"
import { Array as A, Cause, HashMap, Logger, LogLevel, Match, pipe, Record as R } from "effect"

const statusType: (u: LogLevel.LogLevel) => StatusType = pipe(
  Match.type<LogLevel.LogLevel>(),
  Match.tagsExhaustive({
    All: () => "ok" as const,
    Debug: () => "debug" as const,
    Error: () => "error" as const,
    Fatal: () => "critical" as const,
    Info: () => "info" as const,
    None: () => "ok" as const,
    Trace: () => "debug" as const,
    Warning: () => "warn" as const,
  }),
)

const init = () => {
  if (ENV() === "DEVELOPMENT") {
    return
  }

  const config: Parameters<typeof datadogLogs.init>[0] = {
    clientToken: PUBLIC_DATADOG_CLIENT_TOKEN,
    site: "datadoghq.eu",
    forwardErrorsToLogs: false,
    service: SERVICE_NAME,
    env: ENV().toLowerCase(),
    sessionSampleRate: 100,
    version: PUBLIC_GIT_REV,
    telemetrySampleRate: 0,
  }

  if (!PUBLIC_DATADOG_CLIENT_TOKEN) {
    console.warn("[DD] Not configured.")
    return
  } else {
    console.info("[DD] Initializing...", config)
  }

  datadogLogs.init(config)
}

const DatadogLogger = Logger.make(
  (options) => {
    const annotations = HashMap.isEmpty(options.annotations)
      ? undefined
      : Object.fromEntries(HashMap.toEntries(options.annotations))
    // const pretty = pipe(
    //   Cause.prettyErrors(options.cause),
    //   (xs) => A.isNonEmptyArray(xs) ? xs : undefined,
    // )

    const message: string = String(options.message)
    const context = {
      ...(annotations && { annotations }),
      // ...(pretty && { pretty }),
    } satisfies object
    const status: StatusType = statusType(options.logLevel)
    const error: Error | undefined = Cause.isEmpty(options.cause)
      ? undefined
      : Cause.squash(options.cause) as Error

    const payload: Parameters<typeof datadogLogs.logger.log> = [
      message,
      context,
      status,
      error,
    ] as const

    datadogLogs.logger.log(...payload)
  },
)

export {
  DatadogLogger as Logger,
  /** @public */
  init as __init,
}
