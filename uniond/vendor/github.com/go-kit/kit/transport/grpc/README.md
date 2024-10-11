# grpc

[gRPC](http://www.grpc.io/) is an excellent, modern IDL and transport for
microservices. If you're starting a greenfield project, go-kit strongly
recommends gRPC as your default transport.

One important note is that while gRPC supports streaming requests and replies,
go-kit does not. You can still use streams in your service, but their
implementation will not be able to take advantage of many go-kit features like middleware.

Using gRPC and go-kit together is very simple.

First, define your service using protobuf3. This is explained
[in gRPC documentation](http://www.grpc.io/docs/#defining-a-service).
See
[addsvc.proto](https://github.com/go-kit/examples/blob/master/addsvc/pb/addsvc.proto)
for an example. Make sure the proto definition matches your service's go-kit
(interface) definition.

Next, get the protoc compiler.

You can download pre-compiled binaries from the
[protobuf release page](https://github.com/google/protobuf/releases).
You will unzip a folder called `protoc3` with a subdirectory `bin` containing
an executable. Move that executable somewhere in your `$PATH` and you're good
to go!

It can also be built from source.

```sh
brew install autoconf automake libtool
git clone https://github.com/google/protobuf
cd protobuf
./autogen.sh ; ./configure ; make ; make install
```

Then, compile your service definition, from .proto to .go.

```sh
protoc add.proto --go_out=plugins=grpc:.
```

Finally, write a tiny binding from your service definition to the gRPC
definition. It's a simple conversion from one domain to another.
See
[grpc.go](https://github.com/go-kit/examples/blob/master/addsvc/pkg/addtransport/grpc.go)
for an example.

That's it!
The gRPC binding can be bound to a listener and serve normal gRPC requests.
And within your service, you can use standard go-kit components and idioms.
See [addsvc](https://github.com/go-kit/examples/tree/master/addsvc/) for
a complete working example with gRPC support. And remember: go-kit services
can support multiple transports simultaneously.
