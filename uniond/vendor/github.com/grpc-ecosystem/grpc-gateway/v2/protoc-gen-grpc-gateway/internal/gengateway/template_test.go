package gengateway

import (
	"strings"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/httprule"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"
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

func compilePath(t *testing.T, path string) httprule.Template {
	parsed, err := httprule.Parse(path)
	if err != nil {
		t.Fatalf("httprule.Parse(%q) failed with %v; want success", path, err)
	}
	return parsed.Compile()
}

func TestApplyTemplateHeader(t *testing.T) {
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth},
	}
	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	file := descriptor.File{
		FileDescriptorProto: &descriptorpb.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			Dependency:  []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType: []*descriptorpb.DescriptorProto{msgdesc},
			Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     descriptorpb.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &descriptorpb.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &descriptorpb.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(false),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth},
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
			FileDescriptorProto: &descriptorpb.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*descriptorpb.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
										Version:  1,
										OpCodes:  []int{0, 0},
										Template: "/v1",
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
		if want := `pattern_ExampleService_Echo_0 = runtime.MustPattern(runtime.NewPattern(1, []int{0, 0}, []string(nil), ""))`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
		if want := `annotatedContext, err = runtime.AnnotateContext(ctx, mux, req, "/example.ExampleService/Echo", runtime.WithHTTPPathPattern("/v1"))`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
	}
}

func TestApplyTemplateRequestWithClientStreaming(t *testing.T) {
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     descriptorpb.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &descriptorpb.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &descriptorpb.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(true),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth},
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
			FileDescriptorProto: &descriptorpb.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*descriptorpb.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
		if want := `pattern_ExampleService_Echo_0 = runtime.MustPattern(runtime.NewPattern(1, []int{0, 0}, []string(nil), ""))`; !strings.Contains(got, want) {
			t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
		}
	}
}

func TestApplyTemplateInProcess(t *testing.T) {
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     descriptorpb.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				TypeName: proto.String("NestedMessage"),
				Number:   proto.Int32(1),
			},
		},
	}
	nesteddesc := &descriptorpb.DescriptorProto{
		Name: proto.String("NestedMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:   proto.String("int32"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_INT32.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("bool"),
				Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   descriptorpb.FieldDescriptorProto_TYPE_BOOL.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	meth := &descriptorpb.MethodDescriptorProto{
		Name:            proto.String("Echo"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("ExampleMessage"),
		ClientStreaming: proto.Bool(true),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth},
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
				`resp, md, err := local_request_ExampleService_Echo_0(annotatedContext, inboundMarshaler, server, req, pathParams)`,
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
			FileDescriptorProto: &descriptorpb.FileDescriptorProto{
				Name:        proto.String("example.proto"),
				Package:     proto.String("example"),
				MessageType: []*descriptorpb.DescriptorProto{msgdesc, nesteddesc},
				Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
	updateMaskDesc := &descriptorpb.FieldDescriptorProto{
		Name:     proto.String("UpdateMask"),
		Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
		Type:     descriptorpb.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
		TypeName: proto.String(".google.protobuf.FieldMask"),
		Number:   proto.Int32(1),
	}
	msgdesc := &descriptorpb.DescriptorProto{
		Name:  proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{updateMaskDesc},
	}
	meth := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth},
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
		FileDescriptorProto: &descriptorpb.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			MessageType: []*descriptorpb.DescriptorProto{msgdesc},
			Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
	msgdesc1 := &descriptorpb.DescriptorProto{
		Name: proto.String("Exam_pleRequest"),
	}
	msgdesc2 := &descriptorpb.DescriptorProto{
		Name: proto.String("example_response"),
	}
	meth1 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("ExampleGe2t"),
		InputType:  proto.String("Exam_pleRequest"),
		OutputType: proto.String("example_response"),
	}
	meth2 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Exampl_ePost"),
		InputType:  proto.String("Exam_pleRequest"),
		OutputType: proto.String("example_response"),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("Example"),
		Method: []*descriptorpb.MethodDescriptorProto{meth1, meth2},
	}
	msg1 := &descriptor.Message{
		DescriptorProto: msgdesc1,
	}
	msg2 := &descriptor.Message{
		DescriptorProto: msgdesc2,
	}
	file := descriptor.File{
		FileDescriptorProto: &descriptorpb.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			Dependency:  []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType: []*descriptorpb.DescriptorProto{msgdesc1, msgdesc2},
			Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
								HTTPMethod: "POST",
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
	if want := `msg, err := client.ExamplEPost(ctx, &protoReq, grpc.Header(&metadata.HeaderMD)`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
	if want := `var protoReq ExamPleRequest`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
	if want := `var protoReq ExampleResponse`; !strings.Contains(got, want) {
		t.Errorf("applyTemplate(%#v) = %s; want to contain %s", file, got, want)
	}
}

func TestDuplicatePathsInSameService(t *testing.T) {
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     descriptorpb.FieldDescriptorProto_TYPE_STRING.Enum(),
				TypeName: proto.String(".google.protobuf.StringValue"),
				Number:   proto.Int32(1),
			},
		},
	}
	meth1 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Echo"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	meth2 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Echo2"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*descriptorpb.MethodDescriptorProto{meth1, meth2},
	}
	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	binding := &descriptor.Binding{
		Index:      1,
		PathTmpl:   compilePath(t, "/v1/example/echo"),
		HTTPMethod: "GET",
		PathParams: nil,
		Body:       nil,
	}
	file := descriptor.File{
		FileDescriptorProto: &descriptorpb.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			MessageType: []*descriptorpb.DescriptorProto{msgdesc},
			Service:     []*descriptorpb.ServiceDescriptorProto{svc},
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
						MethodDescriptorProto: meth1,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings:              []*descriptor.Binding{binding},
					},
					{
						MethodDescriptorProto: meth2,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings:              []*descriptor.Binding{binding},
					},
				},
			},
		},
	}
	_, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
	if err == nil {
		t.Errorf("applyTemplate(%#v) succeeded; want an error", file)
		return
	}
}

func TestDuplicatePathsInDifferentService(t *testing.T) {
	msgdesc := &descriptorpb.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*descriptorpb.FieldDescriptorProto{
			{
				Name:     proto.String("nested"),
				Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:     descriptorpb.FieldDescriptorProto_TYPE_STRING.Enum(),
				TypeName: proto.String(".google.protobuf.StringValue"),
				Number:   proto.Int32(1),
			},
		},
	}
	meth1 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Echo"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	meth2 := &descriptorpb.MethodDescriptorProto{
		Name:       proto.String("Echo2"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}
	svc1 := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleServiceNumberOne"),
		Method: []*descriptorpb.MethodDescriptorProto{meth1, meth2},
	}
	svc2 := &descriptorpb.ServiceDescriptorProto{
		Name:   proto.String("ExampleServiceNumberTwo"),
		Method: []*descriptorpb.MethodDescriptorProto{meth1, meth2},
	}
	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	binding := &descriptor.Binding{
		Index:      1,
		PathTmpl:   compilePath(t, "/v1/example/echo"),
		HTTPMethod: "GET",
		PathParams: nil,
		Body:       nil,
	}
	file := descriptor.File{
		FileDescriptorProto: &descriptorpb.FileDescriptorProto{
			Name:        proto.String("example.proto"),
			Package:     proto.String("example"),
			MessageType: []*descriptorpb.DescriptorProto{msgdesc},
			Service:     []*descriptorpb.ServiceDescriptorProto{svc1, svc2},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/example/example.pb",
			Name: "example_pb",
		},
		Messages: []*descriptor.Message{msg},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc1,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth1,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings:              []*descriptor.Binding{binding},
					},
				},
			},
			{
				ServiceDescriptorProto: svc2,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth2,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings:              []*descriptor.Binding{binding},
					},
				},
			},
		},
	}
	_, err := applyTemplate(param{File: crossLinkFixture(&file), RegisterFuncSuffix: "Handler", AllowPatchFeature: true}, descriptor.NewRegistry())
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed; want success", file)
		return
	}
}
