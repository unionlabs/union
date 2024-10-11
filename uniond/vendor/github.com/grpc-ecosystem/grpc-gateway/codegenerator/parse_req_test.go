package codegenerator_test

import (
	"bytes"
	"fmt"
	"io"
	"reflect"
	"strings"
	"testing"

	"github.com/golang/protobuf/proto"
	plugin "github.com/golang/protobuf/protoc-gen-go/plugin"
	"github.com/grpc-ecosystem/grpc-gateway/codegenerator"
)

var parseReqTests = []struct {
	name string
	in   io.Reader
	out  *plugin.CodeGeneratorRequest
	err  error
}{
	{
		"Empty input should produce empty output",
		mustGetReader(&plugin.CodeGeneratorRequest{}),
		&plugin.CodeGeneratorRequest{},
		nil,
	},
	{
		"Invalid reader should produce error",
		&invalidReader{},
		nil,
		fmt.Errorf("failed to read code generator request: invalid reader"),
	},
	{
		"Invalid proto message should produce error",
		strings.NewReader("{}"),
		nil,
		fmt.Errorf("failed to unmarshal code generator request: unexpected EOF"),
	},
}

func TestParseRequest(t *testing.T) {
	for _, tt := range parseReqTests {
		t.Run(tt.name, func(t *testing.T) {
			out, err := codegenerator.ParseRequest(tt.in)
			if !reflect.DeepEqual(err, tt.err) {
				t.Errorf("got %v, want %v", err, tt.err)
			}
			if err == nil && !reflect.DeepEqual(*out, *tt.out) {
				t.Errorf("got %v, want %v", *out, *tt.out)
			}
		})
	}
}

func mustGetReader(pb proto.Message) io.Reader {
	b, err := proto.Marshal(pb)
	if err != nil {
		panic(err)
	}
	return bytes.NewBuffer(b)
}

type invalidReader struct {
}

func (*invalidReader) Read(p []byte) (int, error) {
	return 0, fmt.Errorf("invalid reader")
}
