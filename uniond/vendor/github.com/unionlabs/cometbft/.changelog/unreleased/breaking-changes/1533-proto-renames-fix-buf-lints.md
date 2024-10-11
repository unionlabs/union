- `[abci]` Renamed the alias types for gRPC requests, responses, and service
  instances to follow the naming changes in the proto-derived
  `api/cometbft/abci/v1` package
  ([\#1533](https://github.com/cometbft/cometbft/pull/1533)):
  * The prefixed naming pattern `RequestFoo`, `ReponseFoo` changed to
    suffixed `FooRequest`, `FooResponse`.
  * Each method gets its own unique request and response type to allow for
    independent evolution with backward compatibility.
  * `ABCIClient` renamed to `ABCIServiceClient`.
  * `ABCIServer` renamed to `ABCIServiceServer`.
