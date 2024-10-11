# net/rpc

[net/rpc](https://golang.org/pkg/net/rpc) is an RPC transport that's part of the Go standard library.
It's a simple and fast transport that's appropriate when all of your services are written in Go.

Using net/rpc with Go kit is very simple.
Just write a simple binding from your service definition to the net/rpc definition.
See [netrpc_binding.go](https://github.com/go-kit/kit/blob/ec8b02591ee873433565a1ae9d317353412d1d27/examples/addsvc/netrpc_binding.go) for an example.

That's it!
The net/rpc binding can be registered to a name, and bound to an HTTP handler, the same as any other net/rpc endpoint.
And within your service, you can use standard Go kit components and idioms.
See [addsvc](https://github.com/go-kit/examples/tree/master/addsvc) for a complete working example with net/rpc support.
And remember: Go kit services can support multiple transports simultaneously.
