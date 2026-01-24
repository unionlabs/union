import { PUBLIC_GIT_REV, PUBLIC_LOG_ENDPOINT, PUBLIC_LOG_TOKEN } from "$env/static/public"
import { ENV } from "$lib/constants.js"
import * as WebSdk from "@effect/opentelemetry/WebSdk"
import { OTLPLogExporter as ProtoOTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-proto"
import { OTLPMetricExporter as HttpOTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http"
import { OTLPMetricExporter as ProtoOTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-proto"
import { OTLPTraceExporter as HttpOTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http"
import { OTLPTraceExporter as ProtoOTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-proto"
import { CompressionAlgorithm } from "@opentelemetry/otlp-exporter-base"
import {
  BatchLogRecordProcessor,
  ConsoleLogRecordExporter,
  SimpleLogRecordProcessor,
} from "@opentelemetry/sdk-logs"
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics"
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base"
import { Effect, Layer, Match, Option, pipe, String as Str } from "effect"

import type { BufferConfig, ReadableSpan, SpanExporter } from "@opentelemetry/sdk-trace-base"

export interface ByteAwareBufferConfig extends BufferConfig {
  maxBatchBytes?: number
}

export class ByteAwareBatchSpanProcessor extends BatchSpanProcessor {
  private readonly maxBatchBytes: number
  private currentBytes = 0

  constructor(exporter: SpanExporter, config: ByteAwareBufferConfig = {}) {
    super(exporter, config)
    this.maxBatchBytes = config.maxBatchBytes ?? 4 * 1024 * 1024
  }

  onEnd(span: ReadableSpan): void {
    const spanBytes = this.estimateBytes(span)

    if (this.currentBytes + spanBytes >= this.maxBatchBytes) {
      this.forceFlush()
      this.currentBytes = 0
    }

    this.currentBytes += spanBytes
    super.onEnd(span)
  }

  private estimateBytes(span: ReadableSpan): number {
    return JSON.stringify({
      traceId: span.spanContext().traceId,
      spanId: span.spanContext().spanId,
      name: span.name,
      kind: span.kind,
      startTime: span.startTime,
      endTime: span.endTime,
      attributes: span.attributes,
      status: span.status,
      events: span.events,
      links: span.links,
    }).length
  }
}

export const TracingLive = Layer.unwrapEffect(
  Effect.gen(function*() {
    const serviceName = "app"
    const serviceVersion = PUBLIC_GIT_REV

    const env = pipe(
      Match.value(ENV()),
      Match.when("PRODUCTION", () => "production" as const),
      Match.when("STAGING", () => "staging" as const),
      Match.when("DEVELOPMENT", () => "development" as const),
      Match.exhaustive,
    )
    if (env === "development") {
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

    const compression = CompressionAlgorithm.GZIP

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
          compression,
        }),
        {
          maxExportBatchSize: 128,
          maxQueueSize: 1024,
        },
      ),
      spanProcessor: new ByteAwareBatchSpanProcessor(
        new ProtoOTLPTraceExporter({
          url: `${endpoint.value}/v1/traces`,
          headers,
          compression,
        }),
        {
          maxBatchBytes: 9_999,
        },
      ),
      metricReader: new PeriodicExportingMetricReader({
        exporter: new ProtoOTLPMetricExporter({
          url: `${endpoint.value}/v1/metrics`,
          headers,
          compression,
        }),
      }),
    }))
  }),
)
