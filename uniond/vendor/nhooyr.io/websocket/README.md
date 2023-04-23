# websocket

[![godoc](https://godoc.org/nhooyr.io/websocket?status.svg)](https://pkg.go.dev/nhooyr.io/websocket)
[![coverage](https://img.shields.io/badge/coverage-88%25-success)](https://nhooyrio-websocket-coverage.netlify.app)

websocket is a minimal and idiomatic WebSocket library for Go.

## Install

```bash
go get nhooyr.io/websocket
```

## Highlights

- Minimal and idiomatic API
- First class [context.Context](https://blog.golang.org/context) support
- Fully passes the WebSocket [autobahn-testsuite](https://github.com/crossbario/autobahn-testsuite)
- [Single dependency](https://pkg.go.dev/nhooyr.io/websocket?tab=imports)
- JSON and protobuf helpers in the [wsjson](https://pkg.go.dev/nhooyr.io/websocket/wsjson) and [wspb](https://pkg.go.dev/nhooyr.io/websocket/wspb) subpackages
- Zero alloc reads and writes
- Concurrent writes
- [Close handshake](https://pkg.go.dev/nhooyr.io/websocket#Conn.Close)
- [net.Conn](https://pkg.go.dev/nhooyr.io/websocket#NetConn) wrapper
- [Ping pong](https://pkg.go.dev/nhooyr.io/websocket#Conn.Ping) API
- [RFC 7692](https://tools.ietf.org/html/rfc7692) permessage-deflate compression
- Compile to [Wasm](https://pkg.go.dev/nhooyr.io/websocket#hdr-Wasm)

## Roadmap

- [ ] HTTP/2 [#4](https://github.com/nhooyr/websocket/issues/4)

## Examples

For a production quality example that demonstrates the complete API, see the
[echo example](./examples/echo).

For a full stack example, see the [chat example](./examples/chat).

### Server

```go
http.HandlerFunc(func (w http.ResponseWriter, r *http.Request) {
	c, err := websocket.Accept(w, r, nil)
	if err != nil {
		// ...
	}
	defer c.Close(websocket.StatusInternalError, "the sky is falling")

	ctx, cancel := context.WithTimeout(r.Context(), time.Second*10)
	defer cancel()

	var v interface{}
	err = wsjson.Read(ctx, c, &v)
	if err != nil {
		// ...
	}

	log.Printf("received: %v", v)

	c.Close(websocket.StatusNormalClosure, "")
})
```

### Client

```go
ctx, cancel := context.WithTimeout(context.Background(), time.Minute)
defer cancel()

c, _, err := websocket.Dial(ctx, "ws://localhost:8080", nil)
if err != nil {
	// ...
}
defer c.Close(websocket.StatusInternalError, "the sky is falling")

err = wsjson.Write(ctx, c, "hi")
if err != nil {
	// ...
}

c.Close(websocket.StatusNormalClosure, "")
```

## Comparison

### gorilla/websocket

Advantages of [gorilla/websocket](https://github.com/gorilla/websocket):

- Mature and widely used
- [Prepared writes](https://pkg.go.dev/github.com/gorilla/websocket#PreparedMessage)
- Configurable [buffer sizes](https://pkg.go.dev/github.com/gorilla/websocket#hdr-Buffers)

Advantages of nhooyr.io/websocket:

- Minimal and idiomatic API
  - Compare godoc of [nhooyr.io/websocket](https://pkg.go.dev/nhooyr.io/websocket) with [gorilla/websocket](https://pkg.go.dev/github.com/gorilla/websocket) side by side.
- [net.Conn](https://pkg.go.dev/nhooyr.io/websocket#NetConn) wrapper
- Zero alloc reads and writes ([gorilla/websocket#535](https://github.com/gorilla/websocket/issues/535))
- Full [context.Context](https://blog.golang.org/context) support
- Dial uses [net/http.Client](https://golang.org/pkg/net/http/#Client)
  - Will enable easy HTTP/2 support in the future
  - Gorilla writes directly to a net.Conn and so duplicates features of net/http.Client.
- Concurrent writes
- Close handshake ([gorilla/websocket#448](https://github.com/gorilla/websocket/issues/448))
- Idiomatic [ping pong](https://pkg.go.dev/nhooyr.io/websocket#Conn.Ping) API
  - Gorilla requires registering a pong callback before sending a Ping
- Can target Wasm ([gorilla/websocket#432](https://github.com/gorilla/websocket/issues/432))
- Transparent message buffer reuse with [wsjson](https://pkg.go.dev/nhooyr.io/websocket/wsjson) and [wspb](https://pkg.go.dev/nhooyr.io/websocket/wspb) subpackages
- [1.75x](https://github.com/nhooyr/websocket/releases/tag/v1.7.4) faster WebSocket masking implementation in pure Go
  - Gorilla's implementation is slower and uses [unsafe](https://golang.org/pkg/unsafe/).
- Full [permessage-deflate](https://tools.ietf.org/html/rfc7692) compression extension support
  - Gorilla only supports no context takeover mode
  - We use [klauspost/compress](https://github.com/klauspost/compress) for much lower memory usage ([gorilla/websocket#203](https://github.com/gorilla/websocket/issues/203))
- [CloseRead](https://pkg.go.dev/nhooyr.io/websocket#Conn.CloseRead) helper ([gorilla/websocket#492](https://github.com/gorilla/websocket/issues/492))
- Actively maintained ([gorilla/websocket#370](https://github.com/gorilla/websocket/issues/370))

#### golang.org/x/net/websocket

[golang.org/x/net/websocket](https://pkg.go.dev/golang.org/x/net/websocket) is deprecated.
See [golang/go/issues/18152](https://github.com/golang/go/issues/18152).

The [net.Conn](https://pkg.go.dev/nhooyr.io/websocket#NetConn) can help in transitioning
to nhooyr.io/websocket.

#### gobwas/ws

[gobwas/ws](https://github.com/gobwas/ws) has an extremely flexible API that allows it to be used
in an event driven style for performance. See the author's [blog post](https://medium.freecodecamp.org/million-websockets-and-go-cc58418460bb).

However when writing idiomatic Go, nhooyr.io/websocket will be faster and easier to use.
