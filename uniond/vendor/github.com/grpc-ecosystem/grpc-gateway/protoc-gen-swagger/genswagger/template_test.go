package genswagger

import (
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"reflect"
	"strings"
	"testing"

	"github.com/golang/protobuf/proto"
	protodescriptor "github.com/golang/protobuf/protoc-gen-go/descriptor"
	plugin "github.com/golang/protobuf/protoc-gen-go/plugin"
	structpb "github.com/golang/protobuf/ptypes/struct"
	"github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway/httprule"
	swagger_options "github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger/options"
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

func reqFromFile(f *descriptor.File) *plugin.CodeGeneratorRequest {
	return &plugin.CodeGeneratorRequest{
		ProtoFile: []*protodescriptor.FileDescriptorProto{
			f.FileDescriptorProto,
		},
		FileToGenerate: []string{f.GetName()},
	}
}

func TestMessageToQueryParametersWithEnumAsInt(t *testing.T) {
	type test struct {
		MsgDescs []*protodescriptor.DescriptorProto
		Message  string
		Params   []swaggerParameterObject
	}

	tests := []test{
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:   proto.String("a"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(1),
						},
						{
							Name:   proto.String("b"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_DOUBLE.Enum(),
							Number: proto.Int32(2),
						},
						{
							Name:   proto.String("c"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Label:  protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
							Number: proto.Int32(3),
						},
					},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "a",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "b",
					In:       "query",
					Required: false,
					Type:     "number",
					Format:   "double",
				},
				swaggerParameterObject{
					Name:             "c",
					In:               "query",
					Required:         false,
					Type:             "array",
					CollectionFormat: "multi",
				},
			},
		},
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.Nested"),
							Number:   proto.Int32(1),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("Nested"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:   proto.String("a"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(1),
						},
						{
							Name:     proto.String("deep"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.Nested.DeepNested"),
							Number:   proto.Int32(2),
						},
					},
					NestedType: []*protodescriptor.DescriptorProto{{
						Name: proto.String("DeepNested"),
						Field: []*protodescriptor.FieldDescriptorProto{
							{
								Name:   proto.String("b"),
								Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
								Number: proto.Int32(1),
							},
							{
								Name:     proto.String("c"),
								Type:     protodescriptor.FieldDescriptorProto_TYPE_ENUM.Enum(),
								TypeName: proto.String(".example.Nested.DeepNested.DeepEnum"),
								Number:   proto.Int32(2),
							},
						},
						EnumType: []*protodescriptor.EnumDescriptorProto{
							{
								Name: proto.String("DeepEnum"),
								Value: []*protodescriptor.EnumValueDescriptorProto{
									{Name: proto.String("FALSE"), Number: proto.Int32(0)},
									{Name: proto.String("TRUE"), Number: proto.Int32(1)},
								},
							},
						},
					}},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "nested.a",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "nested.deep.b",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "nested.deep.c",
					In:       "query",
					Required: false,
					Type:     "integer",
					Enum:     []string{"0", "1"},
					Default:  "0",
				},
			},
		},
	}

	for _, test := range tests {
		reg := descriptor.NewRegistry()
		reg.SetEnumsAsInts(true)
		msgs := []*descriptor.Message{}
		for _, msgdesc := range test.MsgDescs {
			msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType:    test.MsgDescs,
				Service:        []*protodescriptor.ServiceDescriptorProto{},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: msgs,
		}
		reg.Load(&plugin.CodeGeneratorRequest{
			ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
		})

		message, err := reg.LookupMsg("", ".example."+test.Message)
		if err != nil {
			t.Fatalf("failed to lookup message: %s", err)
		}
		params, err := messageToQueryParameters(message, reg, []descriptor.Parameter{}, nil)
		if err != nil {
			t.Fatalf("failed to convert message to query parameters: %s", err)
		}
		// avoid checking Items for array types
		for i := range params {
			params[i].Items = nil
		}
		if !reflect.DeepEqual(params, test.Params) {
			t.Errorf("expected %v, got %v", test.Params, params)
		}
	}
}

func TestMessageToQueryParameters(t *testing.T) {
	type test struct {
		MsgDescs []*protodescriptor.DescriptorProto
		Message  string
		Params   []swaggerParameterObject
	}

	tests := []test{
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:   proto.String("a"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(1),
						},
						{
							Name:   proto.String("b"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_DOUBLE.Enum(),
							Number: proto.Int32(2),
						},
						{
							Name:   proto.String("c"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Label:  protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
							Number: proto.Int32(3),
						},
					},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "a",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "b",
					In:       "query",
					Required: false,
					Type:     "number",
					Format:   "double",
				},
				swaggerParameterObject{
					Name:             "c",
					In:               "query",
					Required:         false,
					Type:             "array",
					CollectionFormat: "multi",
				},
			},
		},
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.Nested"),
							Number:   proto.Int32(1),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("Nested"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:   proto.String("a"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(1),
						},
						{
							Name:     proto.String("deep"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.Nested.DeepNested"),
							Number:   proto.Int32(2),
						},
					},
					NestedType: []*protodescriptor.DescriptorProto{{
						Name: proto.String("DeepNested"),
						Field: []*protodescriptor.FieldDescriptorProto{
							{
								Name:   proto.String("b"),
								Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
								Number: proto.Int32(1),
							},
							{
								Name:     proto.String("c"),
								Type:     protodescriptor.FieldDescriptorProto_TYPE_ENUM.Enum(),
								TypeName: proto.String(".example.Nested.DeepNested.DeepEnum"),
								Number:   proto.Int32(2),
							},
						},
						EnumType: []*protodescriptor.EnumDescriptorProto{
							{
								Name: proto.String("DeepEnum"),
								Value: []*protodescriptor.EnumValueDescriptorProto{
									{Name: proto.String("FALSE"), Number: proto.Int32(0)},
									{Name: proto.String("TRUE"), Number: proto.Int32(1)},
								},
							},
						},
					}},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "nested.a",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "nested.deep.b",
					In:       "query",
					Required: false,
					Type:     "string",
				},
				swaggerParameterObject{
					Name:     "nested.deep.c",
					In:       "query",
					Required: false,
					Type:     "string",
					Enum:     []string{"FALSE", "TRUE"},
					Default:  "FALSE",
				},
			},
		},
	}

	for _, test := range tests {
		reg := descriptor.NewRegistry()
		msgs := []*descriptor.Message{}
		for _, msgdesc := range test.MsgDescs {
			msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType:    test.MsgDescs,
				Service:        []*protodescriptor.ServiceDescriptorProto{},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: msgs,
		}
		reg.Load(&plugin.CodeGeneratorRequest{
			ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
		})

		message, err := reg.LookupMsg("", ".example."+test.Message)
		if err != nil {
			t.Fatalf("failed to lookup message: %s", err)
		}
		params, err := messageToQueryParameters(message, reg, []descriptor.Parameter{}, nil)
		if err != nil {
			t.Fatalf("failed to convert message to query parameters: %s", err)
		}
		// avoid checking Items for array types
		for i := range params {
			params[i].Items = nil
		}
		if !reflect.DeepEqual(params, test.Params) {
			t.Errorf("expected %v, got %v", test.Params, params)
		}
	}
}

// TestMessagetoQueryParametersNoRecursive, is a check that cyclical references between messages
//  are not falsely detected given previous known edge-cases.
func TestMessageToQueryParametersNoRecursive(t *testing.T) {
	type test struct {
		MsgDescs []*protodescriptor.DescriptorProto
		Message  string
	}

	tests := []test{
		// First test:
		// Here is a message that has two of another message adjacent to one another in a nested message.
		// There is no loop but this was previouly falsely flagged as a cycle.
		// Example proto:
		// message NonRecursiveMessage {
		//      string field = 1;
		// }
		// message BaseMessage {
		//      NonRecursiveMessage first = 1;
		//      NonRecursiveMessage second = 2;
		// }
		// message QueryMessage {
		//      BaseMessage first = 1;
		//      string second = 2;
		// }
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("QueryMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("first"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.BaseMessage"),
							Number:   proto.Int32(1),
						},
						{
							Name:   proto.String("second"),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(2),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("BaseMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("first"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.NonRecursiveMessage"),
							Number:   proto.Int32(1),
						},
						{
							Name:     proto.String("second"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.NonRecursiveMessage"),
							Number:   proto.Int32(2),
						},
					},
				},
				// Note there is no recursive nature to this message
				&protodescriptor.DescriptorProto{
					Name: proto.String("NonRecursiveMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name: proto.String("field"),
							//Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
							Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number: proto.Int32(1),
						},
					},
				},
			},
			Message: "QueryMessage",
		},
	}

	for _, test := range tests {
		reg := descriptor.NewRegistry()
		msgs := []*descriptor.Message{}
		for _, msgdesc := range test.MsgDescs {
			msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType:    test.MsgDescs,
				Service:        []*protodescriptor.ServiceDescriptorProto{},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: msgs,
		}
		reg.Load(&plugin.CodeGeneratorRequest{
			ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
		})

		message, err := reg.LookupMsg("", ".example."+test.Message)
		if err != nil {
			t.Fatalf("failed to lookup message: %s", err)
		}

		_, err = messageToQueryParameters(message, reg, []descriptor.Parameter{}, nil)
		if err != nil {
			t.Fatalf("No recursion error should be thrown: %s", err)
		}
	}
}

// TestMessagetoQueryParametersRecursive, is a check that cyclical references between messages
//  are handled gracefully. The goal is to insure that attempts to add messages with cyclical
//  references to query-parameters returns an error message.
func TestMessageToQueryParametersRecursive(t *testing.T) {
	type test struct {
		MsgDescs []*protodescriptor.DescriptorProto
		Message  string
	}

	tests := []test{
		// First test:
		// Here we test that a message that references it self through a field will return an error.
		// Example proto:
		// message DirectRecursiveMessage {
		//      DirectRecursiveMessage nested = 1;
		// }
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("DirectRecursiveMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.DirectRecursiveMessage"),
							Number:   proto.Int32(1),
						},
					},
				},
			},
			Message: "DirectRecursiveMessage",
		},
		// Second test:
		// Here we test that a cycle through multiple messages is detected and that an error is returned.
		// Sample:
		// message Root { NodeMessage nested = 1; }
		// message NodeMessage { CycleMessage nested = 1; }
		// message CycleMessage { Root nested = 1; }
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("RootMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.NodeMessage"),
							Number:   proto.Int32(1),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("NodeMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.CycleMessage"),
							Number:   proto.Int32(1),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("CycleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("nested"),
							Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.RootMessage"),
							Number:   proto.Int32(1),
						},
					},
				},
			},
			Message: "RootMessage",
		},
	}

	for _, test := range tests {
		reg := descriptor.NewRegistry()
		msgs := []*descriptor.Message{}
		for _, msgdesc := range test.MsgDescs {
			msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType:    test.MsgDescs,
				Service:        []*protodescriptor.ServiceDescriptorProto{},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: msgs,
		}
		reg.Load(&plugin.CodeGeneratorRequest{
			ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
		})

		message, err := reg.LookupMsg("", ".example."+test.Message)
		if err != nil {
			t.Fatalf("failed to lookup message: %s", err)
		}
		_, err = messageToQueryParameters(message, reg, []descriptor.Parameter{}, nil)
		if err == nil {
			t.Fatalf("It should not be allowed to have recursive query parameters")
		}
	}
}

func TestMessageToQueryParametersWithJsonName(t *testing.T) {
	type test struct {
		MsgDescs []*protodescriptor.DescriptorProto
		Message  string
		Params   []swaggerParameterObject
	}

	tests := []test{
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("test_field_a"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number:   proto.Int32(1),
							JsonName: proto.String("testFieldA"),
						},
					},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "testFieldA",
					In:       "query",
					Required: false,
					Type:     "string",
				},
			},
		},
		{
			MsgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{
					Name: proto.String("SubMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("test_field_a"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							Number:   proto.Int32(1),
							JsonName: proto.String("testFieldA"),
						},
					},
				},
				&protodescriptor.DescriptorProto{
					Name: proto.String("ExampleMessage"),
					Field: []*protodescriptor.FieldDescriptorProto{
						{
							Name:     proto.String("sub_message"),
							Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
							TypeName: proto.String(".example.SubMessage"),
							Number:   proto.Int32(1),
							JsonName: proto.String("subMessage"),
						},
					},
				},
			},
			Message: "ExampleMessage",
			Params: []swaggerParameterObject{
				swaggerParameterObject{
					Name:     "subMessage.testFieldA",
					In:       "query",
					Required: false,
					Type:     "string",
				},
			},
		},
	}

	for _, test := range tests {
		reg := descriptor.NewRegistry()
		reg.SetUseJSONNamesForFields(true)
		msgs := []*descriptor.Message{}
		for _, msgdesc := range test.MsgDescs {
			msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
		}
		file := descriptor.File{
			FileDescriptorProto: &protodescriptor.FileDescriptorProto{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType:    test.MsgDescs,
				Service:        []*protodescriptor.ServiceDescriptorProto{},
			},
			GoPkg: descriptor.GoPackage{
				Path: "example.com/path/to/example/example.pb",
				Name: "example_pb",
			},
			Messages: msgs,
		}
		reg.Load(&plugin.CodeGeneratorRequest{
			ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
		})

		message, err := reg.LookupMsg("", ".example."+test.Message)
		if err != nil {
			t.Fatalf("failed to lookup message: %s", err)
		}
		params, err := messageToQueryParameters(message, reg, []descriptor.Parameter{}, nil)
		if err != nil {
			t.Fatalf("failed to convert message to query parameters: %s", err)
		}
		if !reflect.DeepEqual(params, test.Params) {
			t.Errorf("expected %v, got %v", test.Params, params)
		}
	}
}

func TestApplyTemplateSimple(t *testing.T) {
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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			Dependency:     []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo", // TODO(achew22): Figure out what this should really be
								},
							},
						},
					},
				},
			},
		},
	}
	reg := descriptor.NewRegistry()
	fileCL := crossLinkFixture(&file)
	err := reg.Load(reqFromFile(fileCL))
	if err != nil {
		t.Errorf("reg.Load(%#v) failed with %v; want success", file, err)
		return
	}
	result, err := applyTemplate(param{File: fileCL, reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want, is, name := "2.0", result.Swagger, "Swagger"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := "", result.BasePath, "BasePath"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := ([]string)(nil), result.Schemes, "Schemes"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := []string{"application/json"}, result.Consumes, "Consumes"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := []string{"application/json"}, result.Produces, "Produces"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func TestApplyTemplateMultiService(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
	}

	// Create two services that have the same method name. We will test that the
	// operation IDs are different
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}
	svc2 := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("OtherService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}

	msg := &descriptor.Message{
		DescriptorProto: msgdesc,
	}
	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			Dependency:     []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo",
								},
							},
						},
					},
				},
			},
			{
				ServiceDescriptorProto: svc2,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth,
						RequestType:           msg,
						ResponseType:          msg,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "GET",
								Body:       &descriptor.Body{FieldPath: nil},
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/ping",
								},
							},
						},
					},
				},
			},
		},
	}
	reg := descriptor.NewRegistry()
	fileCL := crossLinkFixture(&file)
	err := reg.Load(reqFromFile(fileCL))
	if err != nil {
		t.Errorf("reg.Load(%#v) failed with %v; want success", file, err)
		return
	}
	result, err := applyTemplate(param{File: fileCL, reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}

	// Check that the two services have unique operation IDs even though they
	// have the same method name.
	if want, is := "ExampleService_Example", result.Paths["/v1/echo"].Get.OperationID; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).Paths[0].Get.OperationID = %s want to be %s", file, is, want)
	}
	if want, is := "OtherService_Example", result.Paths["/v1/ping"].Get.OperationID; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).Paths[0].Get.OperationID = %s want to be %s", file, is, want)
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func TestApplyTemplateOverrideOperationID(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
		Options:    &protodescriptor.MethodOptions{},
	}
	swaggerOperation := swagger_options.Operation{
		OperationId: "MyExample",
	}
	if err := proto.SetExtension(proto.Message(meth.Options), swagger_options.E_Openapiv2Operation, &swaggerOperation); err != nil {
		t.Fatalf("proto.SetExtension(MethodDescriptorProto.Options) failed: %v", err)
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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			Dependency:     []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo", // TODO(achew22): Figure out what this should really be
								},
							},
						},
					},
				},
			},
		},
	}

	reg := descriptor.NewRegistry()
	fileCL := crossLinkFixture(&file)
	err := reg.Load(reqFromFile(fileCL))
	if err != nil {
		t.Errorf("reg.Load(%#v) failed with %v; want success", file, err)
		return
	}
	result, err := applyTemplate(param{File: fileCL, reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want, is := "MyExample", result.Paths["/v1/echo"].Get.OperationID; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).Paths[0].Get.OperationID = %s want to be %s", file, is, want)
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func TestApplyTemplateExtensions(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
		Options:    &protodescriptor.MethodOptions{},
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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			Dependency:     []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
			Options:        &protodescriptor.FileOptions{},
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
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo", // TODO(achew22): Figure out what this should really be
								},
							},
						},
					},
				},
			},
		},
	}
	swagger := swagger_options.Swagger{
		Info: &swagger_options.Info{
			Title: "test",
			Extensions: map[string]*structpb.Value{
				"x-info-extension": &structpb.Value{Kind: &structpb.Value_StringValue{StringValue: "bar"}},
			},
		},
		Extensions: map[string]*structpb.Value{
			"x-foo": &structpb.Value{Kind: &structpb.Value_StringValue{StringValue: "bar"}},
			"x-bar": &structpb.Value{Kind: &structpb.Value_ListValue{ListValue: &structpb.ListValue{
				Values: []*structpb.Value{{Kind: &structpb.Value_StringValue{StringValue: "baz"}}},
			}}},
		},
		SecurityDefinitions: &swagger_options.SecurityDefinitions{
			Security: map[string]*swagger_options.SecurityScheme{
				"somescheme": &swagger_options.SecurityScheme{
					Extensions: map[string]*structpb.Value{
						"x-security-baz": &structpb.Value{Kind: &structpb.Value_BoolValue{BoolValue: true}},
					},
				},
			},
		},
	}
	if err := proto.SetExtension(proto.Message(file.FileDescriptorProto.Options), swagger_options.E_Openapiv2Swagger, &swagger); err != nil {
		t.Fatalf("proto.SetExtension(FileDescriptorProto.Options) failed: %v", err)
	}

	swaggerOperation := swagger_options.Operation{
		Responses: map[string]*swagger_options.Response{
			"200": &swagger_options.Response{
				Extensions: map[string]*structpb.Value{
					"x-resp-id": &structpb.Value{Kind: &structpb.Value_StringValue{StringValue: "resp1000"}},
				},
			},
		},
		Extensions: map[string]*structpb.Value{
			"x-op-foo": &structpb.Value{Kind: &structpb.Value_StringValue{StringValue: "baz"}},
		},
	}
	if err := proto.SetExtension(proto.Message(meth.Options), swagger_options.E_Openapiv2Operation, &swaggerOperation); err != nil {
		t.Fatalf("proto.SetExtension(MethodDescriptorProto.Options) failed: %v", err)
	}
	reg := descriptor.NewRegistry()
	fileCL := crossLinkFixture(&file)
	err := reg.Load(reqFromFile(fileCL))
	if err != nil {
		t.Errorf("reg.Load(%#v) failed with %v; want success", file, err)
		return
	}
	result, err := applyTemplate(param{File: fileCL, reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want, is, name := "2.0", result.Swagger, "Swagger"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := []extension{
		{key: "x-bar", value: json.RawMessage("[\n      \"baz\"\n    ]")},
		{key: "x-foo", value: json.RawMessage("\"bar\"")},
	}, result.extensions, "Extensions"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}

	var scheme swaggerSecuritySchemeObject
	for _, v := range result.SecurityDefinitions {
		scheme = v
	}
	if want, is, name := []extension{
		{key: "x-security-baz", value: json.RawMessage("true")},
	}, scheme.extensions, "SecurityScheme.Extensions"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}

	if want, is, name := []extension{
		{key: "x-info-extension", value: json.RawMessage("\"bar\"")},
	}, result.Info.extensions, "Info.Extensions"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}

	var operation *swaggerOperationObject
	var response swaggerResponseObject
	for _, v := range result.Paths {
		operation = v.Get
		response = v.Get.Responses["200"]
	}
	if want, is, name := []extension{
		{key: "x-op-foo", value: json.RawMessage("\"baz\"")},
	}, operation.extensions, "operation.Extensions"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
	if want, is, name := []extension{
		{key: "x-resp-id", value: json.RawMessage("\"resp1000\"")},
	}, response.extensions, "response.Extensions"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}
}

func TestValidateHeaderType(t *testing.T) {
	type test struct {
		Type          string
		Format        string
		expectedError error
	}
	tests := []test{
		{
			"string",
			"date-time",
			nil,
		},
		{
			"boolean",
			"",
			nil,
		},
		{
			"integer",
			"uint",
			nil,
		},
		{
			"integer",
			"uint8",
			nil,
		},
		{
			"integer",
			"uint16",
			nil,
		},
		{
			"integer",
			"uint32",
			nil,
		},
		{
			"integer",
			"uint64",
			nil,
		},
		{
			"integer",
			"int",
			nil,
		},
		{
			"integer",
			"int8",
			nil,
		},
		{
			"integer",
			"int16",
			nil,
		},
		{
			"integer",
			"int32",
			nil,
		},
		{
			"integer",
			"int64",
			nil,
		},
		{
			"integer",
			"float64",
			errors.New("the provided format \"float64\" is not a valid extension of the type \"integer\""),
		},
		{
			"integer",
			"uuid",
			errors.New("the provided format \"uuid\" is not a valid extension of the type \"integer\""),
		},
		{
			"number",
			"uint",
			nil,
		},
		{
			"number",
			"uint8",
			nil,
		},
		{
			"number",
			"uint16",
			nil,
		},
		{
			"number",
			"uint32",
			nil,
		},
		{
			"number",
			"uint64",
			nil,
		},
		{
			"number",
			"int",
			nil,
		},
		{
			"number",
			"int8",
			nil,
		},
		{
			"number",
			"int16",
			nil,
		},
		{
			"number",
			"int32",
			nil,
		},
		{
			"number",
			"int64",
			nil,
		},
		{
			"number",
			"float",
			nil,
		},
		{
			"number",
			"float32",
			nil,
		},
		{
			"number",
			"float64",
			nil,
		},
		{
			"number",
			"complex64",
			nil,
		},
		{
			"number",
			"complex128",
			nil,
		},
		{
			"number",
			"double",
			nil,
		},
		{
			"number",
			"byte",
			nil,
		},
		{
			"number",
			"rune",
			nil,
		},
		{
			"number",
			"uintptr",
			nil,
		},
		{
			"number",
			"date",
			errors.New("the provided format \"date\" is not a valid extension of the type \"number\""),
		},
		{
			"array",
			"",
			errors.New("the provided header type \"array\" is not supported"),
		},
		{
			"foo",
			"",
			errors.New("the provided header type \"foo\" is not supported"),
		},
	}
	for _, v := range tests {
		err := validateHeaderTypeAndFormat(v.Type, v.Format)

		if v.expectedError == nil {
			if err != nil {
				t.Errorf("unexpected error %v", err)
			}
		} else {
			if err == nil {
				t.Fatal("expected header error not returned")
			}
			if err.Error() != v.expectedError.Error() {
				t.Errorf("expected error malformed, expected %q, got %q", v.expectedError.Error(), err.Error())
			}
		}
	}

}

func TestValidateDefaultValueType(t *testing.T) {
	type test struct {
		Type          string
		Value         string
		Format        string
		expectedError error
	}
	tests := []test{
		{
			"string",
			`"string"`,
			"",
			nil,
		},
		{
			"string",
			"\"2012-11-01T22:08:41+00:00\"",
			"date-time",
			nil,
		},
		{
			"string",
			"\"2012-11-01\"",
			"date",
			nil,
		},
		{
			"string",
			"0",
			"",
			errors.New("the provided default value \"0\" does not match provider type \"string\", or is not properly quoted with escaped quotations"),
		},
		{
			"string",
			"false",
			"",
			errors.New("the provided default value \"false\" does not match provider type \"string\", or is not properly quoted with escaped quotations"),
		},
		{
			"boolean",
			"true",
			"",
			nil,
		},
		{
			"boolean",
			"0",
			"",
			errors.New("the provided default value \"0\" does not match provider type \"boolean\""),
		},
		{
			"boolean",
			`"string"`,
			"",
			errors.New("the provided default value \"\\\"string\\\"\" does not match provider type \"boolean\""),
		},
		{
			"number",
			"1.2",
			"",
			nil,
		},
		{
			"number",
			"123",
			"",
			nil,
		},
		{
			"number",
			"nan",
			"",
			errors.New("the provided number \"nan\" is not a valid JSON number"),
		},
		{
			"number",
			"NaN",
			"",
			errors.New("the provided number \"NaN\" is not a valid JSON number"),
		},
		{
			"number",
			"-459.67",
			"",
			nil,
		},
		{
			"number",
			"inf",
			"",
			errors.New("the provided number \"inf\" is not a valid JSON number"),
		},
		{
			"number",
			"infinity",
			"",
			errors.New("the provided number \"infinity\" is not a valid JSON number"),
		},
		{
			"number",
			"Inf",
			"",
			errors.New("the provided number \"Inf\" is not a valid JSON number"),
		},
		{
			"number",
			"Infinity",
			"",
			errors.New("the provided number \"Infinity\" is not a valid JSON number"),
		},
		{
			"number",
			"false",
			"",
			errors.New("the provided default value \"false\" does not match provider type \"number\""),
		},
		{
			"number",
			`"string"`,
			"",
			errors.New("the provided default value \"\\\"string\\\"\" does not match provider type \"number\""),
		},
		{
			"integer",
			"2",
			"",
			nil,
		},
		{
			"integer",
			fmt.Sprint(math.MaxInt32),
			"int32",
			nil,
		},
		{
			"integer",
			fmt.Sprint(math.MaxInt32 + 1),
			"int32",
			errors.New("the provided default value \"2147483648\" does not match provided format \"int32\""),
		},
		{
			"integer",
			fmt.Sprint(math.MaxInt64),
			"int64",
			nil,
		},
		{
			"integer",
			"9223372036854775808",
			"int64",
			errors.New("the provided default value \"9223372036854775808\" does not match provided format \"int64\""),
		},
		{
			"integer",
			"18446744073709551615",
			"uint64",
			nil,
		},
		{
			"integer",
			"false",
			"",
			errors.New("the provided default value \"false\" does not match provided type \"integer\""),
		},
		{
			"integer",
			"1.2",
			"",
			errors.New("the provided default value \"1.2\" does not match provided type \"integer\""),
		},
		{
			"integer",
			`"string"`,
			"",
			errors.New("the provided default value \"\\\"string\\\"\" does not match provided type \"integer\""),
		},
	}
	for _, v := range tests {
		err := validateDefaultValueTypeAndFormat(v.Type, v.Value, v.Format)

		if v.expectedError == nil {
			if err != nil {
				t.Errorf("unexpected error '%v'", err)
			}
		} else {
			if err == nil {
				t.Error("expected update error not returned")
			}
			if err.Error() != v.expectedError.Error() {
				t.Errorf("expected error malformed, expected %q, got %q", v.expectedError.Error(), err.Error())
			}
		}
	}

}

func TestApplyTemplateHeaders(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Example"),
		InputType:  proto.String("ExampleMessage"),
		OutputType: proto.String("ExampleMessage"),
		Options:    &protodescriptor.MethodOptions{},
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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			Dependency:     []string{"a.example/b/c.proto", "a.example/d/e.proto"},
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
			Options:        &protodescriptor.FileOptions{},
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
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo",
								},
							},
						},
					},
				},
			},
		},
	}

	swaggerOperation := swagger_options.Operation{
		Responses: map[string]*swagger_options.Response{
			"200": &swagger_options.Response{
				Description: "Testing Headers",
				Headers: map[string]*swagger_options.Header{
					"string": {
						Description: "string header description",
						Type:        "string",
						Format:      "uuid",
						Pattern:     "",
					},
					"boolean": {
						Description: "boolean header description",
						Type:        "boolean",
						Default:     "true",
						Pattern:     "^true|false$",
					},
					"integer": {
						Description: "integer header description",
						Type:        "integer",
						Default:     "0",
						Pattern:     "^[0-9]$",
					},
					"number": {
						Description: "number header description",
						Type:        "number",
						Default:     "1.2",
						Pattern:     "^[-+]?[0-9]*\\.?[0-9]+([eE][-+]?[0-9]+)?$",
					},
				},
			},
		},
	}
	if err := proto.SetExtension(proto.Message(meth.Options), swagger_options.E_Openapiv2Operation, &swaggerOperation); err != nil {
		t.Fatalf("proto.SetExtension(MethodDescriptorProto.Options) failed: %v", err)
	}
	reg := descriptor.NewRegistry()
	fileCL := crossLinkFixture(&file)
	err := reg.Load(reqFromFile(fileCL))
	if err != nil {
		t.Errorf("reg.Load(%#v) failed with %v; want success", file, err)
		return
	}
	result, err := applyTemplate(param{File: fileCL, reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want, is, name := "2.0", result.Swagger, "Swagger"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
	}

	var response swaggerResponseObject
	for _, v := range result.Paths {
		response = v.Get.Responses["200"]
	}
	if want, is, name := []swaggerHeadersObject{
		{
			"String": swaggerHeaderObject{
				Description: "string header description",
				Type:        "string",
				Format:      "uuid",
				Pattern:     "",
			},
			"Boolean": swaggerHeaderObject{
				Description: "boolean header description",
				Type:        "boolean",
				Default:     json.RawMessage("true"),
				Pattern:     "^true|false$",
			},
			"Integer": swaggerHeaderObject{
				Description: "integer header description",
				Type:        "integer",
				Default:     json.RawMessage("0"),
				Pattern:     "^[0-9]$",
			},
			"Number": swaggerHeaderObject{
				Description: "number header description",
				Type:        "number",
				Default:     json.RawMessage("1.2"),
				Pattern:     "^[-+]?[0-9]*\\.?[0-9]+([eE][-+]?[0-9]+)?$",
			},
		},
	}[0], response.Headers, "response.Headers"; !reflect.DeepEqual(is, want) {
		t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, is, want)
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

	meth.ServerStreaming = proto.Bool(false)

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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc, nesteddesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
									Template: "/v1/echo", // TODO(achew): Figure out what this hsould really be
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
	reg := descriptor.NewRegistry()
	reg.Load(&plugin.CodeGeneratorRequest{ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto}})
	result, err := applyTemplate(param{File: crossLinkFixture(&file), reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}
	if want, got := "2.0", result.Swagger; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).Swagger = %s want to be %s", file, got, want)
	}
	if want, got := "", result.BasePath; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).BasePath = %s want to be %s", file, got, want)
	}
	if want, got := ([]string)(nil), result.Schemes; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).Schemes = %s want to be %s", file, got, want)
	}
	if want, got := []string{"application/json"}, result.Consumes; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).Consumes = %s want to be %s", file, got, want)
	}
	if want, got := []string{"application/json"}, result.Produces; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).Produces = %s want to be %s", file, got, want)
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
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
		ServerStreaming: proto.Bool(true),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}

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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc, nesteddesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
									Template: "/v1/echo", // TODO(achew): Figure out what this hsould really be
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
	reg := descriptor.NewRegistry()
	if err := AddStreamError(reg); err != nil {
		t.Errorf("AddStreamError(%#v) failed with %v; want success", reg, err)
		return
	}
	reg.Load(&plugin.CodeGeneratorRequest{ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto}})
	result, err := applyTemplate(param{File: crossLinkFixture(&file), reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}

	// Only ExampleMessage must be present, not NestedMessage
	if want, got, name := 4, len(result.Definitions), "len(Definitions)"; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).%s = %d want to be %d", file, name, got, want)
	}
	if _, ok := result.Paths["/v1/echo"].Post.Responses["200"]; !ok {
		t.Errorf("applyTemplate(%#v).%s = expected 200 response to be defined", file, `result.Paths["/v1/echo"].Post.Responses["200"]`)
	} else {
		if want, got, name := "A successful response.(streaming responses)", result.Paths["/v1/echo"].Post.Responses["200"].Description, `result.Paths["/v1/echo"].Post.Responses["200"].Description`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
		}
		streamExampleExampleMessage := result.Paths["/v1/echo"].Post.Responses["200"].Schema
		if want, got, name := "object", streamExampleExampleMessage.Type, `result.Paths["/v1/echo"].Post.Responses["200"].Schema.Type`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
		}
		if want, got, name := "Stream result of exampleExampleMessage", streamExampleExampleMessage.Title, `result.Paths["/v1/echo"].Post.Responses["200"].Schema.Title`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
		}
		streamExampleExampleMessageProperties := *(streamExampleExampleMessage.Properties)
		if want, got, name := 2, len(streamExampleExampleMessageProperties), `len(StreamDefinitions["exampleExampleMessage"].Properties)`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %d want to be %d", file, name, got, want)
		} else {
			resultProperty := streamExampleExampleMessageProperties[0]
			if want, got, name := "result", resultProperty.Key, `(*(StreamDefinitions["exampleExampleMessage"].Properties))[0].Key`; !reflect.DeepEqual(got, want) {
				t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
			}
			result := resultProperty.Value.(swaggerSchemaObject)
			if want, got, name := "#/definitions/exampleExampleMessage", result.Ref, `((*(StreamDefinitions["exampleExampleMessage"].Properties))[0].Value.(swaggerSchemaObject)).Ref`; !reflect.DeepEqual(got, want) {
				t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
			}
			errorProperty := streamExampleExampleMessageProperties[1]
			if want, got, name := "error", errorProperty.Key, `(*(StreamDefinitions["exampleExampleMessage"].Properties))[0].Key`; !reflect.DeepEqual(got, want) {
				t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
			}
			err := errorProperty.Value.(swaggerSchemaObject)
			if want, got, name := "#/definitions/runtimeStreamError", err.Ref, `((*(StreamDefinitions["exampleExampleMessage"].Properties))[0].Value.(swaggerSchemaObject)).Ref`; !reflect.DeepEqual(got, want) {
				t.Errorf("applyTemplate(%#v).%s = %s want to be %s", file, name, got, want)
			}
		}
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func TestApplyTemplateRequestWithUnusedReferences(t *testing.T) {
	reqdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("string"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				Number: proto.Int32(1),
			},
		},
	}
	respdesc := &protodescriptor.DescriptorProto{
		Name: proto.String("EmptyMessage"),
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:            proto.String("Example"),
		InputType:       proto.String("ExampleMessage"),
		OutputType:      proto.String("EmptyMessage"),
		ClientStreaming: proto.Bool(false),
		ServerStreaming: proto.Bool(false),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("ExampleService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}

	req := &descriptor.Message{
		DescriptorProto: reqdesc,
	}
	resp := &descriptor.Message{
		DescriptorProto: respdesc,
	}
	stringField := &descriptor.Field{
		Message:              req,
		FieldDescriptorProto: req.GetField()[0],
	}
	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			MessageType:    []*protodescriptor.DescriptorProto{reqdesc, respdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/example/example.pb",
			Name: "example_pb",
		},
		Messages: []*descriptor.Message{req, resp},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth,
						RequestType:           req,
						ResponseType:          resp,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "GET",
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/example",
								},
							},
							{
								HTTPMethod: "POST",
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/example/{string}",
								},
								PathParams: []descriptor.Parameter{
									{
										FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
											{
												Name:   "string",
												Target: stringField,
											},
										}),
										Target: stringField,
									},
								},
								Body: &descriptor.Body{
									FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
										{
											Name:   "string",
											Target: stringField,
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

	reg := descriptor.NewRegistry()
	reg.Load(&plugin.CodeGeneratorRequest{ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto}})
	result, err := applyTemplate(param{File: crossLinkFixture(&file), reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}

	// Only EmptyMessage must be present, not ExampleMessage
	if want, got, name := 1, len(result.Definitions), "len(Definitions)"; !reflect.DeepEqual(got, want) {
		t.Errorf("applyTemplate(%#v).%s = %d want to be %d", file, name, got, want)
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func TestApplyTemplateRequestWithBodyQueryParameters(t *testing.T) {
	bookDesc := &protodescriptor.DescriptorProto{
		Name: proto.String("Book"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("name"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:   proto.String("id"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				Number: proto.Int32(2),
			},
		},
	}
	createDesc := &protodescriptor.DescriptorProto{
		Name: proto.String("CreateBookRequest"),
		Field: []*protodescriptor.FieldDescriptorProto{
			{
				Name:   proto.String("parent"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				Number: proto.Int32(1),
			},
			{
				Name:     proto.String("book"),
				Label:    protodescriptor.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
				Type:     protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				TypeName: proto.String("Book"),
				Number:   proto.Int32(2),
			},
			{
				Name:   proto.String("book_id"),
				Label:  protodescriptor.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
				Type:   protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				Number: proto.Int32(3),
			},
		},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("CreateBook"),
		InputType:  proto.String("CreateBookRequest"),
		OutputType: proto.String("Book"),
	}
	svc := &protodescriptor.ServiceDescriptorProto{
		Name:   proto.String("BookService"),
		Method: []*protodescriptor.MethodDescriptorProto{meth},
	}

	bookMsg := &descriptor.Message{
		DescriptorProto: bookDesc,
	}
	createMsg := &descriptor.Message{
		DescriptorProto: createDesc,
	}

	parentField := &descriptor.Field{
		Message:              createMsg,
		FieldDescriptorProto: createMsg.GetField()[0],
	}
	bookField := &descriptor.Field{
		Message:              createMsg,
		FieldMessage:         bookMsg,
		FieldDescriptorProto: createMsg.GetField()[1],
	}
	bookIDField := &descriptor.Field{
		Message:              createMsg,
		FieldDescriptorProto: createMsg.GetField()[2],
	}

	createMsg.Fields = []*descriptor.Field{parentField, bookField, bookIDField}

	file := descriptor.File{
		FileDescriptorProto: &protodescriptor.FileDescriptorProto{
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("book.proto"),
			MessageType:    []*protodescriptor.DescriptorProto{bookDesc, createDesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
		},
		GoPkg: descriptor.GoPackage{
			Path: "example.com/path/to/book.pb",
			Name: "book_pb",
		},
		Messages: []*descriptor.Message{bookMsg, createMsg},
		Services: []*descriptor.Service{
			{
				ServiceDescriptorProto: svc,
				Methods: []*descriptor.Method{
					{
						MethodDescriptorProto: meth,
						RequestType:           createMsg,
						ResponseType:          bookMsg,
						Bindings: []*descriptor.Binding{
							{
								HTTPMethod: "POST",
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/{parent=publishers/*}/books",
								},
								PathParams: []descriptor.Parameter{
									{
										FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
											{
												Name:   "parent",
												Target: parentField,
											},
										}),
										Target: parentField,
									},
								},
								Body: &descriptor.Body{
									FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{
										{
											Name:   "book",
											Target: bookField,
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
	reg := descriptor.NewRegistry()
	reg.Load(&plugin.CodeGeneratorRequest{ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto}})
	result, err := applyTemplate(param{File: crossLinkFixture(&file), reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}

	if _, ok := result.Paths["/v1/{parent=publishers/*}/books"].Post.Responses["200"]; !ok {
		t.Errorf("applyTemplate(%#v).%s = expected 200 response to be defined", file, `result.Paths["/v1/{parent=publishers/*}/books"].Post.Responses["200"]`)
	} else {
		if want, got, name := 3, len(result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters), `len(result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters)`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %d want to be %d", file, name, got, want)
		}

		type param struct {
			Name     string
			In       string
			Required bool
		}

		p0 := result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[0]
		if want, got, name := (param{"parent", "path", true}), (param{p0.Name, p0.In, p0.Required}), `result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[0]`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %v want to be %v", file, name, got, want)
		}
		p1 := result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[1]
		if want, got, name := (param{"body", "body", true}), (param{p1.Name, p1.In, p1.Required}), `result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[1]`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %v want to be %v", file, name, got, want)
		}
		p2 := result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[2]
		if want, got, name := (param{"book_id", "query", false}), (param{p2.Name, p2.In, p2.Required}), `result.Paths["/v1/{parent=publishers/*}/books"].Post.Parameters[1]`; !reflect.DeepEqual(got, want) {
			t.Errorf("applyTemplate(%#v).%s = %v want to be %v", file, name, got, want)
		}
	}

	// If there was a failure, print out the input and the json result for debugging.
	if t.Failed() {
		t.Errorf("had: %s", file)
		t.Errorf("got: %s", fmt.Sprint(result))
	}
}

func generateFieldsForJSONReservedName() []*descriptor.Field {
	fields := make([]*descriptor.Field, 0)
	fieldName := string("json_name")
	fieldJSONName := string("jsonNAME")
	fieldDescriptor := protodescriptor.FieldDescriptorProto{Name: &fieldName, JsonName: &fieldJSONName}
	field := &descriptor.Field{FieldDescriptorProto: &fieldDescriptor}
	return append(fields, field)
}

func generateMsgsForJSONReservedName() []*descriptor.Message {
	result := make([]*descriptor.Message, 0)
	// The first message, its field is field_abc and its type is NewType
	// NewType field_abc
	fieldName := "field_abc"
	fieldJSONName := "fieldAbc"
	messageName1 := "message1"
	messageType := "pkg.a.NewType"
	pfd := protodescriptor.FieldDescriptorProto{Name: &fieldName, JsonName: &fieldJSONName, TypeName: &messageType}
	result = append(result,
		&descriptor.Message{
			DescriptorProto: &protodescriptor.DescriptorProto{
				Name: &messageName1, Field: []*protodescriptor.FieldDescriptorProto{&pfd},
			},
		})
	// The second message, its name is NewName, its type is string
	// message NewType {
	//    string field_newName [json_name = RESERVEDJSONNAME]
	// }
	messageName := "NewType"
	field := "field_newName"
	fieldJSONName2 := "RESERVEDJSONNAME"
	pfd2 := protodescriptor.FieldDescriptorProto{Name: &field, JsonName: &fieldJSONName2}
	result = append(result, &descriptor.Message{
		DescriptorProto: &protodescriptor.DescriptorProto{
			Name: &messageName, Field: []*protodescriptor.FieldDescriptorProto{&pfd2},
		},
	})
	return result
}

func TestTemplateWithJsonCamelCase(t *testing.T) {
	var tests = []struct {
		input    string
		expected string
	}{
		{"/test/{test_id}", "/test/{testId}"},
		{"/test1/{test1_id}/test2/{test2_id}", "/test1/{test1Id}/test2/{test2Id}"},
		{"/test1/{test1_id}/{test2_id}", "/test1/{test1Id}/{test2Id}"},
		{"/test1/test2/{test1_id}/{test2_id}", "/test1/test2/{test1Id}/{test2Id}"},
		{"/test1/{test1_id1_id2}", "/test1/{test1Id1Id2}"},
		{"/test1/{test1_id1_id2}/test2/{test2_id3_id4}", "/test1/{test1Id1Id2}/test2/{test2Id3Id4}"},
		{"/test1/test2/{test1_id1_id2}/{test2_id3_id4}", "/test1/test2/{test1Id1Id2}/{test2Id3Id4}"},
		{"test/{a}", "test/{a}"},
		{"test/{ab}", "test/{ab}"},
		{"test/{a_a}", "test/{aA}"},
		{"test/{ab_c}", "test/{abC}"},
		{"test/{json_name}", "test/{jsonNAME}"},
		{"test/{field_abc.field_newName}", "test/{fieldAbc.RESERVEDJSONNAME}"},
	}
	reg := descriptor.NewRegistry()
	reg.SetUseJSONNamesForFields(true)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
}

func TestTemplateWithoutJsonCamelCase(t *testing.T) {
	var tests = []struct {
		input    string
		expected string
	}{
		{"/test/{test_id}", "/test/{test_id}"},
		{"/test1/{test1_id}/test2/{test2_id}", "/test1/{test1_id}/test2/{test2_id}"},
		{"/test1/{test1_id}/{test2_id}", "/test1/{test1_id}/{test2_id}"},
		{"/test1/test2/{test1_id}/{test2_id}", "/test1/test2/{test1_id}/{test2_id}"},
		{"/test1/{test1_id1_id2}", "/test1/{test1_id1_id2}"},
		{"/test1/{test1_id1_id2}/test2/{test2_id3_id4}", "/test1/{test1_id1_id2}/test2/{test2_id3_id4}"},
		{"/test1/test2/{test1_id1_id2}/{test2_id3_id4}", "/test1/test2/{test1_id1_id2}/{test2_id3_id4}"},
		{"test/{a}", "test/{a}"},
		{"test/{ab}", "test/{ab}"},
		{"test/{a_a}", "test/{a_a}"},
		{"test/{json_name}", "test/{json_name}"},
		{"test/{field_abc.field_newName}", "test/{field_abc.field_newName}"},
	}
	reg := descriptor.NewRegistry()
	reg.SetUseJSONNamesForFields(false)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
}

func TestTemplateToSwaggerPath(t *testing.T) {
	var tests = []struct {
		input    string
		expected string
	}{
		{"/test", "/test"},
		{"/{test}", "/{test}"},
		{"/{test=prefix/*}", "/{test}"},
		{"/{test=prefix/that/has/multiple/parts/to/it/*}", "/{test}"},
		{"/{test1}/{test2}", "/{test1}/{test2}"},
		{"/{test1}/{test2}/", "/{test1}/{test2}/"},
		{"/{name=prefix/*}", "/{name=prefix/*}"},
		{"/{name=prefix1/*/prefix2/*}", "/{name=prefix1/*/prefix2/*}"},
		{"/{user.name=prefix/*}", "/{user.name=prefix/*}"},
		{"/{user.name=prefix1/*/prefix2/*}", "/{user.name=prefix1/*/prefix2/*}"},
		{"/{parent=prefix/*}/children", "/{parent=prefix/*}/children"},
		{"/{name=prefix/*}:customMethod", "/{name=prefix/*}:customMethod"},
		{"/{name=prefix1/*/prefix2/*}:customMethod", "/{name=prefix1/*/prefix2/*}:customMethod"},
		{"/{user.name=prefix/*}:customMethod", "/{user.name=prefix/*}:customMethod"},
		{"/{user.name=prefix1/*/prefix2/*}:customMethod", "/{user.name=prefix1/*/prefix2/*}:customMethod"},
		{"/{parent=prefix/*}/children:customMethod", "/{parent=prefix/*}/children:customMethod"},
	}
	reg := descriptor.NewRegistry()
	reg.SetUseJSONNamesForFields(false)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
	reg.SetUseJSONNamesForFields(true)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
}

func BenchmarkTemplateToSwaggerPath(b *testing.B) {
	const input = "/{user.name=prefix1/*/prefix2/*}:customMethod"

	b.Run("with JSON names", func(b *testing.B) {
		reg := descriptor.NewRegistry()
		reg.SetUseJSONNamesForFields(false)

		for i := 0; i < b.N; i++ {
			_ = templateToSwaggerPath(input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		}
	})

	b.Run("without JSON names", func(b *testing.B) {
		reg := descriptor.NewRegistry()
		reg.SetUseJSONNamesForFields(true)

		for i := 0; i < b.N; i++ {
			_ = templateToSwaggerPath(input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		}
	})
}

func TestResolveFullyQualifiedNameToSwaggerName(t *testing.T) {
	var tests = []struct {
		input                string
		output               string
		listOfFQMNs          []string
		useFQNForSwaggerName bool
	}{
		{
			".a.b.C",
			"C",
			[]string{
				".a.b.C",
			},
			false,
		},
		{
			".a.b.C",
			"abC",
			[]string{
				".a.C",
				".a.b.C",
			},
			false,
		},
		{
			".a.b.C",
			"abC",
			[]string{
				".C",
				".a.C",
				".a.b.C",
			},
			false,
		},
		{
			".a.b.C",
			"a.b.C",
			[]string{
				".C",
				".a.C",
				".a.b.C",
			},
			true,
		},
	}

	for _, data := range tests {
		names := resolveFullyQualifiedNameToSwaggerNames(data.listOfFQMNs, data.useFQNForSwaggerName)
		output := names[data.input]
		if output != data.output {
			t.Errorf("Expected fullyQualifiedNameToSwaggerName(%v) to be %s but got %s",
				data.input, data.output, output)
		}
	}
}

func TestFQMNtoSwaggerName(t *testing.T) {
	var tests = []struct {
		input    string
		expected string
	}{
		{"/test", "/test"},
		{"/{test}", "/{test}"},
		{"/{test=prefix/*}", "/{test}"},
		{"/{test=prefix/that/has/multiple/parts/to/it/*}", "/{test}"},
		{"/{test1}/{test2}", "/{test1}/{test2}"},
		{"/{test1}/{test2}/", "/{test1}/{test2}/"},
	}
	reg := descriptor.NewRegistry()
	reg.SetUseJSONNamesForFields(false)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
	reg.SetUseJSONNamesForFields(true)
	for _, data := range tests {
		actual := templateToSwaggerPath(data.input, reg, generateFieldsForJSONReservedName(), generateMsgsForJSONReservedName())
		if data.expected != actual {
			t.Errorf("Expected templateToSwaggerPath(%v) = %v, actual: %v", data.input, data.expected, actual)
		}
	}
}

func TestSchemaOfField(t *testing.T) {
	type test struct {
		field    *descriptor.Field
		refs     refMap
		expected swaggerSchemaObject
	}

	var fieldOptions = new(protodescriptor.FieldOptions)
	err := proto.SetExtension(fieldOptions, swagger_options.E_Openapiv2Field, &swagger_options.JSONSchema{
		Title:       "field title",
		Description: "field description",
	})
	if err != nil {
		t.Errorf("proto.SetExtension() failed with %v; want success", err)
	}

	tests := []test{
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name: proto.String("primitive_field"),
					Type: protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "string",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:  proto.String("repeated_primitive_field"),
					Type:  protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
					Label: protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "array",
					Items: &swaggerItemsObject{
						Type: "string",
					},
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.StringValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "string",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("repeated_wrapped_field"),
					TypeName: proto.String(".google.protobuf.StringValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
					Label:    protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "array",
					Items: &swaggerItemsObject{
						Type: "string",
					},
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.BytesValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "string",
					Format: "byte",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.Int32Value"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "integer",
					Format: "int32",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.UInt32Value"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "integer",
					Format: "int64",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.Int64Value"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "string",
					Format: "int64",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.UInt64Value"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "string",
					Format: "uint64",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.FloatValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "number",
					Format: "float",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.DoubleValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "number",
					Format: "double",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.BoolValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "boolean",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.Struct"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "object",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.Value"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "object",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.ListValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "array",
					Items: (*swaggerItemsObject)(&schemaCore{
						Type: "object",
					}),
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("wrapped_field"),
					TypeName: proto.String(".google.protobuf.NullValue"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "string",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("message_field"),
					TypeName: proto.String(".example.Message"),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
				},
			},
			refs: refMap{".example.Message": struct{}{}},
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Ref: "#/definitions/exampleMessage",
				},
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("map_field"),
					Label:    protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
					TypeName: proto.String(".example.Message.MapFieldEntry"),
					Options:  fieldOptions,
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type: "object",
				},
				AdditionalProperties: &swaggerSchemaObject{
					schemaCore: schemaCore{Type: "string"},
				},
				Title:       "field title",
				Description: "field description",
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:    proto.String("array_field"),
					Label:   protodescriptor.FieldDescriptorProto_LABEL_REPEATED.Enum(),
					Type:    protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
					Options: fieldOptions,
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:  "array",
					Items: (*swaggerItemsObject)(&schemaCore{Type: "string"}),
				},
				Title:       "field title",
				Description: "field description",
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:    proto.String("primitive_field"),
					Label:   protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
					Type:    protodescriptor.FieldDescriptorProto_TYPE_INT32.Enum(),
					Options: fieldOptions,
				},
			},
			refs: make(refMap),
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Type:   "integer",
					Format: "int32",
				},
				Title:       "field title",
				Description: "field description",
			},
		},
		{
			field: &descriptor.Field{
				FieldDescriptorProto: &protodescriptor.FieldDescriptorProto{
					Name:     proto.String("message_field"),
					Label:    protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
					Type:     protodescriptor.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
					TypeName: proto.String(".example.Empty"),
					Options:  fieldOptions,
				},
			},
			refs: refMap{".example.Empty": struct{}{}},
			expected: swaggerSchemaObject{
				schemaCore: schemaCore{
					Ref: "#/definitions/exampleEmpty",
				},
				Title:       "field title",
				Description: "field description",
			},
		},
	}

	reg := descriptor.NewRegistry()
	reg.Load(&plugin.CodeGeneratorRequest{
		ProtoFile: []*protodescriptor.FileDescriptorProto{
			{
				SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
				Name:           proto.String("example.proto"),
				Package:        proto.String("example"),
				Dependency:     []string{},
				MessageType: []*protodescriptor.DescriptorProto{
					{
						Name: proto.String("Message"),
						Field: []*protodescriptor.FieldDescriptorProto{
							{
								Name: proto.String("value"),
								Type: protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
							},
						},
						NestedType: []*protodescriptor.DescriptorProto{
							{
								Name:    proto.String("MapFieldEntry"),
								Options: &protodescriptor.MessageOptions{MapEntry: proto.Bool(true)},
								Field: []*protodescriptor.FieldDescriptorProto{
									{},
									{
										Name:  proto.String("value"),
										Label: protodescriptor.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
										Type:  protodescriptor.FieldDescriptorProto_TYPE_STRING.Enum(),
									},
								},
							},
						},
					},
					{
						Name: proto.String("Empty"),
					},
				},
				EnumType: []*protodescriptor.EnumDescriptorProto{
					{
						Name: proto.String("Message"),
					},
				},
				Service: []*protodescriptor.ServiceDescriptorProto{},
			},
		},
	})

	for _, test := range tests {
		refs := make(refMap)
		actual := schemaOfField(test.field, reg, refs)
		expectedSchemaObject := test.expected
		if e, a := expectedSchemaObject, actual; !reflect.DeepEqual(a, e) {
			t.Errorf("Expected schemaOfField(%v) = %v, actual: %v", test.field, e, a)
		}
		if !reflect.DeepEqual(refs, test.refs) {
			t.Errorf("Expected schemaOfField(%v) to add refs %v, not %v", test.field, test.refs, refs)
		}
	}
}

func TestRenderMessagesAsDefinition(t *testing.T) {

	tests := []struct {
		descr    string
		msgDescs []*protodescriptor.DescriptorProto
		schema   map[string]swagger_options.Schema // per-message schema to add
		defs     swaggerDefinitionsObject
	}{
		{
			descr: "no swagger options",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{schemaCore: schemaCore{Type: "object"}},
			},
		},
		{
			descr: "example option",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					ExampleString: `{"foo":"bar"}`,
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{schemaCore: schemaCore{
					Type:    "object",
					Example: json.RawMessage(`{"foo":"bar"}`),
				}},
			},
		},
		{
			descr: "example option with something non-json",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					ExampleString: `XXXX anything goes XXXX`,
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{schemaCore: schemaCore{
					Type:    "object",
					Example: json.RawMessage(`XXXX anything goes XXXX`),
				}},
			},
		},
		{
			descr: "external docs option",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					ExternalDocs: &swagger_options.ExternalDocumentation{
						Description: "glorious docs",
						Url:         "https://nada",
					},
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{
					schemaCore: schemaCore{
						Type: "object",
					},
					ExternalDocs: &swaggerExternalDocumentationObject{
						Description: "glorious docs",
						URL:         "https://nada",
					},
				},
			},
		},
		{
			descr: "JSONSchema options",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					JsonSchema: &swagger_options.JSONSchema{
						Title:            "title",
						Description:      "desc",
						MultipleOf:       100,
						Maximum:          101,
						ExclusiveMaximum: true,
						Minimum:          1,
						ExclusiveMinimum: true,
						MaxLength:        10,
						MinLength:        3,
						Pattern:          "[a-z]+",
						MaxItems:         20,
						MinItems:         2,
						UniqueItems:      true,
						MaxProperties:    33,
						MinProperties:    22,
						Required:         []string{"req"},
						ReadOnly:         true,
					},
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{
					schemaCore: schemaCore{
						Type: "object",
					},
					Title:            "title",
					Description:      "desc",
					MultipleOf:       100,
					Maximum:          101,
					ExclusiveMaximum: true,
					Minimum:          1,
					ExclusiveMinimum: true,
					MaxLength:        10,
					MinLength:        3,
					Pattern:          "[a-z]+",
					MaxItems:         20,
					MinItems:         2,
					UniqueItems:      true,
					MaxProperties:    33,
					MinProperties:    22,
					Required:         []string{"req"},
					ReadOnly:         true,
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.descr, func(t *testing.T) {

			msgs := []*descriptor.Message{}
			for _, msgdesc := range test.msgDescs {
				msgdesc.Options = &protodescriptor.MessageOptions{}
				msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
			}

			reg := descriptor.NewRegistry()
			file := descriptor.File{
				FileDescriptorProto: &protodescriptor.FileDescriptorProto{
					SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
					Name:           proto.String("example.proto"),
					Package:        proto.String("example"),
					Dependency:     []string{},
					MessageType:    test.msgDescs,
					EnumType:       []*protodescriptor.EnumDescriptorProto{},
					Service:        []*protodescriptor.ServiceDescriptorProto{},
				},
				Messages: msgs,
			}
			reg.Load(&plugin.CodeGeneratorRequest{
				ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
			})

			msgMap := map[string]*descriptor.Message{}
			for _, d := range test.msgDescs {
				name := d.GetName()
				msg, err := reg.LookupMsg("example", name)
				if err != nil {
					t.Fatalf("lookup message %v: %v", name, err)
				}
				msgMap[msg.FQMN()] = msg

				if schema, ok := test.schema[name]; ok {
					err := proto.SetExtension(d.Options, swagger_options.E_Openapiv2Schema, &schema)
					if err != nil {
						t.Fatalf("SetExtension(%s, ...) returned error: %v", msg, err)
					}
				}
			}

			refs := make(refMap)
			actual := make(swaggerDefinitionsObject)
			renderMessagesAsDefinition(msgMap, actual, reg, refs)

			if !reflect.DeepEqual(actual, test.defs) {
				t.Errorf("Expected renderMessagesAsDefinition() to add defs %+v, not %+v", test.defs, actual)
			}
		})
	}
}

func TestUpdateSwaggerDataFromComments(t *testing.T) {

	tests := []struct {
		descr                 string
		swaggerObject         interface{}
		comments              string
		expectedError         error
		expectedSwaggerObject interface{}
		useGoTemplate         bool
	}{
		{
			descr:                 "empty comments",
			swaggerObject:         nil,
			expectedSwaggerObject: nil,
			comments:              "",
			expectedError:         nil,
		},
		{
			descr:         "set field to read only",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				ReadOnly:    true,
				Description: "... Output only. ...",
			},
			comments:      "... Output only. ...",
			expectedError: nil,
		},
		{
			descr:         "set title",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Title: "Comment with no trailing dot",
			},
			comments:      "Comment with no trailing dot",
			expectedError: nil,
		},
		{
			descr:         "set description",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Description: "Comment with trailing dot.",
			},
			comments:      "Comment with trailing dot.",
			expectedError: nil,
		},
		{
			descr: "use info object",
			swaggerObject: &swaggerObject{
				Info: swaggerInfoObject{},
			},
			expectedSwaggerObject: &swaggerObject{
				Info: swaggerInfoObject{
					Description: "Comment with trailing dot.",
				},
			},
			comments:      "Comment with trailing dot.",
			expectedError: nil,
		},
		{
			descr:         "multi line comment with title",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Title:       "First line",
				Description: "Second line",
			},
			comments:      "First line\n\nSecond line",
			expectedError: nil,
		},
		{
			descr:         "multi line comment no title",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Description: "First line.\n\nSecond line",
			},
			comments:      "First line.\n\nSecond line",
			expectedError: nil,
		},
		{
			descr:         "multi line comment with summary with dot",
			swaggerObject: &swaggerOperationObject{},
			expectedSwaggerObject: &swaggerOperationObject{
				Summary:     "First line.",
				Description: "Second line",
			},
			comments:      "First line.\n\nSecond line",
			expectedError: nil,
		},
		{
			descr:         "multi line comment with summary no dot",
			swaggerObject: &swaggerOperationObject{},
			expectedSwaggerObject: &swaggerOperationObject{
				Summary:     "First line",
				Description: "Second line",
			},
			comments:      "First line\n\nSecond line",
			expectedError: nil,
		},
		{
			descr:                 "multi line comment with summary no dot",
			swaggerObject:         &schemaCore{},
			expectedSwaggerObject: &schemaCore{},
			comments:              "Any comment",
			expectedError:         errors.New("no description nor summary property"),
		},
		{
			descr:         "without use_go_template",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Title:       "First line",
				Description: "{{import \"documentation.md\"}}",
			},
			comments:      "First line\n\n{{import \"documentation.md\"}}",
			expectedError: nil,
		},
		{
			descr:         "error with use_go_template",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Title:       "First line",
				Description: "open noneexistingfile.txt: no such file or directory",
			},
			comments:      "First line\n\n{{import \"noneexistingfile.txt\"}}",
			expectedError: nil,
			useGoTemplate: true,
		},
		{
			descr:         "template with use_go_template",
			swaggerObject: &swaggerSchemaObject{},
			expectedSwaggerObject: &swaggerSchemaObject{
				Title:       "Template",
				Description: `Description "which means nothing"`,
			},
			comments:      "Template\n\nDescription {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
			expectedError: nil,
			useGoTemplate: true,
		},
	}

	for _, test := range tests {
		t.Run(test.descr, func(t *testing.T) {
			reg := descriptor.NewRegistry()
			if test.useGoTemplate {
				reg.SetUseGoTemplate(true)
			}
			err := updateSwaggerDataFromComments(reg, test.swaggerObject, nil, test.comments, false)
			if test.expectedError == nil {
				if err != nil {
					t.Errorf("unexpected error '%v'", err)
				}
				if !reflect.DeepEqual(test.swaggerObject, test.expectedSwaggerObject) {
					t.Errorf("swaggerObject was not updated corretly, expected '%+v', got '%+v'", test.expectedSwaggerObject, test.swaggerObject)
				}
			} else {
				if err == nil {
					t.Error("expected update error not returned")
				}
				if !reflect.DeepEqual(test.swaggerObject, test.expectedSwaggerObject) {
					t.Errorf("swaggerObject was not updated corretly, expected '%+v', got '%+v'", test.expectedSwaggerObject, test.swaggerObject)
				}
				if err.Error() != test.expectedError.Error() {
					t.Errorf("expected error malformed, expected %q, got %q", test.expectedError.Error(), err.Error())
				}
			}
		})
	}
}

func TestMessageOptionsWithGoTemplate(t *testing.T) {
	tests := []struct {
		descr         string
		msgDescs      []*protodescriptor.DescriptorProto
		schema        map[string]swagger_options.Schema // per-message schema to add
		defs          swaggerDefinitionsObject
		useGoTemplate bool
	}{
		{
			descr: "external docs option",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					JsonSchema: &swagger_options.JSONSchema{
						Title:       "{{.Name}}",
						Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					},
					ExternalDocs: &swagger_options.ExternalDocumentation{
						Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					},
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{
					schemaCore: schemaCore{
						Type: "object",
					},
					Title:       "Message",
					Description: `Description "which means nothing"`,
					ExternalDocs: &swaggerExternalDocumentationObject{
						Description: `Description "which means nothing"`,
					},
				},
			},
			useGoTemplate: true,
		},
		{
			descr: "external docs option",
			msgDescs: []*protodescriptor.DescriptorProto{
				&protodescriptor.DescriptorProto{Name: proto.String("Message")},
			},
			schema: map[string]swagger_options.Schema{
				"Message": swagger_options.Schema{
					JsonSchema: &swagger_options.JSONSchema{
						Title:       "{{.Name}}",
						Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					},
					ExternalDocs: &swagger_options.ExternalDocumentation{
						Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					},
				},
			},
			defs: map[string]swaggerSchemaObject{
				"Message": swaggerSchemaObject{
					schemaCore: schemaCore{
						Type: "object",
					},
					Title:       "{{.Name}}",
					Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					ExternalDocs: &swaggerExternalDocumentationObject{
						Description: "Description {{with \"which means nothing\"}}{{printf \"%q\" .}}{{end}}",
					},
				},
			},
			useGoTemplate: false,
		},
	}

	for _, test := range tests {
		t.Run(test.descr, func(t *testing.T) {

			msgs := []*descriptor.Message{}
			for _, msgdesc := range test.msgDescs {
				msgdesc.Options = &protodescriptor.MessageOptions{}
				msgs = append(msgs, &descriptor.Message{DescriptorProto: msgdesc})
			}

			reg := descriptor.NewRegistry()
			reg.SetUseGoTemplate(test.useGoTemplate)
			file := descriptor.File{
				FileDescriptorProto: &protodescriptor.FileDescriptorProto{
					SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
					Name:           proto.String("example.proto"),
					Package:        proto.String("example"),
					Dependency:     []string{},
					MessageType:    test.msgDescs,
					EnumType:       []*protodescriptor.EnumDescriptorProto{},
					Service:        []*protodescriptor.ServiceDescriptorProto{},
				},
				Messages: msgs,
			}
			reg.Load(&plugin.CodeGeneratorRequest{
				ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto},
			})

			msgMap := map[string]*descriptor.Message{}
			for _, d := range test.msgDescs {
				name := d.GetName()
				msg, err := reg.LookupMsg("example", name)
				if err != nil {
					t.Fatalf("lookup message %v: %v", name, err)
				}
				msgMap[msg.FQMN()] = msg

				if schema, ok := test.schema[name]; ok {
					err := proto.SetExtension(d.Options, swagger_options.E_Openapiv2Schema, &schema)
					if err != nil {
						t.Fatalf("SetExtension(%s, ...) returned error: %v", msg, err)
					}
				}
			}

			refs := make(refMap)
			actual := make(swaggerDefinitionsObject)
			renderMessagesAsDefinition(msgMap, actual, reg, refs)

			if !reflect.DeepEqual(actual, test.defs) {
				t.Errorf("Expected renderMessagesAsDefinition() to add defs %+v, not %+v", test.defs, actual)
			}
		})
	}
}

func TestTemplateWithoutErrorDefinition(t *testing.T) {
	msgdesc := &protodescriptor.DescriptorProto{
		Name:  proto.String("ExampleMessage"),
		Field: []*protodescriptor.FieldDescriptorProto{},
	}
	meth := &protodescriptor.MethodDescriptorProto{
		Name:       proto.String("Echo"),
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
			SourceCodeInfo: &protodescriptor.SourceCodeInfo{},
			Name:           proto.String("example.proto"),
			Package:        proto.String("example"),
			MessageType:    []*protodescriptor.DescriptorProto{msgdesc, msgdesc},
			Service:        []*protodescriptor.ServiceDescriptorProto{svc},
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
								HTTPMethod: "POST",
								PathTmpl: httprule.Template{
									Version:  1,
									OpCodes:  []int{0, 0},
									Template: "/v1/echo",
								},
								Body: &descriptor.Body{
									FieldPath: descriptor.FieldPath([]descriptor.FieldPathComponent{}),
								},
							},
						},
					},
				},
			},
		},
	}
	reg := descriptor.NewRegistry()
	reg.Load(&plugin.CodeGeneratorRequest{ProtoFile: []*protodescriptor.FileDescriptorProto{file.FileDescriptorProto}})
	result, err := applyTemplate(param{File: crossLinkFixture(&file), reg: reg})
	if err != nil {
		t.Errorf("applyTemplate(%#v) failed with %v; want success", file, err)
		return
	}

	defRsp, ok := result.Paths["/v1/echo"].Post.Responses["default"]
	if !ok {
		return
	}

	ref := defRsp.Schema.schemaCore.Ref
	refName := strings.TrimPrefix(ref, "#/definitions/")
	if refName == "" {
		t.Fatal("created default Error response with empty reflink")
	}

	if _, ok := result.Definitions[refName]; !ok {
		t.Errorf("default Error response with reflink '%v', but its definition was not found", refName)
	}
}

func Test_getReservedJsonName(t *testing.T) {
	type args struct {
		fieldName                     string
		messageNameToFieldsToJSONName map[string]map[string]string
		fieldNameToType               map[string]string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			"test case 1: single dot use case",
			args{
				fieldName: "abc.a_1",
				messageNameToFieldsToJSONName: map[string]map[string]string{
					"Msg": {
						"a_1": "a1JSONNAME",
						"b_1": "b1JSONNAME",
					},
				},
				fieldNameToType: map[string]string{
					"abc": "pkg1.test.Msg",
					"bcd": "pkg1.test.Msg",
				},
			},
			"a1JSONNAME",
		},
		{
			"test case 2: single dot use case with no existing field",
			args{
				fieldName: "abc.d_1",
				messageNameToFieldsToJSONName: map[string]map[string]string{
					"Msg": {
						"a_1": "a1JSONNAME",
						"b_1": "b1JSONNAME",
					},
				},
				fieldNameToType: map[string]string{
					"abc": "pkg1.test.Msg",
					"bcd": "pkg1.test.Msg",
				},
			},
			"",
		},
		{
			"test case 3: double dot use case",
			args{
				fieldName: "pkg.abc.a_1",
				messageNameToFieldsToJSONName: map[string]map[string]string{
					"Msg": {
						"a_1": "a1JSONNAME",
						"b_1": "b1JSONNAME",
					},
				},
				fieldNameToType: map[string]string{
					"abc": "pkg1.test.Msg",
					"bcd": "pkg1.test.Msg",
				},
			},
			"a1JSONNAME",
		},
		{
			"test case 4: double dot use case with a not existed field",
			args{
				fieldName: "pkg.abc.c_1",
				messageNameToFieldsToJSONName: map[string]map[string]string{
					"Msg": {
						"a_1": "a1JSONNAME",
						"b_1": "b1JSONNAME",
					},
				},
				fieldNameToType: map[string]string{
					"abc": "pkg1.test.Msg",
					"bcd": "pkg1.test.Msg",
				},
			},
			"",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := getReservedJSONName(tt.args.fieldName, tt.args.messageNameToFieldsToJSONName, tt.args.fieldNameToType); got != tt.want {
				t.Errorf("getReservedJSONName() = %v, want %v", got, tt.want)
			}
		})
	}
}
