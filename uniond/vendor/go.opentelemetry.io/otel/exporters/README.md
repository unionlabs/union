# OpenTelemetry Exporters

Once the OpenTelemetry SDK has created and processed telemetry, it needs to be exported.
This package contains exporters for this purpose.

## Exporter Packages

The following exporter packages are provided with the following OpenTelemetry signal support.

|                                           Exporter Package                                            | Logs | Metrics | Traces |
|:-----------------------------------------------------------------------------------------------------:|:----:|:-------:|:------:|
|       [go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploghttp](./otlp/otlplog/otlploghttp)       |   ✓  |         |        |
| [go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc](./otlp/otlpmetric/otlpmetricgrpc) |      |   ✓     |        |
| [go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetrichttp](./otlp/otlpmetric/otlpmetrichttp) |      |   ✓     |        |
|   [go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc](./otlp/otlptrace/otlptracegrpc)   |      |         |   ✓    |
|   [go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp](./otlp/otlptrace/otlptracehttp)   |      |         |   ✓    |
|                     [go.opentelemetry.io/otel/exporters/prometheus](./prometheus)                     |      |   ✓     |        |
|                [go.opentelemetry.io/otel/exporters/stdout/stdoutlog](./stdout/stdoutlog)              |   ✓  |         |        |
|            [go.opentelemetry.io/otel/exporters/stdout/stdoutmetric](./stdout/stdoutmetric)            |      |   ✓     |        |
|             [go.opentelemetry.io/otel/exporters/stdout/stdouttrace](./stdout/stdouttrace)             |      |         |   ✓    |
|                         [go.opentelemetry.io/otel/exporters/zipkin](./zipkin)                         |      |         |   ✓    |

See the [OpenTelemetry registry] for 3rd-party exporters compatible with this project.

[OpenTelemetry registry]: https://opentelemetry.io/registry/?language=go&component=exporter
