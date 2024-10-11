---
layout: default
title: HttpBody Messages
nav_order: 1
parent: Mapping
---

# HttpBody Messages

The [HTTPBody](https://github.com/googleapis/googleapis/blob/master/google/api/httpbody.proto) messages allow a response message to be specified with custom data content and a custom content-type header. The values included in the HTTPBody response will be used verbatim in the returned message from the gateway. Make sure you format your response carefully!

## Example Usage

1. Define your service in gRPC with an httpbody response message

```protobuf
import "google/api/httpbody.proto";
import "google/api/annotations.proto";
import "google/protobuf/empty.proto";

service HttpBodyExampleService {
	rpc HelloWorld(google.protobuf.Empty) returns (google.api.HttpBody) {
		option (google.api.http) = {
			get: "/helloworld"
		};
	}
	rpc Download(google.protobuf.Empty) returns (stream google.api.HttpBody) {
		option (google.api.http) = {
			get: "/download"
		};
	}
}
```

3. Generate gRPC and reverse-proxy stubs and implement your service.

## Example service implementation

```go
func (*HttpBodyExampleService) Helloworld(ctx context.Context, in *empty.Empty) (*httpbody.HttpBody, error) {
	return &httpbody.HttpBody{
		ContentType: "text/html",
		Data:        []byte("Hello World"),
	}, nil
}

func (HttpBodyExampleService) Download(_ *empty.Empty, stream HttpBodyExampleService_DownloadServer) error {
	msgs := []*httpbody.HttpBody{
		{
			ContentType: "text/html",
			Data:        []byte("Hello 1"),
		},
		{
			ContentType: "text/html",
			Data:        []byte("Hello 2"),
		},
	}

	for _, msg := range msgs {
		if err := stream.Send(msg); err != nil {
			return err
		}
	}

	return nil
}
```
