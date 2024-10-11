## Anti-Patterns of Client creation

### How to properly create a `ClientConn`: `grpc.NewClient`

[`grpc.NewClient`](https://pkg.go.dev/google.golang.org/grpc#NewClient) is the
function in the gRPC library that creates a virtual connection from a client
application to a gRPC server.  It takes a target URI (which represents the name
of a logical backend service and resolves to one or more physical addresses) and
a list of options, and returns a
[`ClientConn`](https://pkg.go.dev/google.golang.org/grpc#ClientConn) object that
represents the virtual connection to the server.  The `ClientConn` contains one
or more actual connections to real servers and attempts to maintain these
connections by automatically reconnecting to them when they break.  `NewClient`
was introduced in gRPC-Go v1.63.

### The wrong way: `grpc.Dial`

[`grpc.Dial`](https://pkg.go.dev/google.golang.org/grpc#Dial) is a deprecated
function that also creates the same virtual connection pool as `grpc.NewClient`.
However, unlike `grpc.NewClient`, it immediately starts connecting and supports
a few additional `DialOption`s that control this initial connection attempt.
These are: `WithBlock`, `WithTimeout`, `WithReturnConnectionError`, and
`FailOnNonTempDialError`.

That `grpc.Dial` creates connections immediately is not a problem in and of
itself, but this behavior differs from how gRPC works in all other languages,
and it can be convenient to have a constructor that does not perform I/O.  It
can also be confusing to users, as most people expect a function called `Dial`
to create _a_ connection which may need to be recreated if it is lost.

`grpc.Dial` uses "passthrough" as the default name resolver for backward
compatibility while `grpc.NewClient` uses "dns" as its default name resolver.
This subtle difference is important to legacy systems that also specified a
custom dialer and expected it to receive the target string directly.

For these reasons, using `grpc.Dial` is discouraged.  Even though it is marked
as deprecated, we will continue to support it until a v2 is released (and no
plans for a v2 exist at the time this was written).

### Especially bad: using deprecated `DialOptions`

`FailOnNonTempDialError`, `WithBlock`, and `WithReturnConnectionError` are three
`DialOption`s that are only supported by `Dial` because they only affect the
behavior of `Dial` itself. `WithBlock` causes `Dial` to wait until the
`ClientConn` reports its `State` as `connectivity.Connected`.  The other two deal
with returning connection errors before the timeout (`WithTimeout` or on the
context when using `DialContext`).

The reason these options can be a problem is that connections with a
`ClientConn` are dynamic -- they may come and go over time.  If your client
successfully connects, the server could go down 1 second later, and your RPCs
will fail.  "Knowing you are connected" does not tell you much in this regard.

Additionally, _all_ RPCs created on an "idle" or a "connecting" `ClientConn`
will wait until their deadline or until a connection is established before
failing.  This means that you don't need to check that a `ClientConn` is "ready"
before starting your RPCs.  By default, RPCs will fail if the `ClientConn`
enters the "transient failure" state, but setting `WaitForReady(true)` on a
call will cause it to queue even in the "transient failure" state, and it will
only ever fail due to a deadline, a server response, or a connection loss after
the RPC was sent to a server.

Some users of `Dial` use it as a way to validate the configuration of their
system.  If you wish to maintain this behavior but migrate to `NewClient`, you
can call `GetState`, then `Connect` if the state is `Idle` and
`WaitForStateChange` until the channel is connected.  However, if this fails,
it does not mean that your configuration was bad - it could also mean the
service is not reachable by the client due to connectivity reasons.

## Best practices for error handling in gRPC

Instead of relying on failures at dial time, we strongly encourage developers to
rely on errors from RPCs.  When a client makes an RPC, it can receive an error
response from the server.  These errors can provide valuable information about
what went wrong, including information about network issues, server-side errors,
and incorrect usage of the gRPC API.

By handling errors from RPCs correctly, developers can write more reliable and
robust gRPC applications.  Here are some best practices for error handling in
gRPC:

- Always check for error responses from RPCs and handle them appropriately.
- Use the `status` field of the error response to determine the type of error
  that occurred.
- When retrying failed RPCs, consider using the built-in retry mechanism
  provided by gRPC-Go, if available, instead of manually implementing retries.
  Refer to the [gRPC-Go retry example
  documentation](https://github.com/grpc/grpc-go/blob/master/examples/features/retry/README.md)
  for more information.  Note that this is not a substitute for client-side
  retries as errors that occur after an RPC starts on a server cannot be
  retried through gRPC's built-in mechanism.
- If making an outgoing RPC from a server handler, be sure to translate the
  status code before returning the error from your method handler.  For example,
  if the error is an `INVALID_ARGUMENT` status code, that probably means
  your service has a bug (otherwise it shouldn't have triggered this error), in
  which case `INTERNAL` is more appropriate to return back to your users.

### Example: Handling errors from an RPC

The following code snippet demonstrates how to handle errors from an RPC in
gRPC:

```go
ctx, cancel := context.WithTimeout(context.Background(), time.Second)
defer cancel()

res, err := client.MyRPC(ctx, &MyRequest{})
if err != nil {
    // Handle the error appropriately,
    // log it & return an error to the caller, etc.
    log.Printf("Error calling MyRPC: %v", err)
    return nil, err
}

// Use the response as appropriate
log.Printf("MyRPC response: %v", res)
```

To determine the type of error that occurred, you can use the status field of
the error response:

```go
resp, err := client.MakeRPC(context.TODO(), request)
if err != nil {
  if status, ok := status.FromError(err); ok {
    // Handle the error based on its status code
    if status.Code() == codes.NotFound {
      log.Println("Requested resource not found")
    } else {
      log.Printf("RPC error: %v", status.Message())
    }
  } else {
    // Handle non-RPC errors
    log.Printf("Non-RPC error: %v", err)
  }
  return
}

// Use the response as needed
log.Printf("Response received: %v", resp)
```

### Example: Using a backoff strategy

When retrying failed RPCs, use a backoff strategy to avoid overwhelming the
server or exacerbating network issues:

```go
var res *MyResponse
var err error

retryableStatusCodes := map[codes.Code]bool{
  codes.Unavailable: true, // etc
}

// Retry the RPC a maximum number of times.
for i := 0; i < maxRetries; i++ {
    // Make the RPC.
    res, err = client.MyRPC(context.TODO(), &MyRequest{})

    // Check if the RPC was successful.
    if !retryableStatusCodes[status.Code(err)] {
        // The RPC was successful or errored in a non-retryable way;
        // do not retry.
        break
    }

    // The RPC is retryable; wait for a backoff period before retrying.
    backoff := time.Duration(i+1) * time.Second
    log.Printf("Error calling MyRPC: %v; retrying in %v", err, backoff)
    time.Sleep(backoff)
}

// Check if the RPC was successful after all retries.
if err != nil {
    // All retries failed, so handle the error appropriately
    log.Printf("Error calling MyRPC: %v", err)
    return nil, err
}

// Use the response as appropriate.
log.Printf("MyRPC response: %v", res)
```
