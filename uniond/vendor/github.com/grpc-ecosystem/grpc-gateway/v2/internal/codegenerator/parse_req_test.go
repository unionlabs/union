package codegenerator_test

import (
	"bytes"
	"errors"
	"io"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/codegenerator"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/pluginpb"
)

var parseReqTests = []struct {
	name      string
	in        io.Reader
	out       *pluginpb.CodeGeneratorRequest
	expectErr bool
}{
	{
		"Empty input should produce empty output",
		mustGetReader(&pluginpb.CodeGeneratorRequest{}),
		&pluginpb.CodeGeneratorRequest{},
		false,
	},
	{
		"Invalid reader should produce error",
		&invalidReader{},
		nil,
		true,
	},
	{
		"Invalid proto message should produce error",
		strings.NewReader("{}"),
		nil,
		true,
	},
}

func TestParseRequest(t *testing.T) {
	for _, tt := range parseReqTests {
		t.Run(tt.name, func(t *testing.T) {
			out, err := codegenerator.ParseRequest(tt.in)
			if tt.expectErr && err == nil {
				t.Error("did not error as expected")
			}
			if diff := cmp.Diff(out, tt.out, protocmp.Transform()); diff != "" {
				t.Errorf(diff)
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
	return 0, errors.New("invalid reader")
}
