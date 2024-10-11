package mergedregistry_test

import (
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"strings"
	"testing"

	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/descriptorpb"

	"github.com/cosmos/gogoproto/proto"
	_ "github.com/cosmos/gogoproto/types"
)

func TestMergedRegistry(t *testing.T) {
	reg, err := proto.MergedRegistry()
	if err != nil {
		t.Error(err)
	}
	// There are 11 .proto files in `google/protobuf` directory.
	if reg.NumFiles() != 11 {
		t.Error(fmt.Errorf("expected 11 files, got %d", reg.NumFiles()))
	}
}

func TestMergedFileDescriptorsWithValidation(t *testing.T) {
	t.Run("correct merge", func(t *testing.T) {
		t.Parallel()

		appFDs := proto.GogoResolver.(*protoregistry.Files)
		globalFiles := protoregistry.GlobalFiles

		fdSet, err := proto.MergedFileDescriptorsWithValidation(globalFiles, appFDs)
		if err != nil {
			t.Fatal(err)
		}

		// In this case, addition is fine since we know up front
		// that there is no overlap in globalFiles and appFDs.
		wantSize := globalFiles.NumFiles() + appFDs.NumFiles()

		if len(fdSet.File) != wantSize {
			t.Fatalf("wrong merged fd count: got %d, want %d", len(fdSet.File), wantSize)
		}

		gotNames := make(map[string]struct{}, wantSize)
		for _, fd := range fdSet.File {
			gotNames[fd.GetName()] = struct{}{}
		}

		globalFiles.RangeFiles(func(fileDescriptor protoreflect.FileDescriptor) bool {
			_, ok := gotNames[fileDescriptor.Path()]
			if !ok {
				t.Fatalf("global path %s not in merged file descriptor set", fileDescriptor.Path())
			}

			return false
		})

		appFDs.RangeFiles(func(fileDescriptor protoreflect.FileDescriptor) bool {
			_, ok := gotNames[fileDescriptor.Path()]
			if !ok {
				t.Fatalf("app path %s not in merged file descriptor set", fileDescriptor.Path())
			}

			return true
		})
	})

	t.Run("debug error on import path for global files", func(t *testing.T) {
		t.Parallel()

		// Find the existing file descriptor in the real global set.
		existingFD, err := protoregistry.GlobalFiles.FindFileByPath("google/protobuf/descriptor.proto")
		if err != nil {
			t.Fatal(err)
		}

		// Make a new global file set, and register one FD with an invalid path.
		gf := new(protoregistry.Files)
		if err := gf.RegisterFile(&pathOverrideFileDescriptor{
			OverridePath:   "example.com/foo/bar",
			FileDescriptor: existingFD,
		}); err != nil {
			t.Fatal(err)
		}

		// Merging just this one global should error due to an invalid path.
		_, err = proto.MergedFileDescriptorsWithValidation(gf, nil)
		if err == nil {
			t.Fatal("expected error when merging global with invalid path, but did not get error")
		}
		if !strings.Contains(err.Error(), "does not start with expected") {
			t.Fatalf("expected error to mention 'does not start with expected'; got %q", err.Error())
		}
	})

	t.Run("debug error on import path for app files", func(t *testing.T) {
		t.Parallel()

		fdBytes := proto.AllFileDescriptors()["google/protobuf/any.proto"]
		appFDs := map[string][]byte{
			// Decode and re-encode the proto description for the FD we picked,
			// giving it a name that will fail validation.
			"example.com/foo/bar": rewriteGzippedFDProto(fdBytes, "example.com/foo/bar"),
		}

		appFiles := fileDescriptorMapToFiles(t, appFDs)

		// Merging just this one global should error due to an invalid path.
		_, err := proto.MergedFileDescriptorsWithValidation(nil, appFiles)
		if err == nil {
			t.Fatal("expected error when merging app FD with invalid path, but did not get error")
		}
		if !strings.Contains(err.Error(), "does not start with expected") {
			t.Fatalf("expected error to mention 'does not start with expected'; got %q", err.Error())
		}
	})

	t.Run("debug error on diff in global and app FDs", func(t *testing.T) {
		t.Parallel()

		// Existing appFDs has one entry.
		appFDs := proto.AllFileDescriptors()

		anyBytes := appFDs["google/protobuf/any.proto"]

		// Decode the copy.
		gzr := new(gzip.Reader)
		if err := gzr.Reset(bytes.NewReader(anyBytes)); err != nil {
			t.Fatal(err)
		}

		buf := new(bytes.Buffer)
		if _, err := buf.ReadFrom(gzr); err != nil {
			t.Fatal(err)
		}

		modFD := &descriptorpb.FileDescriptorProto{}
		if err := protov2.Unmarshal(buf.Bytes(), modFD); err != nil {
			t.Fatal(err)
		}

		// And drop one of the messages from the copy.
		modFD.MessageType = modFD.MessageType[1:]

		gf := new(protoregistry.Files)
		modF, err := protodesc.NewFile(modFD, gf)
		if err != nil {
			t.Fatal(err)
		}
		gf.RegisterFile(modF)

		_, err = proto.MergedFileDescriptorsWithValidation(gf, proto.GogoResolver.(*protoregistry.Files)) // merged.File is slice of *descriptorpb.FileDescriptorProto
		if err == nil {
			t.Fatal("expected error when merging app FD mismatched with global FD, but did not get error")
		}
		if !strings.Contains(err.Error(), "file descriptor mismatches") {
			t.Fatalf("expected error to mention 'file descriptor mismatches'; got %q", err.Error())
		}
	})
}

// pathOverrideFileDescriptor wraps a protoreflect.FileDescriptor
// to provide a different value from its Path() method.
type pathOverrideFileDescriptor struct {
	OverridePath string

	protoreflect.FileDescriptor
}

func (d *pathOverrideFileDescriptor) Path() string {
	return d.OverridePath
}

// rewriteGzippedFDProto decodes bz, changes the Name to newName,
// and then returns the re-encoded result.
func rewriteGzippedFDProto(bz []byte, newName string) []byte {
	gzr := new(gzip.Reader)
	if err := gzr.Reset(bytes.NewReader(bz)); err != nil {
		panic(err)
	}

	buf := new(bytes.Buffer)
	if _, err := buf.ReadFrom(gzr); err != nil {
		panic(err)
	}

	fd := &descriptorpb.FileDescriptorProto{}
	if err := protov2.Unmarshal(buf.Bytes(), fd); err != nil {
		panic(err)
	}

	fd.Name = &newName

	bz, err := protov2.Marshal(fd)
	if err != nil {
		panic(err)
	}

	buf.Reset()
	gzw := gzip.NewWriter(buf)
	if _, err := gzw.Write(bz); err != nil {
		panic(err)
	}
	if err := gzw.Close(); err != nil {
		panic(err)
	}

	return buf.Bytes()
}

func fileDescriptorMapToFiles(tb testing.TB, m map[string][]byte) *protoregistry.Files {
	files := new(protoregistry.Files)
	for _, bz := range m {
		gzr, err := gzip.NewReader(bytes.NewReader(bz))
		if err != nil {
			tb.Fatal(err)
		}

		unzipped, err := io.ReadAll(gzr)
		if err != nil {
			tb.Fatal(err)
		}

		fd := &descriptorpb.FileDescriptorProto{}
		if err := protov2.Unmarshal(unzipped, fd); err != nil {
			tb.Fatal(err)
		}

		file, err := protodesc.FileOptions{AllowUnresolvable: true}.New(fd, files)
		if err != nil {
			tb.Fatal(err)
		}

		err = files.RegisterFile(file)
		if err != nil {
			tb.Fatal(err)
		}
	}

	return files
}
