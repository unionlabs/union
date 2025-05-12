import { PUBLIC_DATADOG_CLIENT_TOKEN } from "$env/static/public"
import { ENV, SERVICE_NAME } from "$lib/constants"
import { datadogLogs, StatusType } from "@datadog/browser-logs"
import { WebSdk } from "@effect/opentelemetry"
import { BatchSpanProcessor, ConsoleSpanExporter } from "@opentelemetry/sdk-trace-base"
import {
  Cause,
  Context,
  Effect,
  FiberRef,
  FiberRefs,
  flow,
  HashMap,
  Logger,
  LogLevel,
  Match,
  Option,
  pipe,
  String as Str,
  Struct,
  Tracer,
} from "effect"

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

/**
 * Format the traceId and spanId to be compatible with Datadog.
 *
 * Converts the OTEL traceId (128-bit uint as 32-hex-char string) and spanId (64-bit uint as 16-hex-char string)
 * to the Datadog traceId and spanId (64-bit uint) format.
 *
 * {@link https://docs.datadoghq.com/tracing/other_telemetry/connect_logs_and_traces/opentelemetry?tab=nodejs Correlating OpenTelemetry Traces and Logs}
 */
export const withDatadogFormat = (span: Tracer.AnySpan): Tracer.AnySpan => {
  console.log("span.traceId", span.traceId)
  const traceIdEnd = span.traceId.slice(span.traceId.length / 2)

  const traceId = BigInt(`0x${traceIdEnd}`).toString()
  const spanId = BigInt(`0x${span.spanId}`).toString()

  return Struct.evolve(span, {
    traceId: () => traceId,
    spanId: () => spanId,
  })
}

/**
 * Add identifiers from the current Span to the log annotations.
 *
 * {@link https://docs.datadoghq.com/tracing/other_telemetry/connect_logs_and_traces/opentelemetry?tab=nodejs Correlating OpenTelemetry Traces and Logs}
 */
export const withSpanAnnotations = <Message, Output>(
  self: Logger.Logger<Message, Output>,
): Logger.Logger<Message, Output> =>
  Logger.mapInputOptions(self, (options: Logger.Logger.Options<Message>) => {
    const span = Option.flatMap(
      FiberRefs.get(options.context, FiberRef.currentContext),
      Context.getOption(Tracer.ParentSpan),
    )
    if (span._tag === "None") {
      return options
    }

    const { spanId, traceId } = withDatadogFormat(span.value)

    return Struct.evolve(options, {
      annotations: flow(
        HashMap.set("dd.trace_id", traceId as unknown),
        HashMap.set("dd.span_id", spanId as unknown),
      ),
    })
  })

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
        String(message), // message
        { // context
          "annotations": annotations.toJSON(),
          "dd.span_id": spans,
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
  withSpanAnnotations,
)

const DatadogResource = WebSdk.layer(() => ({
  resource: { serviceName: SERVICE_NAME },
  spanProcessor: new BatchSpanProcessor(new OLTP()),
}))

export { DatadogLogger as Logger, DatadogResource as Resource }
