# package tracing

`package tracing` provides [Dapper]-style request tracing to services.

## Rationale

Request tracing is a fundamental building block for large distributed
applications. It's instrumental in understanding request flows, identifying
hot spots, and diagnosing errors. All microservice infrastructures will
benefit from request tracing; sufficiently large infrastructures will require
it.

## Zipkin

[Zipkin] is one of the most used OSS distributed tracing platforms available
with support for many different languages and frameworks. Go kit provides
bindings to the native Go tracing implementation [zipkin-go]. If using Zipkin
with Go kit in a polyglot microservices environment, this is the preferred
binding to use. Instrumentation exists for `kit/transport/http` and
`kit/transport/grpc`. The bindings are highlighted in the [addsvc] example. For
more information regarding Zipkin feel free to visit [Zipkin's Gitter].

## OpenCensus

Go kit supports transport and endpoint middlewares for the [OpenCensus]
instrumentation library. OpenCensus provides a cross language consistent data
model and instrumentation libraries for tracing and metrics. From this data
model it allows exports to various tracing and metrics backends including but
not limited to Zipkin, Prometheus, Stackdriver Trace & Monitoring, Jaeger,
AWS X-Ray and Datadog. Go kit uses the [opencensus-go] implementation to power
its middlewares.

## OpenTracing

Go kit supports the [OpenTracing] API and uses the [opentracing-go] package to
provide tracing middlewares for its servers and clients. Currently OpenTracing
instrumentation exists for `kit/transport/http` and `kit/transport/grpc`.

Since [OpenTracing] is an effort to provide a generic API, Go kit should support
a multitude of tracing backends. If a Tracer implementation or OpenTracing
bridge in Go for your back-end exists, it should work out of the box.

Please note that the "world view" of existing tracing systems do differ.
OpenTracing can not guarantee you that tracing alignment is perfect in a
microservice environment especially one which is not exclusively OpenTracing
enabled or switching from one tracing backend to another truly entails just a
change in configuration.

The following tracing back-ends are known to work with Go kit through the
OpenTracing interface and are highlighted in the [addsvc] example.

### AppDash

[Appdash] support is available straight from their system repository in the
[appdash/opentracing] directory.

### LightStep

[LightStep] support is available through their standard Go package
[lightstep-tracer-go].

### Zipkin

[Zipkin] support is available through the [zipkin-go-opentracing] package.

## OpenTelemetry

[OpenTelemetry] came to life as a result of merging [OpenCensus] and [OpenTracing].
Go kit instrumentation can be found in [opentelemetry-go-contrib]
which is a central repository of instrumentation libraries.

[Dapper]: http://research.google.com/pubs/pub36356.html
[addsvc]: https://github.com/go-kit/examples/tree/master/addsvc
[README]: https://github.com/go-kit/kit/blob/master/tracing/zipkin/README.md

[OpenTracing]: http://opentracing.io
[opentracing-go]: https://github.com/opentracing/opentracing-go

[Zipkin]: http://zipkin.io/
[Open Zipkin GitHub]: https://github.com/openzipkin
[zipkin-go-opentracing]: https://github.com/openzipkin-contrib/zipkin-go-opentracing
[zipkin-go]: https://github.com/openzipkin/zipkin-go
[Zipkin's Gitter]: https://gitter.im/openzipkin/zipkin

[Appdash]: https://github.com/sourcegraph/appdash
[appdash/opentracing]: https://github.com/sourcegraph/appdash/tree/master/opentracing

[LightStep]: http://lightstep.com/
[lightstep-tracer-go]: https://github.com/lightstep/lightstep-tracer-go

[OpenCensus]: https://opencensus.io/
[opencensus-go]: https://github.com/census-instrumentation/opencensus-go

[OpenTelemetry]: https://opentelemetry.io/
[opentelemetry-go-contrib]: https://github.com/open-telemetry/opentelemetry-go-contrib
