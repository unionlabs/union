## Languages known issues

### Python

- Assert the type required in your server/client since python is able to receive `Bytes array` or `utf8 strings`.

```python
def SayHello(self, request, context):
    # request might be a byte array or a utf8 string

    r = HelloRequest.HelloRequest().GetRootAs(request, 0)
    reply = "Unknown"
    if r.Name():
        reply = r.Name()
    # Issues might happen if type checking isnt present.
    # thus encoding it as a `reply.decode('UTF-8')`
    return build_reply("welcome " + reply.decode('UTF-8'))

```

This can be prevented by making sure all the requests coming to/from python are `Bytes array`

```python
def say_hello(stub, builder):
    hello_request = bytes(builder.Output())
    reply = stub.SayHello(hello_request)
    r = HelloReply.HelloReply.GetRootAs(reply)
    print(r.Message())
```

### Go

- Always requires the `content-type` of the payload to be set to `application/grpc+flatbuffers`

example: `.SayHello(ctx, b, grpc.CallContentSubtype("flatbuffers"))`