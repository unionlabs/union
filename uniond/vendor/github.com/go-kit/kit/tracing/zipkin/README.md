# Zipkin

## Development and Testing Set-up

Great efforts have been made to make [Zipkin] easier to test, develop and
experiment against. [Zipkin] can now be run from a single Docker container or by
running its self-contained executable jar without extensive configuration. In
its default configuration you will run [Zipkin] with a HTTP collector, In memory
Span storage backend and web UI on port 9411.

Example:
```
docker run -d -p 9411:9411 openzipkin/zipkin
```

[zipkin]: http://zipkin.io

## Middleware Usage

Follow the [addsvc] example to check out how to wire the [Zipkin] Middleware.
The changes should be relatively minor.

The [zipkin-go] package has Reporters to send Spans to the [Zipkin] HTTP and
Kafka Collectors.

### Configuring the Zipkin HTTP Reporter

To use the HTTP Reporter with a [Zipkin] instance running on localhost you
bootstrap [zipkin-go] like this:

```go
var (
  serviceName        = "MyService"
  serviceHostPort    = "localhost:8000"
  zipkinHTTPEndpoint = "http://localhost:9411/api/v2/spans"
)

// create an instance of the HTTP Reporter.
reporter := zipkin.NewReporter(zipkinHTTPEndpoint)

// create our tracer's local endpoint (how the service is identified in Zipkin).
localEndpoint, err := zipkin.NewEndpoint(serviceName, serviceHostPort)

// create our tracer instance.
tracer, err = zipkin.NewTracer(reporter, zipkin.WithLocalEndpoint(localEndpoint))
  ...

```

[zipkin-go]: https://github.com/openzipkin/zipkin-go
[addsvc]: https://github.com/go-kit/examples/tree/master/addsvc
[Log]: https://github.com/go-kit/kit/tree/master/log

### Tracing Resources

Here is an example of how you could trace resources and work with local spans.
```go
import (
	zipkin "github.com/openzipkin/zipkin-go"
)

func (svc *Service) GetMeSomeExamples(ctx context.Context, ...) ([]Examples, error) {
  // Example of annotating a database query:
  var (
    spanContext model.SpanContext
    serviceName = "MySQL"
    serviceHost = "mysql.example.com:3306"
    queryLabel  = "GetExamplesByParam"
    query       = "select * from example where param = :value"
  )

  // retrieve the parent span from context to use as parent if available.
  if parentSpan := zipkin.SpanFromContext(ctx); parentSpan != nil {
    spanContext = parentSpan.Context()
  }

  // create the remote Zipkin endpoint
  ep, _ := zipkin.NewEndpoint(serviceName, serviceHost)

  // create a new span to record the resource interaction
  span := zipkin.StartSpan(
    queryLabel,
    zipkin.Parent(parentSpan.Context()),
    zipkin.WithRemoteEndpoint(ep),
  )

	// add interesting key/value pair to our span
	span.SetTag("query", query)

	// add interesting timed event to our span
	span.Annotate(time.Now(), "query:start")

	// do the actual query...

	// let's annotate the end...
	span.Annotate(time.Now(), "query:end")

	// we're done with this span.
	span.Finish()

	// do other stuff
	...
}
```
