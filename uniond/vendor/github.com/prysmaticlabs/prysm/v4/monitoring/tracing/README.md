#### How to view collected traces

##### Prerequisites:
 - [Docker](https://www.docker.com/get-started) (For Jaeger image)
 - [Go](https://golang.org/) 1.11+ (For execution traces collected by pprof)

##### Using Jaeger
Tracing is disabled by default, to enable, you can use the option `--enable-tracing`.
Jaeger endpoint can be configured with the `--tracing-endpoint` option and defaults to `http://127.0.0.1:14268`.

Run Jaeger:
```sh
$ docker run -d --name jaeger   -e COLLECTOR_ZIPKIN_HTTP_PORT=9411   -p 5775:5775/udp   -p 6831:6831/udp   -p 6832:6832/udp   -p 5778:5778   -p 16686:16686   -p 14268:14268   -p 9411:9411   jaegertracing/all-in-one:1.6
```

This will start the UI at `http://localhost:16686`

##### Using the Go tool
Tracing is disabled by default, to enable, you can use the option `--enable-tracing`.
Run the application using the `--pprof` option to enable pprof (for trace collection).

To collect traces for 5 seconds:
```sh
$ curl http://localhost:6060/debug/pprof/trace?seconds=5 -o trace.out
```

View the trace with:
```sh
$ go tool trace trace.out
2018/05/04 10:39:59 Parsing trace...
2018/05/04 10:39:59 Splitting trace...
2018/05/04 10:39:59 Opening browser. Trace viewer is listening on http://127.0.0.1:51803
```

#### How to collect additional traces

We use the OpenCensus library to create traces. To trace the execution of a p2p
message through the system, we must define [spans](https://godoc.org/go.opencensus.io/trace#Span) around the code that handles the message. To correlate the trace with other spans defined for the same message, use the context passed inside the [Message](https://godoc.org/github.com/prysmaticlabs/prysm/shared/deprecated-p2p#Message) struct to create a span:

```go
var msg p2p.Message
var mySpan *trace.Span
msg.Ctx, mySpan = trace.StartSpan(msg.Ctx, "myOperation")
myOperation()
mySpan.End()
```

Another example on how to define spans can be found here: https://godoc.org/go.opencensus.io/trace#example-StartSpan
