package descriptor

import (
	"testing"

	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/types/descriptorpb"
)

func TestGoPackageStandard(t *testing.T) {
	for _, spec := range []struct {
		pkg  GoPackage
		want bool
	}{
		{
			pkg:  GoPackage{Path: "fmt", Name: "fmt"},
			want: true,
		},
		{
			pkg:  GoPackage{Path: "encoding/json", Name: "json"},
			want: true,
		},
		{
			pkg:  GoPackage{Path: "google.golang.org/protobuf/encoding/protojson", Name: "jsonpb"},
			want: false,
		},
		{
			pkg:  GoPackage{Path: "golang.org/x/net/context", Name: "context"},
			want: false,
		},
		{
			pkg:  GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway", Name: "main"},
			want: false,
		},
		{
			pkg:  GoPackage{Path: "github.com/google/googleapis/google/api/http.pb", Name: "http_pb", Alias: "htpb"},
			want: false,
		},
	} {
		if got, want := spec.pkg.Standard(), spec.want; got != want {
			t.Errorf("%#v.Standard() = %v; want %v", spec.pkg, got, want)
		}
	}
}

func TestGoPackageString(t *testing.T) {
	for _, spec := range []struct {
		pkg  GoPackage
		want string
	}{
		{
			pkg:  GoPackage{Path: "fmt", Name: "fmt"},
			want: `"fmt"`,
		},
		{
			pkg:  GoPackage{Path: "encoding/json", Name: "json"},
			want: `"encoding/json"`,
		},
		{
			pkg:  GoPackage{Path: "google.golang.org/protobuf/encoding/protojson", Name: "jsonpb"},
			want: `"google.golang.org/protobuf/encoding/protojson"`,
		},
		{
			pkg:  GoPackage{Path: "golang.org/x/net/context", Name: "context"},
			want: `"golang.org/x/net/context"`,
		},
		{
			pkg:  GoPackage{Path: "github.com/grpc-ecosystem/grpc-gateway", Name: "main"},
			want: `"github.com/grpc-ecosystem/grpc-gateway"`,
		},
		{
			pkg:  GoPackage{Path: "github.com/google/googleapis/google/api/http.pb", Name: "http_pb", Alias: "htpb"},
			want: `htpb "github.com/google/googleapis/google/api/http.pb"`,
		},
	} {
		if got, want := spec.pkg.String(), spec.want; got != want {
			t.Errorf("%#v.String() = %q; want %q", spec.pkg, got, want)
		}
	}
}

func TestFieldPath(t *testing.T) {
	var fds []*descriptorpb.FileDescriptorProto
	for _, src := range []string{
		`
		name: 'example.proto'
		package: 'example'
		message_type <
			name: 'Nest'
			field <
				name: 'nest2_field'
				label: LABEL_OPTIONAL
				type: TYPE_MESSAGE
				type_name: 'Nest2'
				number: 1
			>
			field <
				name: 'terminal_field'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 2
			>
		>
		syntax: "proto3"
		`, `
		name: 'another.proto'
		package: 'example'
		message_type <
			name: 'Nest2'
			field <
				name: 'nest_field'
				label: LABEL_OPTIONAL
				type: TYPE_MESSAGE
				type_name: 'Nest'
				number: 1
			>
			field <
				name: 'terminal_field'
				label: LABEL_OPTIONAL
				type: TYPE_STRING
				number: 2
			>
		>
		syntax: "proto2"
		`,
	} {
		var fd descriptorpb.FileDescriptorProto
		if err := prototext.Unmarshal([]byte(src), &fd); err != nil {
			t.Fatalf("proto.UnmarshalText(%s, &fd) failed with %v; want success", src, err)
		}
		fds = append(fds, &fd)
	}
	nest1 := &Message{
		DescriptorProto: fds[0].MessageType[0],
		Fields: []*Field{
			{FieldDescriptorProto: fds[0].MessageType[0].Field[0]},
			{FieldDescriptorProto: fds[0].MessageType[0].Field[1]},
		},
	}
	nest2 := &Message{
		DescriptorProto: fds[1].MessageType[0],
		Fields: []*Field{
			{FieldDescriptorProto: fds[1].MessageType[0].Field[0]},
			{FieldDescriptorProto: fds[1].MessageType[0].Field[1]},
		},
	}
	file1 := &File{
		FileDescriptorProto: fds[0],
		GoPkg:               GoPackage{Path: "example", Name: "example"},
		Messages:            []*Message{nest1},
	}
	file2 := &File{
		FileDescriptorProto: fds[1],
		GoPkg:               GoPackage{Path: "example", Name: "example"},
		Messages:            []*Message{nest2},
	}
	crossLinkFixture(file1)
	crossLinkFixture(file2)

	c1 := FieldPathComponent{
		Name:   "nest_field",
		Target: nest2.Fields[0],
	}
	if got, want := c1.ValueExpr(), "GetNestField()"; got != want {
		t.Errorf("c1.ValueExpr() = %q; want %q", got, want)
	}
	if got, want := c1.AssignableExpr(), "NestField"; got != want {
		t.Errorf("c1.AssignableExpr() = %q; want %q", got, want)
	}

	c2 := FieldPathComponent{
		Name:   "nest2_field",
		Target: nest1.Fields[0],
	}
	if got, want := c2.ValueExpr(), "Nest2Field"; got != want {
		t.Errorf("c2.ValueExpr() = %q; want %q", got, want)
	}
	if got, want := c2.ValueExpr(), "Nest2Field"; got != want {
		t.Errorf("c2.ValueExpr() = %q; want %q", got, want)
	}

	fp := FieldPath{
		c1, c2, c1, FieldPathComponent{
			Name:   "terminal_field",
			Target: nest1.Fields[1],
		},
	}
	if got, want := fp.AssignableExpr("resp", "example"), "resp.GetNestField().Nest2Field.GetNestField().TerminalField"; got != want {
		t.Errorf("fp.AssignableExpr(%q) = %q; want %q", "resp", got, want)
	}

	fp2 := FieldPath{
		c2, c1, c2, FieldPathComponent{
			Name:   "terminal_field",
			Target: nest2.Fields[1],
		},
	}
	if got, want := fp2.AssignableExpr("resp", "example"), "resp.Nest2Field.GetNestField().Nest2Field.TerminalField"; got != want {
		t.Errorf("fp2.AssignableExpr(%q) = %q; want %q", "resp", got, want)
	}

	var fpEmpty FieldPath
	if got, want := fpEmpty.AssignableExpr("resp", "example"), "resp"; got != want {
		t.Errorf("fpEmpty.AssignableExpr(%q) = %q; want %q", "resp", got, want)
	}
}

func TestGoType(t *testing.T) {
	src := `
		name: 'example.proto'
		package: 'example'
		message_type <
			name: 'Message'
			field <
				name: 'field'
				type: TYPE_STRING
				number: 1
			>
		>,
		enum_type <
			name: 'EnumName'
		>,
	`

	var fd descriptorpb.FileDescriptorProto
	if err := prototext.Unmarshal([]byte(src), &fd); err != nil {
		t.Fatalf("proto.UnmarshalText(%s, &fd) failed with %v; want success", src, err)
	}

	msg := &Message{
		DescriptorProto: fd.MessageType[0],
		Fields: []*Field{
			{FieldDescriptorProto: fd.MessageType[0].Field[0]},
		},
	}
	enum := &Enum{
		EnumDescriptorProto: fd.EnumType[0],
	}
	file := &File{
		FileDescriptorProto: &fd,
		GoPkg:               GoPackage{Path: "example", Name: "example"},
		Messages:            []*Message{msg},
		Enums:               []*Enum{enum},
	}
	crossLinkFixture(file)

	if got, want := msg.GoType("example"), "Message"; got != want {
		t.Errorf("msg.GoType() = %q; want %q", got, want)
	}
	if got, want := msg.GoType("extPackage"), "example.Message"; got != want {
		t.Errorf("msg.GoType() = %q; want %q", got, want)
	}
	msg.ForcePrefixedName = true
	if got, want := msg.GoType("example"), "example.Message"; got != want {
		t.Errorf("msg.GoType() = %q; want %q", got, want)
	}

	if got, want := enum.GoType("example"), "EnumName"; got != want {
		t.Errorf("enum.GoType() = %q; want %q", got, want)
	}
	if got, want := enum.GoType("extPackage"), "example.EnumName"; got != want {
		t.Errorf("enum.GoType() = %q; want %q", got, want)
	}
	enum.ForcePrefixedName = true
	if got, want := enum.GoType("example"), "example.EnumName"; got != want {
		t.Errorf("enum.GoType() = %q; want %q", got, want)
	}

}
