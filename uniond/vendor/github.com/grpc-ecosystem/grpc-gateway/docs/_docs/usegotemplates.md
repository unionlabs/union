---
category: documentation
name: Use go templates in protofile comments
---

# Use go templates in protofile comments

Use [Go templates](https://golang.org/pkg/text/template/)
in your protofile comments to allow more advanced documentation such
as:  
* Documentation about fields in the proto objects.  
* Import the content of external files (such as
    [Markdown](https://en.wikipedia.org/wiki/Markdown)).

## How to use it

By default this function is turned off, so if you want to use it you
have to set the `use_go_templates` flag to true inside of the
`swagger_out` flag.

```shell
--swagger_out=use_go_templates=true:.
```

### Example script

Example of a bash script with the `use_go_templates` flag set to true:

```shell
$ protoc -I. \
    --go_out=plugins=grpc:. \
    --grpc-gateway_out=logtostderr=true:. \
    --swagger_out=logtostderr=true,use_go_templates=true:. \
    path/to/my/proto/v1/myproto.proto 
```

### Example proto file

Example of a protofile with Go templates. This proto file imports documentation from another file, `tables.md`:
```protobuf
service LoginService {
    // Login
    // 
    // {{.MethodDescriptorProto.Name}} is a call with the method(s) {{$first := true}}{{range .Bindings}}{{if $first}}{{$first = false}}{{else}}, {{end}}{{.HTTPMethod}}{{end}} within the "{{.Service.Name}}" service.
    // It takes in "{{.RequestType.Name}}" and returns a "{{.ResponseType.Name}}".
    //
    // {{import "tables.md"}}
    rpc Login (LoginRequest) returns (LoginReply) {
        option (google.api.http) = {
            post: "/v1/example/login"
            body: "*"
        };
    }
}

message LoginRequest {
    // The entered username 
    string username = 1;
    // The entered password
    string password = 2;
}

message LoginReply {
    // Whether you have access or not
    bool access = 1;
}
```

The content of `tables.md`:

```markdown
## {{.RequestType.Name}}
| Field ID    | Name      | Type                                                       | Description                  |
| ----------- | --------- | ---------------------------------------------------------  | ---------------------------- | {{range .RequestType.Fields}}
| {{.Number}} | {{.Name}} | {{if eq .Label.String "LABEL_REPEATED"}}[]{{end}}{{.Type}} | {{fieldcomments .Message .}} | {{end}}  
 
## {{.ResponseType.Name}}
| Field ID    | Name      | Type                                                       | Description                  |
| ----------- | --------- | ---------------------------------------------------------- | ---------------------------- | {{range .ResponseType.Fields}}
| {{.Number}} | {{.Name}} | {{if eq .Label.String "LABEL_REPEATED"}}[]{{end}}{{.Type}} | {{fieldcomments .Message .}} | {{end}}  
```

## Swagger output

### SwaggerUI

This is how the swagger file would be rendered in [SwaggerUI](https://swagger.io/tools/swagger-ui/)

![Screenshot swaggerfile in SwaggerUI](https://raw.githubusercontent.com/grpc-ecosystem/grpc-gateway/master/docs/_imgs/gotemplates/swaggerui.png)

### Postman

This is how the swagger file would be rendered in [Postman](https://www.getpostman.com/)

![Screenshot swaggerfile in Postman](https://raw.githubusercontent.com/grpc-ecosystem/grpc-gateway/master/docs/_imgs/gotemplates/postman.png)

For a more detailed example of a protofile that has Go templates enabled,
[see the examples](https://github.com/grpc-ecosystem/grpc-gateway/blob/master/examples/internal/proto/examplepb/use_go_template.proto).
