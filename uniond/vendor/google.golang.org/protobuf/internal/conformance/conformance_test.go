// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package conformance_test

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/proto"

	pb "google.golang.org/protobuf/internal/testprotos/conformance"
	epb "google.golang.org/protobuf/internal/testprotos/conformance/editions"
	empb "google.golang.org/protobuf/internal/testprotos/conformance/editionsmigration"
)

func init() {
	// When the environment variable RUN_AS_CONFORMANCE_PLUGIN is set,
	// we skip running the tests and instead act as a conformance plugin.
	// This allows the binary to pass itself to conformance.
	if os.Getenv("RUN_AS_CONFORMANCE_PLUGIN") == "1" {
		main()
		os.Exit(0)
	}
}

var (
	execute   = flag.Bool("execute", false, "execute the conformance test")
	protoRoot = flag.String("protoroot", os.Getenv("PROTOBUF_ROOT"), "The root of the protobuf source tree.")
)

func Test(t *testing.T) {
	if !*execute {
		t.SkipNow()
	}
	binPath := filepath.Join(*protoRoot, "bazel-bin", "conformance", "conformance_test_runner")
	cmd := exec.Command(binPath,
		"--failure_list", "failing_tests.txt",
		"--text_format_failure_list", "failing_tests_text_format.txt",
		"--enforce_recommended",
		"--maximum_edition", "2023",
		os.Args[0])
	cmd.Env = append(os.Environ(), "RUN_AS_CONFORMANCE_PLUGIN=1")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("execution error: %v\n\n%s", err, out)
	}
}

func main() {
	var sizeBuf [4]byte
	inbuf := make([]byte, 0, 4096)
	for {
		_, err := io.ReadFull(os.Stdin, sizeBuf[:])
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatalf("conformance: read request: %v", err)
		}
		size := binary.LittleEndian.Uint32(sizeBuf[:])
		if int(size) > cap(inbuf) {
			inbuf = make([]byte, size)
		}
		inbuf = inbuf[:size]
		if _, err := io.ReadFull(os.Stdin, inbuf); err != nil {
			log.Fatalf("conformance: read request: %v", err)
		}

		req := &pb.ConformanceRequest{}
		if err := proto.Unmarshal(inbuf, req); err != nil {
			log.Fatalf("conformance: parse request: %v", err)
		}
		res := handle(req)

		out, err := proto.Marshal(res)
		if err != nil {
			log.Fatalf("conformance: marshal response: %v", err)
		}
		binary.LittleEndian.PutUint32(sizeBuf[:], uint32(len(out)))
		if _, err := os.Stdout.Write(sizeBuf[:]); err != nil {
			log.Fatalf("conformance: write response: %v", err)
		}
		if _, err := os.Stdout.Write(out); err != nil {
			log.Fatalf("conformance: write response: %v", err)
		}
	}
}

func handle(req *pb.ConformanceRequest) (res *pb.ConformanceResponse) {
	var msg proto.Message = &pb.TestAllTypesProto2{}
	switch req.GetMessageType() {
	case "protobuf_test_messages.proto3.TestAllTypesProto3":
		msg = &pb.TestAllTypesProto3{}
	case "protobuf_test_messages.proto2.TestAllTypesProto2":
		msg = &pb.TestAllTypesProto2{}
	case "protobuf_test_messages.editions.TestAllTypesEdition2023":
		msg = &epb.TestAllTypesEdition2023{}
	case "protobuf_test_messages.editions.proto3.TestAllTypesProto3":
		msg = &empb.TestAllTypesProto3{}
	case "protobuf_test_messages.editions.proto2.TestAllTypesProto2":
		msg = &empb.TestAllTypesProto2{}
	default:
		panic(fmt.Sprintf("unknown message type: %s", req.GetMessageType()))
	}

	// Unmarshal the test message.
	var err error
	switch p := req.Payload.(type) {
	case *pb.ConformanceRequest_ProtobufPayload:
		err = proto.Unmarshal(p.ProtobufPayload, msg)
	case *pb.ConformanceRequest_JsonPayload:
		err = protojson.UnmarshalOptions{
			DiscardUnknown: req.TestCategory == pb.TestCategory_JSON_IGNORE_UNKNOWN_PARSING_TEST,
		}.Unmarshal([]byte(p.JsonPayload), msg)
	case *pb.ConformanceRequest_TextPayload:
		err = prototext.Unmarshal([]byte(p.TextPayload), msg)
	default:
		return &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_RuntimeError{
				RuntimeError: "unknown request payload type",
			},
		}
	}
	if err != nil {
		return &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_ParseError{
				ParseError: err.Error(),
			},
		}
	}

	// Marshal the test message.
	var b []byte
	switch req.RequestedOutputFormat {
	case pb.WireFormat_PROTOBUF:
		b, err = proto.Marshal(msg)
		res = &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_ProtobufPayload{
				ProtobufPayload: b,
			},
		}
	case pb.WireFormat_JSON:
		b, err = protojson.Marshal(msg)
		res = &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_JsonPayload{
				JsonPayload: string(b),
			},
		}
	case pb.WireFormat_TEXT_FORMAT:
		b, err = prototext.MarshalOptions{
			EmitUnknown: req.PrintUnknownFields,
		}.Marshal(msg)
		res = &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_TextPayload{
				TextPayload: string(b),
			},
		}
	default:
		return &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_RuntimeError{
				RuntimeError: "unknown output format",
			},
		}
	}
	if err != nil {
		return &pb.ConformanceResponse{
			Result: &pb.ConformanceResponse_SerializeError{
				SerializeError: err.Error(),
			},
		}
	}
	return res
}
