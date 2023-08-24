package proto

import (
	"bytes"
	"errors"
	"fmt"
	"runtime"
	"strings"
	"sync"

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
// file descriptors registered with the given globalFiles and appFiles.
//
// In contrast to MergedFileDescriptorsWithValidation,
// MergedFileDescriptors does not validate import paths
func MergedFileDescriptors(globalFiles *protoregistry.Files, gogoFiles *protoregistry.Files) (*descriptorpb.FileDescriptorSet, error) {
	return mergedFileDescriptors(globalFiles, gogoFiles, false)
}

// MergedFileDescriptorsWithValidation returns a single FileDescriptorSet containing all the
// file descriptors registered with the given globalFiles and appFiles.
//
// If there are any incorrect import paths that do not match
// the fully qualified package name, or if there is a common file descriptor
// that differs accross globalFiles and appFiles, an error is returned.
func MergedFileDescriptorsWithValidation(globalFiles *protoregistry.Files, gogoFiles *protoregistry.Files) (*descriptorpb.FileDescriptorSet, error) {
	return mergedFileDescriptors(globalFiles, gogoFiles, true)
}

// MergedGlobalFileDescriptors calls MergedFileDescriptors
// with [protoregistry.GlobalFiles] and all files
// registered through [RegisterFile].
func MergedGlobalFileDescriptors() (*descriptorpb.FileDescriptorSet, error) {
	return MergedFileDescriptors(protoregistry.GlobalFiles, gogoProtoRegistry)
}

// MergedGlobalFileDescriptorsWithValidation calls MergedFileDescriptorsWithValidation
// with [protoregistry.GlobalFiles] and all files
// registered through [RegisterFile].
func MergedGlobalFileDescriptorsWithValidation() (*descriptorpb.FileDescriptorSet, error) {
	return MergedFileDescriptorsWithValidation(protoregistry.GlobalFiles, gogoProtoRegistry)
}

// MergedRegistry returns a *protoregistry.Files that acts as a single registry
// which contains all the file descriptors registered with both gogoproto and
// protoregistry (the latter taking precendence if there's a mismatch).
func MergedRegistry() (*protoregistry.Files, error) {
	fds, err := MergedGlobalFileDescriptors()
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

// descriptorErrorCollector collects errors sent on its exported channel fields.
// If any errors occur, they are collected on the err field.
type descriptorErrorCollector struct {
	validate bool

	// Close the quit channel to request the collection goroutine to stop.
	quit chan struct{}

	// The done channel will be closed once the collection goroutine has finished.
	done chan struct{}

	ProcessErrCh chan error
	ImportErrCh  chan error
	DiffCh       chan string

	// Set at the end of collect().
	err error
}

// newDescriptorErrorCollector initializes and returns a descriptorErrorCollector.
// It starts a goroutine running the descriptorErrorCollector's collect method in the background.
func newDescriptorErrorCollector(chanSize int, validate bool) *descriptorErrorCollector {
	c := &descriptorErrorCollector{
		validate: validate,

		quit: make(chan struct{}),
		done: make(chan struct{}),

		ProcessErrCh: make(chan error, chanSize),
		ImportErrCh:  make(chan error, chanSize),
		DiffCh:       make(chan string, chanSize),
	}
	go c.collect()
	return c
}

// collect runs in its own goroutine,
// collecting process errors and import path and file descriptor differences.
// If any of those occur, it assigns to c.err.
// Stop the goroutine by closing c.quit.
// The goroutine closes c.done when it returns.
func (c *descriptorErrorCollector) collect() {
	defer close(c.done)

	// Write the process errors to buf first -- no need to hold them in a separate slice.
	var buf bytes.Buffer

	// Don't know the incoming order of any errors, so hold the import and diff errors
	// in their own slice until the quit signal is received.
	var importErrMsgs, diffs []string

LOOP:
	for {
		select {
		case <-c.quit:
			break LOOP

		case err := <-c.ProcessErrCh:
			// Always accept process errors (no need to check c.validate).
			// Accumulate them directly into buf since those always go in the front.
			fmt.Fprintf(&buf, "Failure during processing: %v\n", err)

		case err := <-c.ImportErrCh:
			if !c.validate {
				panic(fmt.Errorf("BUG: import error received when validate=false: %w", err))
			}
			importErrMsgs = append(importErrMsgs, err.Error())

		case diff := <-c.DiffCh:
			if !c.validate {
				panic(fmt.Errorf("BUG: diff received when validate=false: %s", diff))
			}
			diffs = append(diffs, diff)
		}
	}

	if buf.Len() == 0 && len(importErrMsgs) == 0 && len(diffs) == 0 {
		// No errors received. Stop here so we don't assign to c.err.
		return
	}

	if len(importErrMsgs) > 0 {
		fmt.Fprintf(&buf, "Got %d file descriptor import path errors:\n\t%s\n", len(importErrMsgs), strings.Join(importErrMsgs, "\n\t"))
	}
	if len(diffs) > 0 {
		fmt.Fprintf(&buf, "Got %d file descriptor mismatches. Make sure gogoproto and protoregistry use the same .proto files. '-' lines are from protoregistry, '+' lines from gogo's registry.\n\n\t%s\n", len(diffs), strings.Join(diffs, "\n\t"))
	}

	c.err = errors.New(buf.String())
}

// descriptorProcessor runs the heavy lifting for concurrent registry merging.
// See the mergedFileDescriptors function for how everything coordinates.
type descriptorProcessor struct {
	processWG    sync.WaitGroup
	globalFileCh chan protoreflect.FileDescriptor
	appFileCh    chan protoreflect.FileDescriptor

	fdWG sync.WaitGroup
	fdCh chan *descriptorpb.FileDescriptorProto
	fds  []*descriptorpb.FileDescriptorProto
}

// process reads from p.globalFileCh and p.appFileCh, processing each file descriptor as appropriate,
// and sends the processed file descriptors through p.fdCh for eventual return from mergedFileDescriptors.
// Any errors during processing are sent to ec.ProcessErrCh,
// which collects the errors also for possible return from mergedFileDescriptors.
//
// If validate is true, extra work is performed to validate import paths
// and to check validity of duplicated file descriptors.
//
// process is intended to be run in a goroutine.
func (p *descriptorProcessor) process(globalFiles *protoregistry.Files, ec *descriptorErrorCollector, validate bool) {
	defer p.processWG.Done()

	// Read the global files to exhaustion first.
	for fileDescriptor := range p.globalFileCh {
		fd := protodesc.ToFileDescriptorProto(fileDescriptor)
		if validate {
			if err := CheckImportPath(fd.GetName(), fd.GetPackage()); err != nil {
				// Track the import error but don't stop processing.
				// It is more helpful to present all the import errors,
				// rather than just stopping on the first one.
				ec.ImportErrCh <- err
			}
		}

		// Collect all the file descriptors in the collectFDs goroutine.
		p.fdCh <- fd
	}

	// Now handle all the app files.
	for gogoFd := range p.appFileCh {
		// If the app FD is not in protoregistry, we need to track it.
		gogoFdp := protodesc.ToFileDescriptorProto(gogoFd)
		if validate {
			if err := CheckImportPath(gogoFdp.GetName(), gogoFdp.GetPackage()); err != nil {
				// Track the import error but don't stop processing.
				// It is more helpful to present all the import errors,
				// rather than just stopping on the first one.
				ec.ImportErrCh <- err
			}
		}

		protoregFd, err := globalFiles.FindFileByPath(*gogoFdp.Name)
		if err != nil {
			if !errors.Is(err, protoregistry.NotFound) {
				// Non-nil error, and it wasn't a not found error.
				ec.ProcessErrCh <- err
				continue
			}
			// Otherwise it was a not found error, so add it.
			// At this point we can't validate.
			p.fdCh <- gogoFdp
			continue
		}

		if validate {
			fdp := protodesc.ToFileDescriptorProto(protoregFd)

			if !protov2.Equal(fdp, gogoFdp) {
				diff := cmp.Diff(fdp, gogoFdp, protocmp.Transform())
				ec.DiffCh <- fmt.Sprintf("Mismatch in %s:\n%s", *gogoFdp.Name, diff)
			}
		}
	}
}

// collectFDs runs in its own goroutine, exhausing p.fdCh to populate p.fds,
// and then sorting p.fds in-place.
func (p *descriptorProcessor) collectFDs() {
	defer p.fdWG.Done()

	for fd := range p.fdCh {
		p.fds = append(p.fds, fd)
	}

	slices.SortFunc(p.fds, func(x, y *descriptorpb.FileDescriptorProto) bool {
		return *x.Name < *y.Name
	})
}

// mergedFileDescriptors coordinates an instance of a descriptorProcessor
// and a descriptorErrorCollector to concurrently merge the file descriptors in globalFiles and appFiles,
// into a new *descriptorpb.FileDescriptorSet.
//
// If validate is true, do extra work to validate that import paths are properly formed
// and that "duplicated" file descriptors across globalFiles and appFiles
// are indeed identical, returning an error if either of those conditions are invalidated.
func mergedFileDescriptors(globalFiles *protoregistry.Files, gogoFiles *protoregistry.Files, validate bool) (*descriptorpb.FileDescriptorSet, error) {
	// GOMAXPROCS is the number of CPU cores available, by default.
	// Respect that setting as the number of CPU-bound goroutines,
	// and for channel sizes.
	nProcs := runtime.GOMAXPROCS(0)

	ec := newDescriptorErrorCollector(nProcs, validate)

	p := &descriptorProcessor{
		globalFileCh: make(chan protoreflect.FileDescriptor, nProcs),
		appFileCh:    make(chan protoreflect.FileDescriptor, nProcs),

		fdCh: make(chan *descriptorpb.FileDescriptorProto, nProcs),
		fds:  make([]*descriptorpb.FileDescriptorProto, 0, globalFiles.NumFiles()),
	}

	// Start the file-descriptor-processing goroutines.
	p.processWG.Add(nProcs)
	for i := 0; i < nProcs; i++ {
		go p.process(globalFiles, ec, validate)
	}

	// Start the goroutine that collects all the processed file descriptors.
	p.fdWG.Add(1)
	go p.collectFDs()

	// Now synchronously iterate through globalFiles,
	// sending the proto file descriptors to the processor goroutines.
	globalFiles.RangeFiles(func(fileDescriptor protoreflect.FileDescriptor) bool {
		p.globalFileCh <- fileDescriptor
		return true
	})
	// Signal that no more global files will be sent.
	close(p.globalFileCh)

	// Same for gogoFiles: send everything then signal app files are finished.
	gogoFiles.RangeFiles(func(fileDescriptor protoreflect.FileDescriptor) bool {
		p.appFileCh <- fileDescriptor
		return true
	})
	close(p.appFileCh)

	// Since we are done sending file descriptors and we have closed those channels,
	// wait for the processor goroutines to complete.
	p.processWG.Wait()

	// Now close the FD channel since the processors are done,
	// and no more processed FD values will be sent.
	close(p.fdCh)

	// Wait until FD collection is complete.
	p.fdWG.Wait()

	// Since FD collection is done, stop the error collector,
	// and if it found an error, return it.
	close(ec.quit)
	<-ec.done
	if ec.err != nil {
		return nil, ec.err
	}

	// Otherwise success.
	return &descriptorpb.FileDescriptorSet{
		File: p.fds,
	}, nil
}

// HybridResolver is a protodesc.Resolver that uses both protoregistry.GlobalFiles
// and the gogo proto global registry, checking protoregistry.GlobalFiles first and
// then gogo proto global registry.
var HybridResolver Resolver = &hybridResolver{}

// Resolver is a protodesc.Resolver that can range over all the files in the resolver.
type Resolver interface {
	protodesc.Resolver

	// RangeFiles calls f for each file descriptor in the resolver while f returns true.
	RangeFiles(f func(fileDescriptor protoreflect.FileDescriptor) bool)
}

type hybridResolver struct{}

var _ protodesc.Resolver = &hybridResolver{}

func (r *hybridResolver) FindFileByPath(path string) (protoreflect.FileDescriptor, error) {
	if fd, err := protoregistry.GlobalFiles.FindFileByPath(path); err == nil {
		return fd, nil
	}

	return gogoProtoRegistry.FindFileByPath(path)
}

func (r *hybridResolver) FindDescriptorByName(name protoreflect.FullName) (protoreflect.Descriptor, error) {
	if desc, err := protoregistry.GlobalFiles.FindDescriptorByName(name); err == nil {
		return desc, nil
	}

	return gogoProtoRegistry.FindDescriptorByName(name)
}

func (r *hybridResolver) RangeFiles(f func(fileDescriptor protoreflect.FileDescriptor) bool) {
	seen := make(map[protoreflect.FullName]bool, protoregistry.GlobalFiles.NumFiles())

	protoregistry.GlobalFiles.RangeFiles(func(fd protoreflect.FileDescriptor) bool {
		seen[fd.FullName()] = true
		return f(fd)
	})

	gogoProtoRegistry.RangeFiles(func(fd protoreflect.FileDescriptor) bool {
		if seen[fd.FullName()] {
			return true
		}
		return f(fd)
	})
}
