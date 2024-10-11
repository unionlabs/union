# Thrift

[Thrift](https://thrift.apache.org/) is a large IDL and transport package from Apache, popularized by Facebook.
Thrift is well-supported in Go kit, for organizations that already have significant Thrift investment.
And using Thrift with Go kit is very simple.

First, define your service in the Thrift IDL.
The [Thrift IDL documentation](https://thrift.apache.org/docs/idl) provides more details.
See [addsvc.thrift](https://github.com/go-kit/examples/blob/master/addsvc/thrift/addsvc.thrift) for an example.
Make sure the Thrift definition matches your service's Go kit (interface) definition.

Next, [download Thrift](https://thrift.apache.org/download) and [install the compiler](https://thrift.apache.org/docs/install/).
On a Mac, you may be able to `brew install thrift`.

Then, compile your service definition, from .thrift to .go.
You'll probably want to specify the package_prefix option to the --gen go flag.
See [THRIFT-3021](https://issues.apache.org/jira/browse/THRIFT-3021) for more details.

```
thrift -r --gen go:package_prefix=github.com/my-org/my-repo/thrift/gen-go/ add.thrift
```

Finally, write a tiny binding from your service definition to the Thrift definition.
It's a straightforward conversion from one domain to the other.
See [thrift.go](https://github.com/go-kit/examples/blob/master/addsvc/pkg/addtransport/thrift.go) for an example.

That's it!
The Thrift binding can be bound to a listener and serve normal Thrift requests.
And within your service, you can use standard Go kit components and idioms.
Unfortunately, setting up a Thrift listener is rather laborious and nonidiomatic in Go.
Fortunately, [addsvc](https://github.com/go-kit/examples/tree/master/addsvc) is a complete working example with Thrift support.
And remember: Go kit services can support multiple transports simultaneously.
