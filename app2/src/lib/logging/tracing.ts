import { PUBLIC_GIT_REV, PUBLIC_LOG_ENDPOINT, PUBLIC_LOG_TOKEN } from "$env/static/public"
import { ENV } from "$lib/constants.js"
import * as WebSdk from "@effect/opentelemetry/WebSdk"
import { OTLPLogExporter as ProtoOTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-proto"
import { OTLPMetricExporter as HttpOTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http"
import { OTLPMetricExporter as ProtoOTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-proto"
import { OTLPTraceExporter as HttpOTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http"
import { OTLPTraceExporter as ProtoOTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-proto"
import {
  BatchLogRecordProcessor,
  ConsoleLogRecordExporter,
  SimpleLogRecordProcessor,
} from "@opentelemetry/sdk-logs"
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics"
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base"
import { Effect, Layer, Match, Option, pipe, String as Str } from "effect"

export const TracingLive = Layer.unwrapEffect(
  Effect.gen(function*() {
    const serviceName = "app"
    const serviceVersion = PUBLIC_GIT_REV

    const env = pipe(
      Match.value(ENV()),
      Match.when("PRODUCTION", () => "production"),
      Match.when("STAGING", () => "staging"),
      Match.when("DEVELOPMENT", () => "development"),
      Match.exhaustive,
    )

    if (env === "DEVELOPMENT") {
      return WebSdk.layer(() => ({
        resource: {
          serviceName,
          serviceVersion,
          attributes: {
            env,
          },
        },
        logRecordProcessor: new SimpleLogRecordProcessor(new ConsoleLogRecordExporter()),
        spanProcessor: new BatchSpanProcessor(new HttpOTLPTraceExporter()),
        metricReader: new PeriodicExportingMetricReader({
          exporter: new HttpOTLPMetricExporter({}),
        }),
      }))
    }

    const endpoint = pipe(
      Option.fromNullable(PUBLIC_LOG_ENDPOINT),
      Option.flatMap(Option.liftPredicate(Str.isNonEmpty)),
    )

    if (Option.isNone(endpoint) || Str.isEmpty(endpoint.value)) {
      yield* Effect.logWarning(
        `The app is running in a ${env} environment, but no OTEL logging endpoint is configured.`,
      )
      return Layer.empty
    }

    const headers = {
      Authorization: `Bearer ${PUBLIC_LOG_TOKEN}`,
    }

    return WebSdk.layer(() => ({
      resource: {
        serviceName,
        serviceVersion,
        attributes: {
          env,
        },
      },
      logRecordProcessor: new BatchLogRecordProcessor(
        new ProtoOTLPLogExporter({
          url: `${endpoint.value}/v1/logs`,
          headers,
        }),
      ),
      spanProcessor: new BatchSpanProcessor(
        new ProtoOTLPTraceExporter({
          url: `${endpoint.value}/v1/traces`,
          headers,
        }),
      ),
      metricReader: new PeriodicExportingMetricReader({
        exporter: new ProtoOTLPMetricExporter({
          url: `${endpoint.value}/v1/metrics`,
          headers,
        }),
      }),
    }))
  }),
)
