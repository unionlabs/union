package proto

import (
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"strings"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/exp/slices"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/descriptorpb"
)

// MergedFileDescriptors returns a single FileDescriptorSet containing all the
// file descriptors registered with both gogoproto and protoregistry. When
// merging it also performs the following checks:
// - check that all file descriptors' import paths are correct (i.e. match
// their fully-qualified package name). A warning is logged if this check fails.
// - check that if both gogo and protoregistry import the same file descriptor,
// that these are identical under proto.Equal. A warning is logged if this
// check fails. If there is a mismatch, the final merged file descriptor set will contain the
// protoregistry file descriptor, and discard the gogo one.
func MergedFileDescriptors() (*descriptorpb.FileDescriptorSet, error) {
	return mergedFileDescriptors(false)
}

// DebugFileDescriptorsMismatch is a helper function to debug file descriptor
// mismatches. It returns an error if there are any mismatches.
func DebugFileDescriptorsMismatch() error {
	_, err := mergedFileDescriptors(true)
	return err
}

func mergedFileDescriptors(debug bool) (*descriptorpb.FileDescriptorSet, error) {
	fds := &descriptorpb.FileDescriptorSet{}

	// While combing through the file descriptors, we'll also log any errors
	// we encounter.
	var (
		checkImportErr []string
		diffErr        []string
	)

	// Add protoregistry file descriptors to our final file descriptor set.
	protoregistry.GlobalFiles.RangeFiles(func(fileDescriptor protoreflect.FileDescriptor) bool {
		fd := protodesc.ToFileDescriptorProto(fileDescriptor)
		if err := CheckImportPath(fd.GetName(), fd.GetPackage()); err != nil {
			checkImportErr = append(checkImportErr, err.Error())
		}

		fds.File = append(fds.File, protodesc.ToFileDescriptorProto(fileDescriptor))

		return true
	})

	// load gogo proto file descriptors
	gogoFds := AllFileDescriptors()
	for _, compressedBz := range gogoFds {
		rdr, err := gzip.NewReader(bytes.NewReader(compressedBz))
		if err != nil {
			return nil, err
		}

		bz, err := io.ReadAll(rdr)
		if err != nil {
			return nil, err
		}

		fd := &descriptorpb.FileDescriptorProto{}
		err = protov2.Unmarshal(bz, fd)
		if err != nil {
			return nil, err
		}

		err = CheckImportPath(fd.GetName(), fd.GetPackage())
		if err != nil {
			checkImportErr = append(checkImportErr, err.Error())
		}

		// If it's not in the protoregistry file descriptors, add it.
		protoregFd, err := protoregistry.GlobalFiles.FindFileByPath(*fd.Name)
		// If we already loaded gogo's file descriptor, compare that the 2
		// are strictly equal, or else log a warning.
		if err != nil {
			// If we have a NotFound error, we add this file descriptor to the
			// final file descriptor set.
			if errors.Is(err, protoregistry.NotFound) {
				fds.File = append(fds.File, fd)
			} else {
				return nil, err
			}
		} else {
			// If there's a mismatch, we log a warning. If there was no
			// mismatch, then we do nothing, and take the protoregistry file
			// descriptor as the correct one.
			if !protov2.Equal(protodesc.ToFileDescriptorProto(protoregFd), fd) {
				diff := cmp.Diff(protodesc.ToFileDescriptorProto(protoregFd), fd, protocmp.Transform())
				diffErr = append(diffErr, fmt.Sprintf("Mismatch in %s:\n%s", *fd.Name, diff))
			}
		}
	}

	if debug {
		errStr := new(bytes.Buffer)
		if len(checkImportErr) > 0 {
			fmt.Fprintf(errStr, "Got %d file descriptor import path errors:\n\t%s\n", len(checkImportErr), strings.Join(checkImportErr, "\n\t"))
		}
		if len(diffErr) > 0 {
			fmt.Fprintf(errStr, "Got %d file descriptor mismatches. Make sure gogoproto and protoregistry use the same .proto files. '-' lines are from protoregistry, '+' lines from gogo's registry.\n\n\t%s\n", len(diffErr), strings.Join(diffErr, "\n\t"))
		}
		if errStr.Len() > 0 {
			return nil, fmt.Errorf(errStr.String())
		}
	}

	slices.SortFunc(fds.File, func(x, y *descriptorpb.FileDescriptorProto) bool {
		return *x.Name < *y.Name
	})

	return fds, nil
}

// MergedRegistry returns a *protoregistry.Files that acts as a single registry
// which contains all the file descriptors registered with both gogoproto and
// protoregistry (the latter taking precendence if there's a mismatch).
func MergedRegistry() (*protoregistry.Files, error) {
	fds, err := MergedFileDescriptors()
	if err != nil {
		return nil, err
	}

	return protodesc.NewFiles(fds)
}

// CheckImportPath checks that the import path of the given file descriptor
// matches its fully qualified package name. To mimic gogo's old behavior, the
// fdPackage string can be empty.
//
// Example:
// Proto file "google/protobuf/descriptor.proto" should be imported
// from OS path ./google/protobuf/descriptor.proto, relatively to a protoc
// path folder (-I flag).
func CheckImportPath(fdName, fdPackage string) error {
	expectedPrefix := strings.ReplaceAll(fdPackage, ".", "/") + "/"
	if !strings.HasPrefix(fdName, expectedPrefix) {
		return fmt.Errorf("file name %s does not start with expected %s; please make sure your folder structure matches the proto files fully-qualified names", fdName, expectedPrefix)
	}

	return nil
}
