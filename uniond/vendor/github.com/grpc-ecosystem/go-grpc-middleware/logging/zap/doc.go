/*
`grpc_zap` is a gRPC logging middleware backed by ZAP loggers

It accepts a user-configured `zap.Logger` that will be used for logging completed gRPC calls. The same `zap.Logger` will
be used for logging completed gRPC calls, and be populated into the `context.Context` passed into gRPC handler code.

On calling `StreamServerInterceptor` or `UnaryServerInterceptor` this logging middleware will add gRPC call information
to the ctx so that it will be present on subsequent use of the `ctx_zap` logger.

If a deadline is present on the gRPC request the grpc.request.deadline tag is populated when the request begins. grpc.request.deadline
is a string representing the time (RFC3339) when the current call will expire.

This package also implements request and response *payload* logging, both for server-side and client-side. These will be
logged as structured `jsonpb` fields for every message received/sent (both unary and streaming). For that please use
`Payload*Interceptor` functions for that. Please note that the user-provided function that determines whether to log
the full request/response payload needs to be written with care, this can significantly slow down gRPC.

ZAP can also be made as a backend for gRPC library internals. For that use `ReplaceGrpcLoggerV2`.


*Server Interceptor*
Below is a JSON formatted example of a log that would be logged by the server interceptor:

	{
	  "level": "info",									// string  zap log levels
	  "msg": "finished unary call",						// string  log message

	  "grpc.code": "OK",								// string  grpc status code
	  "grpc.method": "Ping",							// string  method name
	  "grpc.service": "mwitkow.testproto.TestService",  // string  full name of the called service
	  "grpc.start_time": "2006-01-02T15:04:05Z07:00",   // string  RFC3339 representation of the start time
	  "grpc.request.deadline": "2006-01-02T15:04:05Z07:00",   // string  RFC3339 deadline of the current request if supplied
	  "grpc.request.value": "something",				// string  value on the request
	  "grpc.time_ms": 1.345,							// float32 run time of the call in ms

	  "peer.address": {
	    "IP": "127.0.0.1",								// string  IP address of calling party
	    "Port": 60216,									// int     port call is coming in on
	    "Zone": ""										// string  peer zone for caller
	  },
	  "span.kind": "server",							// string  client | server
	  "system": "grpc"									// string

	  "custom_field": "custom_value",					// string  user defined field
	  "custom_tags.int": 1337,							// int     user defined tag on the ctx
	  "custom_tags.string": "something",				// string  user defined tag on the ctx
	}

*Payload Interceptor*
Below is a JSON formatted example of a log that would be logged by the payload interceptor:

	{
	  "level": "info",													// string zap log levels
	  "msg": "client request payload logged as grpc.request.content",   // string log message

	  "grpc.request.content": {											// object content of RPC request
	    "msg" : {														// object ZAP specific inner object
		  "value": "something",											// string defined by caller
	      "sleepTimeMs": 9999											// int    defined by caller
	    }
	  },
	  "grpc.method": "Ping",											// string method being called
	  "grpc.service": "mwitkow.testproto.TestService",					// string service being called

	  "span.kind": "client",											// string client | server
	  "system": "grpc"													// string
	}

Note - due to implementation ZAP differs from Logrus in the "grpc.request.content" object by having an inner "msg" object.


Please see examples and tests for examples of use.
Please see settable_test.go for canonical integration through "zaptest" with golang testing infrastructure.
*/
package grpc_zap
