/*
`ctxlogrus` is a ctxlogger that is backed by logrus

It accepts a user-configured `logrus.Logger` that will be used for logging. The same `logrus.Logger` will
be populated into the `context.Context` passed into gRPC handler code.

You can use `ctx_logrus.Extract` to log into a request-scoped `logrus.Logger` instance in your handler code.

As `ctx_logrus.Extract` will iterate all tags on from `grpc_ctxtags` it is therefore expensive so it is advised that you
extract once at the start of the function from the context and reuse it for the remainder of the function (see examples).

Please see examples and tests for examples of use.
*/
package ctxlogrus
