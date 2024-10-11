---
layout: default
title: Patch feature
nav_order: 2
parent: Mapping
---

# Patch feature

The HTTP PATCH method allows a resource to be partially updated.

If a binding is mapped to patch and the request message has exactly one FieldMask message in it, additional code is rendered for the gateway handler that will populate the FieldMask based on the request body. FieldMask is treated as a regular field by the gateway if the request method is not PATCH, or if the HttpRule body is `"*"`

There are two scenarios:

- The FieldMask is hidden from the REST request as per the
  [Google API design guide](https://cloud.google.com/apis/design/standard_methods#update) (as in the first additional binding in the
  [UpdateV2](https://github.com/grpc-ecosystem/grpc-gateway/blob/370d869f65d1ffb3d07187fb0db238eca2371ce3/examples/internal/proto/examplepb/a_bit_of_everything.proto#L428-L431) example). In this case, the FieldMask is updated from the request body and set in the gRPC request message. 
  - By default this feature is enabled, if you need to disable it, you can use the plugin option `allow_patch_feature=false`. 
  - Note: The same option is supported by the `protoc-gen-openapiv2` plugin.
- The FieldMask is exposed to the REST request (as in the second additional binding in the [UpdateV2](https://github.com/grpc-ecosystem/grpc-gateway/blob/370d869f65d1ffb3d07187fb0db238eca2371ce3/examples/internal/proto/examplepb/a_bit_of_everything.proto#L432-L435) example). For this case, the field mask is left untouched by the gateway.

## Example Usage

1. Create a PATCH request.

   The PATCH request needs to include the message and the update mask.

   ```protobuf
   // UpdateV2Request request for update includes the message and the update mask
   message UpdateV2Request {
    ABitOfEverything abe = 1;
    google.protobuf.FieldMask update_mask = 2;
   }
   ```

2. Define your service in gRPC

   If you want to use PATCH with fieldmask hidden from REST request only include the request message in the body.

   ```protobuf
   rpc UpdateV2(UpdateV2Request) returns (google.protobuf.Empty) {
    option (google.api.http) = {
      put: "/v2/example/a_bit_of_everything/{abe.uuid}"
      body: "abe"
      additional_bindings {
        patch: "/v2/example/a_bit_of_everything/{abe.uuid}"
        body: "abe"
      }
    };
   }
   ```

   If you want to use PATCH with fieldmask exposed to the REST request then include the entire request message.

   ```protobuf
   rpc UpdateV2(UpdateV2Request) returns (google.protobuf.Empty) {
    option (google.api.http) = {
      patch: "/v2a/example/a_bit_of_everything/{abe.uuid}"
      body: "*"
    };
   }
   ```

3. Generate gRPC and reverse-proxy stubs and implement your service.

## cURL examples

In the example below, we will partially update our ABitOfEverything resource by passing only the field we want to change. Since we are using the endpoint with field mask hidden we only need to pass the field we want to change ("string_value") and it will keep everything else in our resource the same.

```sh
$ curl \
  --data '{"stringValue": "strprefix/foo"}' \
  -X PATCH \
  http://address:port/v2/example/a_bit_of_everything/1
```

If we know what fields we want to update then we can use PATCH with field mask approach. For this, we need to pass the resource and the update_mask. Below only the "single_nested" will get updated because that is what we specify in the field_mask.

```sh
$ curl \
  --data '{"abe":{"singleNested":{"amount":457},"stringValue":"some value that will not get updated because not in the field mask"},"updateMask":"singleNested"}}' \
  -X PATCH \
  http://address:port/v2a/example/a_bit_of_everything/1
```
