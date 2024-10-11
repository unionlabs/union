---
layout: default
title: Learn More
nav_order: 5
parent: Tutorials
---

# Learn More

## How it works

When the HTTP request arrives at the gRPC-Gateway, it parses the JSON data into a protobuf message. It then makes a normal Go gRPC client request using the parsed protobuf message. The Go gRPC client encodes the protobuf structure into the protobuf binary format and sends it to the gRPC server. The gRPC Server handles the request and returns the response in the protobuf binary format. The Go gRPC client parses it into a protobuf message and returns it to the gRPC-Gateway, which encodes the protobuf message to JSON and returns it to the original client.

## google.api.http

Read more about `google.api.http` in [the source file documentation](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto).

## HTTP and gRPC Transcoding

Read more about HTTP and gRPC Transcoding on [AIP 127](https://google.aip.dev/127).
