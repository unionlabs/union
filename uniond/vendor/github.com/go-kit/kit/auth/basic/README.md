This package provides a Basic Authentication middleware.

It'll try to compare credentials from Authentication request header to a username/password pair in middleware constructor.

More details about this type of authentication can be found in [Mozilla article](https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication).

## Usage

```go
import httptransport "github.com/go-kit/kit/transport/http"

httptransport.NewServer(
		AuthMiddleware(cfg.auth.user, cfg.auth.password, "Example Realm")(makeUppercaseEndpoint()),
		decodeMappingsRequest,
		httptransport.EncodeJSONResponse,
		httptransport.ServerBefore(httptransport.PopulateRequestContext),
	)
```

For AuthMiddleware to be able to pick up the Authentication header from an HTTP request we need to pass it through the context with something like ```httptransport.ServerBefore(httptransport.PopulateRequestContext)```.