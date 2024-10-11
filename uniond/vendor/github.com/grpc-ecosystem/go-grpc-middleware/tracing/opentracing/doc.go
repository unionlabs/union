// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

/*
`grpc_opentracing` adds OpenTracing

OpenTracing Interceptors

These are both client-side and server-side interceptors for OpenTracing. They are a provider-agnostic, with backends
such as Zipkin, or Google Stackdriver Trace.

For a service that sends out requests and receives requests, you *need* to use both, otherwise downstream requests will
not have the appropriate requests propagated.

All server-side spans are tagged with grpc_ctxtags information.

For more information see:
http://opentracing.io/documentation/
https://github.com/opentracing/specification/blob/master/semantic_conventions.md

*/
package grpc_opentracing
