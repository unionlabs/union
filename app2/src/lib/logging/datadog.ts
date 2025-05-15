import { PUBLIC_DATADOG_CLIENT_TOKEN } from "$env/static/public"
import { ENV, SERVICE_NAME } from "$lib/constants"
import { datadogLogs, StatusType } from "@datadog/browser-logs"
import { Cause, Logger, LogLevel, Match, pipe } from "effect"

const statusType: (u: LogLevel.LogLevel) => StatusType = pipe(
  Match.type<LogLevel.LogLevel>(),
  Match.tagsExhaustive({
    All: () => "ok" as const,
    Debug: () => "debug" as const,
    Error: () => "error" as const,
    Fatal: () => "critical" as const,
    Info: () => "info" as const,
    None: () => "ok" as const,
    Trace: () => "notice" as const,
    Warning: () => "warn" as const,
  }),
)

export const init = () => {
  const config: Parameters<typeof datadogLogs.init>[0] = {
    clientToken: PUBLIC_DATADOG_CLIENT_TOKEN,
    site: "datadoghq.eu",
    forwardErrorsToLogs: false,
    service: SERVICE_NAME,
    env: ENV().toLowerCase(),
    sessionSampleRate: 100,
    version: "unknown",
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

const DatadogLogger = pipe(
  Logger.make(
    ({ logLevel, message, annotations, cause, context, date, fiberId, spans }) => {
      const payload: Parameters<typeof datadogLogs.logger.log> = [
        String(message),
        {
          ...annotations,
        },
        statusType(logLevel),
        Cause.squash(cause) as Error,
      ] as const

      globalThis.console.log(`[DD]`, {
        orig: {
          logLevel,
          message,
          annotations,
          cause,
          context,
          date,
          fiberId,
          spans,
        },
        payload,
      })

      // datadogLogs.logger.log(...payload)
    },
  ),
)

export { DatadogLogger as Logger }
