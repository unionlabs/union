// Copyright 2017 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

/*
`grpcweb` implements the gRPC-Web spec as a wrapper around a gRPC-Go Server.

It allows web clients (see companion JS library) to talk to gRPC-Go servers over the gRPC-Web spec. It supports
HTTP/1.1 and HTTP2 encoding of a gRPC stream and supports unary and server-side streaming RPCs. Bi-di and client
streams are unsupported due to limitations in browser protocol support.

See https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md for the protocol specification.

Here's an example of how to use it inside an existing gRPC Go server on a separate http.Server that serves over TLS:

	grpcServer := grpc.Server()
	wrappedGrpc := grpcweb.WrapServer(grpcServer)
	tlsHttpServer.Handler = http.HandlerFunc(func(resp http.ResponseWriter, req *http.Request) {
		if wrappedGrpc.IsGrpcWebRequest(req) {
			wrappedGrpc.ServeHTTP(resp, req)
			return
		}
		// Fall back to other servers.
		http.DefaultServeMux.ServeHTTP(resp, req)
	})

If you'd like to have a standalone binary, please take a look at `grpcwebproxy`.

*/
package grpcweb
