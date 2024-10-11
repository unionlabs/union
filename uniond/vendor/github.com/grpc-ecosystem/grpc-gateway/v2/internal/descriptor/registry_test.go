package descriptor

import (
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/descriptor/openapiconfig"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/pluginpb"
)

func newGeneratorFromSources(req *pluginpb.CodeGeneratorRequest, sources ...string) (*protogen.Plugin, error) {
	for _, src := range sources {
		var fd descriptorpb.FileDescriptorProto
		if err := prototext.Unmarshal([]byte(src), &fd); err != nil {
			return nil, err
		}
		req.FileToGenerate = append(req.FileToGenerate, fd.GetName())
		req.ProtoFile = append(req.ProtoFile, &fd)
	}
	return protogen.Options{}.New(req)
}

func loadFileWithCodeGeneratorRequest(t *testing.T, reg *Registry, req *pluginpb.CodeGeneratorRequest, sources ...string) []*descriptorpb.FileDescriptorProto {
	t.Helper()
	plugin, err := newGeneratorFromSources(req, sources...)
	if err != nil {
		t.Fatalf("failed to create a generator: %v", err)
	}
	err = reg.LoadFromPlugin(plugin)
	if err != nil {
		t.Fatalf("failed to Registry.LoadFromPlugin(): %v", err)
	}
	return plugin.Request.ProtoFile
}

func loadFile(t *testing.T, reg *Registry, src string) *descriptorpb.FileDescriptorProto {
	t.Helper()
	fds := loadFileWithCodeGeneratorRequest(t, reg, &pluginpb.CodeGeneratorRequest{}, src)
	return fds[0]
}

func TestLoadFile(t *testing.T) {
	reg := NewRegistry()
	fd := loadFile(t, reg, `
		name: 'example.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
		message_type <
			name: 'ExampleMessage'
			field <
				name: 'str'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 1
			>
		>
	`)

	file := reg.files["example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}

	msg, err := reg.LookupMsg("", ".example.ExampleMessage")
	if err != nil {
		t.Errorf("reg.LookupMsg(%q, %q)) failed with %v; want success", "", ".example.ExampleMessage", err)
		return
	}
	if got, want := msg.DescriptorProto, fd.MessageType[0]; got != want {
		t.Errorf("reg.lookupMsg(%q, %q).DescriptorProto = %#v; want %#v", "", ".example.ExampleMessage", got, want)
	}
	if got, want := msg.File, file; got != want {
		t.Errorf("msg.File = %v; want %v", got, want)
	}
	if got := msg.Outers; got != nil {
		t.Errorf("msg.Outers = %v; want %v", got, nil)
	}
	if got, want := len(msg.Fields), 1; got != want {
		t.Errorf("len(msg.Fields) = %d; want %d", got, want)
	} else if got, want := msg.Fields[0].FieldDescriptorProto, fd.MessageType[0].Field[0]; got != want {
		t.Errorf("msg.Fields[0].FieldDescriptorProto = %v; want %v", got, want)
	} else if got, want := msg.Fields[0].Message, msg; got != want {
		t.Errorf("msg.Fields[0].Message = %v; want %v", got, want)
	}

	if got, want := len(file.Messages), 1; got != want {
		t.Errorf("file.Meeesages = %#v; want %#v", file.Messages, []*Message{msg})
	}
	if got, want := file.Messages[0], msg; got != want {
		t.Errorf("file.Meeesages[0] = %v; want %v", got, want)
	}
}

func TestLoadFileNestedPackage(t *testing.T) {
	reg := NewRegistry()
	loadFile(t, reg, `
		name: 'example.proto'
		package: 'example.nested.nested2'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.nested.nested2' >
	`)

	file := reg.files["example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.nested.nested2", Name: "example_nested_nested2"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadFileWithDir(t *testing.T) {
	reg := NewRegistry()
	loadFile(t, reg, `
		name: 'path/to/example.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`)

	file := reg.files["path/to/example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadFileWithoutPackage(t *testing.T) {
	reg := NewRegistry()
	loadFile(t, reg, `
		name: 'path/to/example_file.proto'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example_file' >
	`)

	file := reg.files["path/to/example_file.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example_file", Name: "example_file"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadFileWithMapping(t *testing.T) {
	reg := NewRegistry()
	loadFileWithCodeGeneratorRequest(t, reg, &pluginpb.CodeGeneratorRequest{
		Parameter: proto.String("Mpath/to/example.proto=example.com/proj/example/proto"),
	}, `
		name: 'path/to/example.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`)

	file := reg.files["path/to/example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "example.com/proj/example/proto", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadFileWithPackageNameCollision(t *testing.T) {
	reg := NewRegistry()
	loadFile(t, reg, `
		name: 'path/to/another.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`)
	loadFile(t, reg, `
		name: 'path/to/example.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`)
	if err := reg.ReserveGoPackageAlias("ioutil", "io/ioutil"); err != nil {
		t.Fatalf("reg.ReserveGoPackageAlias(%q) failed with %v; want success", "ioutil", err)
	}
	loadFile(t, reg, `
		name: 'path/to/ioutil.proto'
		package: 'ioutil'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/ioutil' >
	`)

	file := reg.files["path/to/another.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "path/to/another.proto")
		return
	}
	wantPkg := GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}

	file = reg.files["path/to/example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "path/to/example.proto")
		return
	}
	wantPkg = GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example", Name: "example", Alias: ""}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}

	file = reg.files["path/to/ioutil.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "path/to/ioutil.proto")
		return
	}
	wantPkg = GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway/runtime/internal/ioutil", Name: "ioutil", Alias: "ioutil_0"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadFileWithIdenticalGoPkg(t *testing.T) {
	reg := NewRegistry()
	loadFileWithCodeGeneratorRequest(t, reg, &pluginpb.CodeGeneratorRequest{
		Parameter: proto.String("Mpath/to/another.proto=example.com/example,Mpath/to/example.proto=example.com/example"),
	}, `
		name: 'path/to/another.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`, `
		name: 'path/to/example.proto'
		package: 'example'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
	`)

	file := reg.files["path/to/example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "example.com/example", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}

	file = reg.files["path/to/another.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg = GoPackage{Path: "example.com/example", Name: "example"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

// TestLookupMsgWithoutPackage tests a case when there is no "package" directive.
// In Go, it is required to have a generated package so we rely on
// google.golang.org/protobuf/compiler/protogen to provide it.
func TestLookupMsgWithoutPackage(t *testing.T) {
	reg := NewRegistry()
	fd := loadFile(t, reg, `
		name: 'example.proto'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
		message_type <
			name: 'ExampleMessage'
			field <
				name: 'str'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 1
			>
		>
	`)

	msg, err := reg.LookupMsg("", ".ExampleMessage")
	if err != nil {
		t.Errorf("reg.LookupMsg(%q, %q)) failed with %v; want success", "", ".ExampleMessage", err)
		return
	}
	if got, want := msg.DescriptorProto, fd.MessageType[0]; got != want {
		t.Errorf("reg.lookupMsg(%q, %q).DescriptorProto = %#v; want %#v", "", ".ExampleMessage", got, want)
	}
}

func TestLookupMsgWithNestedPackage(t *testing.T) {
	reg := NewRegistry()
	fd := loadFile(t, reg, `
		name: 'example.proto'
		package: 'nested.nested2.mypackage'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
		message_type <
			name: 'ExampleMessage'
			field <
				name: 'str'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 1
			>
		>
	`)

	for _, name := range []string{
		"nested.nested2.mypackage.ExampleMessage",
		"nested2.mypackage.ExampleMessage",
		"mypackage.ExampleMessage",
		"ExampleMessage",
	} {
		msg, err := reg.LookupMsg("nested.nested2.mypackage", name)
		if err != nil {
			t.Errorf("reg.LookupMsg(%q, %q)) failed with %v; want success", ".nested.nested2.mypackage", name, err)
			return
		}
		if got, want := msg.DescriptorProto, fd.MessageType[0]; got != want {
			t.Errorf("reg.lookupMsg(%q, %q).DescriptorProto = %#v; want %#v", ".nested.nested2.mypackage", name, got, want)
		}
	}

	for _, loc := range []string{
		".nested.nested2.mypackage",
		"nested.nested2.mypackage",
		".nested.nested2",
		"nested.nested2",
		".nested",
		"nested",
		".",
		"",
		"somewhere.else",
	} {
		name := "nested.nested2.mypackage.ExampleMessage"
		msg, err := reg.LookupMsg(loc, name)
		if err != nil {
			t.Errorf("reg.LookupMsg(%q, %q)) failed with %v; want success", loc, name, err)
			return
		}
		if got, want := msg.DescriptorProto, fd.MessageType[0]; got != want {
			t.Errorf("reg.lookupMsg(%q, %q).DescriptorProto = %#v; want %#v", loc, name, got, want)
		}
	}

	for _, loc := range []string{
		".nested.nested2.mypackage",
		"nested.nested2.mypackage",
		".nested.nested2",
		"nested.nested2",
		".nested",
		"nested",
	} {
		name := "nested2.mypackage.ExampleMessage"
		msg, err := reg.LookupMsg(loc, name)
		if err != nil {
			t.Errorf("reg.LookupMsg(%q, %q)) failed with %v; want success", loc, name, err)
			return
		}
		if got, want := msg.DescriptorProto, fd.MessageType[0]; got != want {
			t.Errorf("reg.lookupMsg(%q, %q).DescriptorProto = %#v; want %#v", loc, name, got, want)
		}
	}
}

func TestLoadWithInconsistentTargetPackage(t *testing.T) {
	for _, spec := range []struct {
		req        string
		consistent bool
	}{
		// root package, explicit go package
		{
			req: `
				file_to_generate: 'a.proto'
				file_to_generate: 'b.proto'
				proto_file <
					name: 'a.proto'
					options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.foo' >
					message_type < name: 'A' >
					service <
						name: "AService"
						method <
							name: "Meth"
							input_type: "A"
							output_type: "A"
							options <
								[google.api.http] < post: "/v1/a" body: "*" >
							>
						>
					>
				>
				proto_file <
					name: 'b.proto'
					options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.foo' >
					message_type < name: 'B' >
					service <
						name: "BService"
						method <
							name: "Meth"
							input_type: "B"
							output_type: "B"
							options <
								[google.api.http] < post: "/v1/b" body: "*" >
							>
						>
					>
				>
			`,
			consistent: true,
		},
		// named package, explicit go package
		{
			req: `
				file_to_generate: 'a.proto'
				file_to_generate: 'b.proto'
				proto_file <
					name: 'a.proto'
					package: 'example.foo'
					options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.foo' >
					message_type < name: 'A' >
					service <
						name: "AService"
						method <
							name: "Meth"
							input_type: "A"
							output_type: "A"
							options <
								[google.api.http] < post: "/v1/a" body: "*" >
							>
						>
					>
				>
				proto_file <
					name: 'b.proto'
					package: 'example.foo'
					options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example.foo' >
					message_type < name: 'B' >
					service <
						name: "BService"
						method <
							name: "Meth"
							input_type: "B"
							output_type: "B"
							options <
								[google.api.http] < post: "/v1/b" body: "*" >
							>
						>
					>
				>
			`,
			consistent: true,
		},
	} {
		var req pluginpb.CodeGeneratorRequest
		if err := prototext.Unmarshal([]byte(spec.req), &req); err != nil {
			t.Fatalf("proto.UnmarshalText(%s, &file) failed with %v; want success", spec.req, err)
		}
		_, err := newGeneratorFromSources(&req)
		if got, want := err == nil, spec.consistent; got != want {
			if want {
				t.Errorf("reg.Load(%s) failed with %v; want success", spec.req, err)
				continue
			}
			t.Errorf("reg.Load(%s) succeeded; want an package inconsistency error", spec.req)
		}
	}
}

func TestLoadOverriddenPackageName(t *testing.T) {
	reg := NewRegistry()
	loadFile(t, reg, `
		name: 'example.proto'
		package: 'example'
		options < go_package: 'example.com/xyz;pb' >
	`)
	file := reg.files["example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "example.com/xyz", Name: "pb"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestLoadWithStandalone(t *testing.T) {
	reg := NewRegistry()
	reg.SetStandalone(true)
	loadFile(t, reg, `
		name: 'example.proto'
		package: 'example'
		options < go_package: 'example.com/xyz;pb' >
	`)
	file := reg.files["example.proto"]
	if file == nil {
		t.Errorf("reg.files[%q] = nil; want non-nil", "example.proto")
		return
	}
	wantPkg := GoPackage{Path: "example.com/xyz", Name: "pb", Alias: "extPb"}
	if got, want := file.GoPkg, wantPkg; got != want {
		t.Errorf("file.GoPkg = %#v; want %#v", got, want)
	}
}

func TestUnboundExternalHTTPRules(t *testing.T) {
	reg := NewRegistry()
	methodName := ".example.ExampleService.Echo"
	reg.AddExternalHTTPRule(methodName, nil)
	assertStringSlice(t, "unbound external HTTP rules", reg.UnboundExternalHTTPRules(), []string{methodName})
	loadFile(t, reg, `
		name: "path/to/example.proto",
		package: "example"
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
		message_type <
			name: "StringMessage"
			field <
				name: "string"
				number: 1
				label: LABEL_OPTIONAL
				type: TYPE_STRING
			>
		>
		service <
			name: "ExampleService"
			method <
				name: "Echo"
				input_type: "StringMessage"
				output_type: "StringMessage"
			>
		>
	`)
	assertStringSlice(t, "unbound external HTTP rules", reg.UnboundExternalHTTPRules(), []string{})
}

func TestRegisterOpenAPIOptions(t *testing.T) {
	codeReqText := `file_to_generate: 'a.proto'
	proto_file <
		name: 'a.proto'
		package: 'example.foo'
		options < go_package: 'github.com/grpc-ecosystem/grpc-gateway/runtime/internal/example' >
		message_type <
			name: 'ExampleMessage'
			field <
				name: 'str'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 1
			>
		>
		service <
			name: "AService"
			method <
				name: "Meth"
				input_type: "ExampleMessage"
				output_type: "ExampleMessage"
				options <
					[google.api.http] < post: "/v1/a" body: "*" >
				>
			>
		>
	>
	`
	var codeReq pluginpb.CodeGeneratorRequest
	if err := prototext.Unmarshal([]byte(codeReqText), &codeReq); err != nil {
		t.Fatalf("proto.UnmarshalText(%s, &file) failed with %v; want success", codeReqText, err)
	}

	for _, tcase := range []struct {
		options   *openapiconfig.OpenAPIOptions
		shouldErr bool
		desc      string
	}{
		{
			desc: "handle nil options",
		},
		{
			desc: "successfully add options if referenced entity exists",
			options: &openapiconfig.OpenAPIOptions{
				File: []*openapiconfig.OpenAPIFileOption{
					{
						File: "a.proto",
					},
				},
				Method: []*openapiconfig.OpenAPIMethodOption{
					{
						Method: "example.foo.AService.Meth",
					},
				},
				Message: []*openapiconfig.OpenAPIMessageOption{
					{
						Message: "example.foo.ExampleMessage",
					},
				},
				Service: []*openapiconfig.OpenAPIServiceOption{
					{
						Service: "example.foo.AService",
					},
				},
				Field: []*openapiconfig.OpenAPIFieldOption{
					{
						Field: "example.foo.ExampleMessage.str",
					},
				},
			},
		},
		{
			desc: "reject fully qualified names with leading \".\"",
			options: &openapiconfig.OpenAPIOptions{
				File: []*openapiconfig.OpenAPIFileOption{
					{
						File: "a.proto",
					},
				},
				Method: []*openapiconfig.OpenAPIMethodOption{
					{
						Method: ".example.foo.AService.Meth",
					},
				},
				Message: []*openapiconfig.OpenAPIMessageOption{
					{
						Message: ".example.foo.ExampleMessage",
					},
				},
				Service: []*openapiconfig.OpenAPIServiceOption{
					{
						Service: ".example.foo.AService",
					},
				},
				Field: []*openapiconfig.OpenAPIFieldOption{
					{
						Field: ".example.foo.ExampleMessage.str",
					},
				},
			},
			shouldErr: true,
		},
		{
			desc: "error if file does not exist",
			options: &openapiconfig.OpenAPIOptions{
				File: []*openapiconfig.OpenAPIFileOption{
					{
						File: "b.proto",
					},
				},
			},
			shouldErr: true,
		},
		{
			desc: "error if method does not exist",
			options: &openapiconfig.OpenAPIOptions{
				Method: []*openapiconfig.OpenAPIMethodOption{
					{
						Method: "example.foo.AService.Meth2",
					},
				},
			},
			shouldErr: true,
		},
		{
			desc: "error if message does not exist",
			options: &openapiconfig.OpenAPIOptions{
				Message: []*openapiconfig.OpenAPIMessageOption{
					{
						Message: "example.foo.NonexistentMessage",
					},
				},
			},
			shouldErr: true,
		},
		{
			desc: "error if service does not exist",
			options: &openapiconfig.OpenAPIOptions{
				Service: []*openapiconfig.OpenAPIServiceOption{
					{
						Service: "example.foo.AService1",
					},
				},
			},
			shouldErr: true,
		},
		{
			desc: "error if field does not exist",
			options: &openapiconfig.OpenAPIOptions{
				Field: []*openapiconfig.OpenAPIFieldOption{
					{
						Field: "example.foo.ExampleMessage.str1",
					},
				},
			},
			shouldErr: true,
		},
	} {
		t.Run(tcase.desc, func(t *testing.T) {
			reg := NewRegistry()
			loadFileWithCodeGeneratorRequest(t, reg, &codeReq)
			err := reg.RegisterOpenAPIOptions(tcase.options)
			if (err != nil) != tcase.shouldErr {
				t.Fatalf("got unexpected error: %s", err)
			}
		})
	}
}

func assertStringSlice(t *testing.T, message string, got, want []string) {
	if len(got) != len(want) {
		t.Errorf("%s = %#v len(%d); want %#v len(%d)", message, got, len(got), want, len(want))
	}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("%s[%d] = %#v; want %#v", message, i, got[i], want[i])
		}
	}
}
