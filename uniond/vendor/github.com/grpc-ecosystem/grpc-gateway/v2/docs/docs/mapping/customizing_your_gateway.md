---
layout: default
title: Customizing your gateway
nav_order: 5
parent: Mapping
---

# Customizing your gateway

## Message serialization

### Custom serializer

You might want to serialize request/response messages in MessagePack instead of JSON, for example:

1. Write a custom implementation of [`Marshaler`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#Marshaler).

2. Register your marshaler with [`WithMarshalerOption`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#WithMarshalerOption).

   e.g.

   ```go
   var m your.MsgPackMarshaler
   mux := runtime.NewServeMux(
   	runtime.WithMarshalerOption("application/x-msgpack", m),
   )
   ```

You can see [the default implementation for JSON](https://github.com/grpc-ecosystem/grpc-gateway/blob/main/runtime/marshal_jsonpb.go) for reference.

### Using proto names in JSON

The protocol buffer compiler generates camelCase JSON tags that are used by default.
If you want to use the exact case used in the proto files, set `UseProtoNames: true`:

```go
mux := runtime.NewServeMux(
	runtime.WithMarshalerOption(runtime.MIMEWildcard, &runtime.JSONPb{
		MarshalOptions: protojson.MarshalOptions{
			UseProtoNames: true,
		},
		UnmarshalOptions: protojson.UnmarshalOptions{
			DiscardUnknown: true,
		},
	}),
)
```

### Pretty-print JSON responses when queried with ?pretty

You can have Elasticsearch-style `?pretty` support in your gateway's endpoints as follows:

1. Wrap the ServeMux using a stdlib [`http.HandlerFunc`](https://golang.org/pkg/net/http/#HandlerFunc) that translates the provided query parameter into a custom `Accept` header.

2. Register a pretty-printing marshaler for that MIME code.

For example:

```go
mux := runtime.NewServeMux(
	runtime.WithMarshalerOption("application/json+pretty", &runtime.JSONPb{
		MarshalOptions: protojson.MarshalOptions{
			Indent: "  ",
			Multiline: true, // Optional, implied by presence of "Indent".
		},
		UnmarshalOptions: protojson.UnmarshalOptions{
			DiscardUnknown: true,
		},
	}),
)
prettier := func(h http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// checking Values as map[string][]string also catches ?pretty and ?pretty=
		// r.URL.Query().Get("pretty") would not.
		if _, ok := r.URL.Query()["pretty"]; ok {
			r.Header.Set("Accept", "application/json+pretty")
		}
		h.ServeHTTP(w, r)
	})
}
http.ListenAndServe(":8080", prettier(mux))
```

Now, either when passing the header `Accept: application/json+pretty` or appending `?pretty` to your HTTP endpoints, the response will be pretty-printed.

Note that this will conflict with any methods having input messages with fields named `pretty`; also, this example code does not remove the query parameter `pretty` from further processing.

## Customize unmarshaling per Content-Type

Having different unmarshaling options per Content-Type is as easy as configuring a custom marshaler:

```go
mux := runtime.NewServeMux(
	runtime.WithMarshalerOption("application/json+strict", &runtime.JSONPb{
		UnmarshalOptions: &protojson.UnmarshalOptions{
			DiscardUnknown: false, // explicit "false", &protojson.UnmarshalOptions{} would have the same effect
		},
	}),
)
```

## Mapping from HTTP request headers to gRPC client metadata

You might not like [the default mapping rule](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#DefaultHeaderMatcher) and might want to pass through all the HTTP headers, for example:

1. Write a [`HeaderMatcherFunc`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#HeaderMatcherFunc).

2. Register the function with [`WithIncomingHeaderMatcher`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#WithIncomingHeaderMatcher)

   e.g.

   ```go
   func CustomMatcher(key string) (string, bool) {
   	switch key {
   	case "X-Custom-Header1":
   		return key, true
   	case "X-Custom-Header2":
   		return "custom-header2", true
   	default:
   		return key, false
   	}
   }

   mux := runtime.NewServeMux(
   	runtime.WithIncomingHeaderMatcher(CustomMatcher),
   )
   ```

To keep the [the default mapping rule](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#DefaultHeaderMatcher) alongside with your own rules write:

```go
func CustomMatcher(key string) (string, bool) {
	switch key {
	case "X-User-Id":
		return key, true
	default:
		return runtime.DefaultHeaderMatcher(key)
	}
}
```

It will work with both:

```sh
$ curl --header "x-user-id: 100d9f38-2777-4ee2-ac3b-b3a108f81a30" ...
```

and

```sh
$ curl --header "X-USER-ID: 100d9f38-2777-4ee2-ac3b-b3a108f81a30" ...
```

To access this header on gRPC server side use:

```go
userID := ""
if md, ok := metadata.FromIncomingContext(ctx); ok {
	if uID, ok := md["x-user-id"]; ok {
		userID = strings.Join(uID, ",")
	}
}
```

## Mapping from gRPC server metadata to HTTP response headers

Use [`WithOutgoingHeaderMatcher`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#WithOutgoingHeaderMatcher). See [gRPC metadata docs](https://github.com/grpc/grpc-go/blob/master/Documentation/grpc-metadata.md) for more info on sending / receiving gRPC metadata, for example:

```go
if appendCustomHeader {
	grpc.SendHeader(ctx, metadata.New(map[string]string{
		"x-custom-header1": "value",
	}))
}
```

## Mutate response messages or set response headers

### Set HTTP headers

You might want to return a subset of response fields as HTTP response headers; You might want to simply set an application-specific token in a header. Or you might want to mutate the response messages to be returned.

1. Write a filter function.

```go
func myFilter(ctx context.Context, w http.ResponseWriter, resp proto.Message) error {
	t, ok := resp.(*externalpb.Tokenizer)
	if ok {
		w.Header().Set("X-My-Tracking-Token", t.Token)
		t.Token = ""
	}
	return nil
}
```

2. Register the filter with [`WithForwardResponseOption`](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#WithForwardResponseOption)

e.g.

```go
mux := runtime.NewServeMux(
	runtime.WithForwardResponseOption(myFilter),
)
```

### Controlling HTTP response status codes

To have the most control over the HTTP response status codes, you can use custom metadata.

While handling the rpc, set the intended status code:

```go
_ = grpc.SetHeader(ctx, metadata.Pairs("x-http-code", "401"))
```

Now, before sending the HTTP response, we need to check for this metadata pair and explicitly set the status code for the response if found.
To do so, create a function and hook it into the gRPC-Gateway as a Forward Response Option.

The function looks like this:

```go
func httpResponseModifier(ctx context.Context, w http.ResponseWriter, p proto.Message) error {
	md, ok := runtime.ServerMetadataFromContext(ctx)
	if !ok {
		return nil
	}

	// set http status code
	if vals := md.HeaderMD.Get("x-http-code"); len(vals) > 0 {
		code, err := strconv.Atoi(vals[0])
		if err != nil {
			return err
		}
		// delete the headers to not expose any grpc-metadata in http response
		delete(md.HeaderMD, "x-http-code")
		delete(w.Header(), "Grpc-Metadata-X-Http-Code")
		w.WriteHeader(code)
	}

	return nil
}
```

And it gets hooked into the gRPC-Gateway with:

```go
gwMux := runtime.NewServeMux(
	runtime.WithForwardResponseOption(httpResponseModifier),
)
```

Additional responses can be added to the Protocol Buffer definitions to match the new status codes:

```protobuf
service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply) {
    option (google.api.http) = {
      post: "/v1/example/echo"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      responses: {
        key: "201"
        value: {
          description: "A successful response."
          schema: {
            json_schema: {
              ref: ".mypackage.HelloReply"
            }
          }
        }
      }
    };
  }

  rpc SayGoodbye (GoodbyeRequest) returns (google.protobuf.Empty) {
    option (google.api.http) = {
      delete: "/v1/example/echo/{id}"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      responses: {
        key: "204"
        value: {
          description: "A successful response."
          schema: {}
        }
      }
    };
  }
}
```

## Error handler

To override error handling for a `*runtime.ServeMux`, use the
`runtime.WithErrorHandler` option. This will configure all unary error
responses to pass through this error handler.

## Stream Error Handler

The error handler described in the previous section applies only to RPC methods that have a unary response.

When the method has a streaming response, gRPC-Gateway handles that by emitting a newline-separated stream of "chunks". Each chunk is an envelope that can contain either a response message or an error. Only the last chunk will include an error, and only when the RPC handler ends abnormally (i.e. with an error code).

Because of the way the errors are included in the response body, the other error handler signature is insufficient. So for server streams, you must install a _different_ error handler:

```go
mux := runtime.NewServeMux(
	runtime.WithStreamErrorHandler(handleStreamError),
)
```

The signature of the handler is much more rigid because we need to know the structure of the error payload to properly encode the "chunk" schema into an OpenAPI spec.

So the function must return a `*runtime.StreamError`. The handler can choose to omit some fields and can filter/transform the original error, such as stripping stack traces from error messages.

Here's an example custom handler:

```go
// handleStreamError overrides default behavior for computing an error
// message for a server stream.
//
// It uses a default "502 Bad Gateway" HTTP code, only emits "safe"
// messages and does not set the details field (so it will
// be omitted from the resulting JSON object that is sent to client).
func handleStreamError(ctx context.Context, err error) *status.Status {
	code := codes.Internal
	msg := "unexpected error"
	if s, ok := status.FromError(err); ok {
		code = s.Code()
		// default message, based on the gRPC status
		msg = s.Message()
		// see if error details include "safe" message to send
		// to external callers
		for _, msg := range s.Details() {
			if safe, ok := msg.(*SafeMessage); ok {
				msg = safe.Text
				break
			}
		}
	}
	return status.Errorf(code, msg)
}
```

If no custom handler is provided, the default stream error handler will include any gRPC error attributes (code, message, detail messages), if the error being reported includes them. If the error does not have these attributes, a gRPC code of `Unknown` (2) is reported.

## Controlling path parameter unescaping

<!-- TODO(v3): Remove comments about default behavior -->

By default, gRPC-Gateway unescapes the entire URL path string attempting to route a request. This causes routing errors when the path parameter contains an illegal character such as `/`.

To replicate the behavior described in [google.api.http](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto#L224), use [runtime.WithUnescapingMode()](https://pkg.go.dev/github.com/grpc-ecosystem/grpc-gateway/runtime?tab=doc#WithUnescapingMode) to configure the unescaping behavior, as in the example below:

```go
mux := runtime.NewServeMux(
	runtime.WithUnescapingMode(runtime.UnescapingModeAllExceptReserved),
)
```

For multi-segment parameters (e.g. `{id=**}`) [RFC 6570](https://tools.ietf.org/html/rfc6570) Reserved Expansion characters are left escaped and the gRPC API will need to unescape them.

To replicate the default V2 escaping behavior but also allow passing pct-encoded `/` characters, the ServeMux can be configured as in the example below:

```go
mux := runtime.NewServeMux(
	runtime.WithUnescapingMode(runtime.UnescapingModeAllCharacters),
)
```

## Routing Error handler

To override the error behavior when `*runtime.ServeMux` was not able to serve the request due to routing issues, use the `runtime.WithRoutingErrorHandler` option.

This will configure all HTTP routing errors to pass through this error handler. The default behavior is to map HTTP error codes to gRPC errors.

HTTP statuses and their mappings to gRPC statuses:

- HTTP `404 Not Found` -> gRPC `5 NOT_FOUND`
- HTTP `405 Method Not Allowed` -> gRPC `12 UNIMPLEMENTED`
- HTTP `400 Bad Request` -> gRPC `3 INVALID_ARGUMENT`

This method is not used outside of the initial routing.

### Customizing Routing Errors

If you want to retain HTTP `405 Method Not Allowed` instead of allowing it to be converted to the equivalent of the gRPC `12 UNIMPLEMENTED`, which is  HTTP `501 Not Implmented` you can use the following example:

```go
func handleRoutingError(ctx context.Context, mux *ServeMux, marshaler Marshaler, w http.ResponseWriter, r *http.Request, httpStatus int) {
	if httpStatus != http.StatusMethodNotAllowed {
		runtime.DefaultRoutingErrorHandler(ctx, mux, marshaler, writer, request, httpStatus)
		return
	}

	// Use HTTPStatusError to customize the DefaultHTTPErrorHandler status code
	err := &HTTPStatusError{
		HTTPStatus: httpStatus
		Err:        status.Error(codes.Unimplemented, http.StatusText(httpStatus))
	}

	runtime.DefaultHTTPErrorHandler(ctx, mux, marshaler, w , r, err)
}
```

To use this routing error handler, construct the mux as follows:
```go
mux := runtime.NewServeMux(
	runtime.WithRoutingErrorHandler(handleRoutingError),
)
```
