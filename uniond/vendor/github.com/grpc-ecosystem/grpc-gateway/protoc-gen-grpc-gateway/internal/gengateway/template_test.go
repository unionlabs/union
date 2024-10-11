package gengateway

import (
	"strings"
	"testing"

	"github.com/golang/protobuf/proto"
	protodescriptor "github.com/golang/protobuf/protoc-gen-go/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway/httprule"
)

func crossLinkFixture(f *descriptor.File) *descriptor.File {
	for _, m := range f.Messages {
		m.File = f
	}
	for _, svc := range f.Services {
		svc.File = f
		for _, m := range svc.Methods {
			m.Service = svc
			for _, b := range m.Bindings {
				b.Method = m
				for _, param := range b.PathParams {
					param.Method = m
				}
			}
		}
	}
	return f
}

func TestApplyTemplateHeader(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			Dependency:  []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType: []*protodescriptor.DescriptorProto{msgdesc},
			Service:     []*protodescriptor.ServiceDescriptorProto{svc},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/example/example.pb",
			Name: "example_pb",
		},
		Messages: []*descriptor.Message{msg},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "GET",
								Body:       &descriptor.Body{FieldPath: nil},
							},
						},
					},
				},
			},
		},
	}
	got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want := "package example_pb\n"; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
}

func TestApplyTemplateRequestWithoutClientStreaming(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &protodescriptor.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(false),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	for _, spec := range []struct {
		serverStreaming bool
		sigWant         string
	}{
		{
			serverStreaming: false,
			sigWant:         `func request_ExampleService_Echo_0(ctx context.Context, marshaler runtime.Marshaler, client ExampleServiceClient, req *http.Request, pathParams map[string]string) (proto.Message, runtime.ServerMetadata, error) {`,
		},
		{
			serverStreaming: true,
			sigWant:         `func request_ExampleService_Echo_0(ctx context.Context, marshaler runtime.Marshaler, client ExampleServiceClient, req *http.Request, pathParams map[string]string) (ExampleService_EchoClient, runtime.ServerMetadata, error) {`,
		},
	} {
		meth.ServerStreaming = proto.Bool(spec.serverStreaming)

		msg := &descriptor.Message{
			DescriptorProto: msgdesc,
		}
		nested := &descriptor.Message{
			DescriptorProto: nesteddesc,
		}

		nestedField := &descriptor.Field{
			Message:              msg,
			FieldDescriptorProto: msg.GetField()[0],
		}
		intField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[0],
		}
		boolField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[1],
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*protodescriptor.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*protodescriptor.ServiceDescriptorProto{svc},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: []*descriptor.Message{msg, nested},
			Services: []*descriptor.Service{
				{
					ServiceDescriptorProto: svc,
					Methods: []*descriptor.Method{
						{
							MethodDescriptorProto: meth,
							RequestType:           msg,
							ResponseType:          msg,
							Bindings: []*descriptor.Binding{
								{
									HTTPMethod: "POST",
									PathTmpl: httprule.Template{
										Version: 1,
										OpCodes: []int{0, 0},
									},
									PathParams: []descriptor.Parameter{
										{
											FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
												{
													Name:   "nested",
													Target: nestedField,
												},
												{
													Name:   "int32",
													Target: intField,
												},
											}),
											Target: intField,
										},
									},
									Body: &descriptor.Body{
										FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
											{
												Name:   "nested",
												Target: nestedField,
											},
											{
												Name:   "bool",
												Target: boolField,
											},
										}),
									},
								},
							},
						},
					},
				},
			},
		}
		got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
		if err != nil {
			t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
			return
		}
		if want := spec.sigWant; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `marshaler.NewDecoder(newReader()).Decode(&protoReq.GetNested().Bool)`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `val, ok = pathParams["nested.int32"]`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `protoReq.GetNested().Int32, err = runtime.Int32P(val)`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `func RegisterExampleServiceHandler(ctx context.Context, mux *runtime.ServeMux, conn *grpc.ClientConn) error {`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `pattern_ExampleService_Echo_0 = runtime.MustPattern(runtime.NewPattern(1, []int{0, 0}, []string(nil), "", runtime.AssumeColonVerbOpt(true)))`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
	}
}

func TestApplyTemplateRequestWithClientStreaming(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &protodescriptor.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(true),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	for _, spec := range []struct {
		serverStreaming bool
		sigWant         string
	}{
		{
			serverStreaming: false,
			sigWant:         `func request_ExampleService_Echo_0(ctx context.Context, marshaler runtime.Marshaler, client ExampleServiceClient, req *http.Request, pathParams map[string]string) (proto.Message, runtime.ServerMetadata, error) {`,
		},
		{
			serverStreaming: true,
			sigWant:         `func request_ExampleService_Echo_0(ctx context.Context, marshaler runtime.Marshaler, client ExampleServiceClient, req *http.Request, pathParams map[string]string) (ExampleService_EchoClient, runtime.ServerMetadata, error) {`,
		},
	} {
		meth.ServerStreaming = proto.Bool(spec.serverStreaming)

		msg := &descriptor.Message{
			DescriptorProto: msgdesc,
		}
		nested := &descriptor.Message{
			DescriptorProto: nesteddesc,
		}

		nestedField := &descriptor.Field{
			Message:              msg,
			FieldDescriptorProto: msg.GetField()[0],
		}
		intField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[0],
		}
		boolField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[1],
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*protodescriptor.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*protodescriptor.ServiceDescriptorProto{svc},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: []*descriptor.Message{msg, nested},
			Services: []*descriptor.Service{
				{
					ServiceDescriptorProto: svc,
					Methods: []*descriptor.Method{
						{
							MethodDescriptorProto: meth,
							RequestType:           msg,
							ResponseType:          msg,
							Bindings: []*descriptor.Binding{
								{
									HTTPMethod: "POST",
									PathTmpl: httprule.Template{
										Version: 1,
										OpCodes: []int{0, 0},
									},
									PathParams: []descriptor.Parameter{
										{
											FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
												{
													Name:   "nested",
													Target: nestedField,
												},
												{
													Name:   "int32",
													Target: intField,
												},
											}),
											Target: intField,
										},
									},
									Body: &descriptor.Body{
										FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
											{
												Name:   "nested",
												Target: nestedField,
											},
											{
												Name:   "bool",
												Target: boolField,
											},
										}),
									},
								},
							},
						},
					},
				},
			},
		}
		got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
		if err != nil {
			t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
			return
		}
		if want := spec.sigWant; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `func RegisterExampleServiceHandler(ctx context.Context, mux *runtime.ServeMux, conn *grpc.ClientConn) error {`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `pattern_ExampleService_Echo_0 = runtime.MustPattern(runtime.NewPattern(1, []int{0, 0}, []string(nil), "", runtime.AssumeColonVerbOpt(true)))`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
	}
}

func TestApplyTemplateInProcess(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &protodescriptor.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(true),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	for _, spec := range []struct {
		clientStreaming bool
		serverStreaming bool
		sigWant         []string
	}{
		{
			clientStreaming: false,
			serverStreaming: false,
			sigWant: []string{
				`func local_request_ExampleService_Echo_0(ctx context.Context, marshaler runtime.Marshaler, server ExampleServiceServer, req *http.Request, pathParams map[string]string) (proto.Message, runtime.ServerMetadata, error) {`,
				`resp, md, err := local_request_ExampleService_Echo_0(rctx, inboundMarshaler, server, req, pathParams)`,
			},
		},
		{
			clientStreaming: true,
			serverStreaming: true,
			sigWant: []string{
				`err := status.Error(codes.Unimplemented, "streaming calls are not yet supported in the in-process transport")`,
			},
		},
		{
			clientStreaming: true,
			serverStreaming: false,
			sigWant: []string{
				`err := status.Error(codes.Unimplemented, "streaming calls are not yet supported in the in-process transport")`,
			},
		},
		{
			clientStreaming: false,
			serverStreaming: true,
			sigWant: []string{
				`err := status.Error(codes.Unimplemented, "streaming calls are not yet supported in the in-process transport")`,
			},
		},
	} {
		meth.ClientStreaming = proto.Bool(spec.clientStreaming)
		meth.ServerStreaming = proto.Bool(spec.serverStreaming)

		msg := &descriptor.Message{
			DescriptorProto: msgdesc,
		}
		nested := &descriptor.Message{
			DescriptorProto: nesteddesc,
		}

		nestedField := &descriptor.Field{
			Message:              msg,
			FieldDescriptorProto: msg.GetField()[0],
		}
		intField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[0],
		}
		boolField := &descriptor.Field{
			Message:              nested,
			FieldDescriptorProto: nested.GetField()[1],
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*protodescriptor.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*protodescriptor.ServiceDescriptorProto{svc},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: []*descriptor.Message{msg, nested},
			Services: []*descriptor.Service{
				{
					ServiceDescriptorProto: svc,
					Methods: []*descriptor.Method{
						{
							MethodDescriptorProto: meth,
							RequestType:           msg,
							ResponseType:          msg,
							Bindings: []*descriptor.Binding{
								{
									HTTPMethod: "POST",
									PathTmpl: httprule.Template{
										Version: 1,
										OpCodes: []int{0, 0},
									},
									PathParams: []descriptor.Parameter{
										{
											FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
												{
													Name:   "nested",
													Target: nestedField,
												},
												{
													Name:   "int32",
													Target: intField,
												},
											}),
											Target: intField,
										},
									},
									Body: &descriptor.Body{
										FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
											{
												Name:   "nested",
												Target: nestedField,
											},
											{
												Name:   "bool",
												Target: boolField,
											},
										}),
									},
								},
							},
						},
					},
				},
			},
		}
		got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
		if err != nil {
			t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
			return
		}

		for _, want := range spec.sigWant {
			if !strings.Contains(got, want) {
				t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
			}
		}

		if want := `func RegisterExampleServiceHandlerServer(ctx context.Context, mux *runtime.ServeMux, server ExampleServiceServer) error {`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
	}
}

func TestAllowPatchFeature(t *testing.T) {
	updateMaskDesc := &protodescriptor.FieldDescriptorProto{
		Name:     proto.String("UpdateMask"),
		Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
		Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
		TypeName: proto.String(".google.protobuf.FieldMask"),
		Number:   proto.Int32(1),
	}
	msgdesc := &protodescriptor.DescriptorProto{
		Name:  proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{updateMaskDesc},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	updateMaskField := &descriptor.Field{
		Message:              msg,
		FieldDescriptorProto: updateMaskDesc,
	}
	msg.Fields = append(msg.Fields, updateMaskField)
	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			MessageType: []*protodescriptor.DescriptorProto{msgdesc},
			Service:     []*protodescriptor.ServiceDescriptorProto{svc},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/example/example.pb",
			Name: "example_pb",
		},
		Messages: []*descriptor.Message{msg},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "PATCH",
								Body: &descriptor.Body{FieldPath: descriptor.FieldPath{descriptor.FieldPathComponent{
									Name:   "abe",
									Target: msg.Fields[0],
								}}},
							},
						},
					},
				},
			},
		},
	}
	want := "if protoReq.UpdateMask == nil || len(protoReq.UpdateMask.GetPaths()) == 0 {\n"
	for _, allowPatchFeature := range []bool{true, false} {
		got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: allowPatchFeature}, descriptor.NewRegistry())
		if err != nil {
			t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
			return
		}
		if allowPatchFeature {
			if !strings.Contains(got, want) {
				t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
			}
		} else {
			if strings.Contains(got, want) {
				t.Errorf("applyTemplate(%#v) = %s; want to _not_ contain %s", file, got, want)
			}
		}
	}
}

func TestIdentifierCapitalization(t *testing.T) {
	msgdesc1 := &protodescriptor.DescriptorProto{
		Name: proto.String("Exam_pleRequest"),
	}
	msgdesc2 := &protodescriptor.DescriptorProto{
		Name: proto.String("example_response"),
	}
	meth1 := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("ExampleGe2t"),
		InputType:  proto.String("Exam_pleRequest"),
		OutputType: proto.String("example_response"),
	}
	meth2 := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Exampl_eGet"),
		InputType:  proto.String("Exam_pleRequest"),
		OutputType: proto.String("example_response"),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("Example"),
		Method: []*protodescriptor.MethodDescriptorProto{meth1, meth2},
	}
	msg1 := &descriptor.Message{
		DescriptorProto: msgdesc1,
	}
	msg2 := &descriptor.Message{
		DescriptorProto: msgdesc2,
	}
	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			Dependency:  []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType: []*protodescriptor.DescriptorProto{msgdesc1, msgdesc2},
			Service:     []*protodescriptor.ServiceDescriptorProto{svc},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/example/example.pb",
			Name: "example_pb",
		},
		Messages: []*descriptor.Message{msg1, msg2},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth1,
						RequestType:           msg1,
						ResponseType:          msg1,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "GET",
								Body:       &descriptor.Body{FieldPath: nil},
							},
						},
					},
				},
			},
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth2,
						RequestType:           msg2,
						ResponseType:          msg2,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "GET",
								Body:       &descriptor.Body{FieldPath: nil},
							},
						},
					},
				},
			},
		},
	}

	got, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want := `msg, err := client.ExampleGe2T(ctx, &protoReq, grpc.Header(&metadata.HeaderMD)`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
	if want := `msg, err := client.ExamplEGet(ctx, &protoReq, grpc.Header(&metadata.HeaderMD)`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
	if want := `var protoReq ExamPleRequest`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
	if want := `var protoReq ExampleResponse`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
}
