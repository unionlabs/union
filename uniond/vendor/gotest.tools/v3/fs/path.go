package fs

import (
	"bytes"
	"io"
	"os"

	"gotest.tools/v3/assert"
)

// resourcePath is an adaptor for resources so they can be used as a Path
// with PathOps.
type resourcePath struct{}

func (p *resourcePath) Path() string {
	return "manifest: not a filesystem path"
}

func (p *resourcePath) Remove() {}

type filePath struct {
	resourcePath
	file *file
}

func (p *filePath) SetContent(content io.ReadCloser) {
	p.file.content = content
}

func (p *filePath) SetUID(uid uint32) {
	p.file.uid = uid
}

func (p *filePath) SetGID(gid uint32) {
	p.file.gid = gid
}

type directoryPath struct {
	resourcePath
	directory *directory
}

func (p *directoryPath) SetUID(uid uint32) {
	p.directory.uid = uid
}

func (p *directoryPath) SetGID(gid uint32) {
	p.directory.gid = gid
}

func (p *directoryPath) AddSymlink(path, target string) error {
	p.directory.items[path] = &symlink{
		resource: newResource(defaultSymlinkMode),
		target:   target,
	}
	return nil
}

func (p *directoryPath) AddFile(path string, ops ...PathOp) error {
	newFile := &file{resource: newResource(0)}
	p.directory.items[path] = newFile
	exp := &filePath{file: newFile}
	return applyPathOps(exp, ops)
}

func (p *directoryPath) AddGlobFiles(glob string, ops ...PathOp) error {
	newFile := &file{resource: newResource(0)}
	newFilePath := &filePath{file: newFile}
	p.directory.filepathGlobs[glob] = newFilePath
	return applyPathOps(newFilePath, ops)
}

func (p *directoryPath) AddDirectory(path string, ops ...PathOp) error {
	newDir := newDirectoryWithDefaults()
	p.directory.items[path] = newDir
	exp := &directoryPath{directory: newDir}
	return applyPathOps(exp, ops)
}

// Expected returns a [Manifest] with a directory structured created by ops. The
// [PathOp] operations are applied to the manifest as expectations of the
// filesystem structure and properties.
func Expected(t assert.TestingT, ops ...PathOp) Manifest {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}

	newDir := newDirectoryWithDefaults()
	e := &directoryPath{directory: newDir}
	assert.NilError(t, applyPathOps(e, ops))
	return Manifest{root: newDir}
}

func newDirectoryWithDefaults() *directory {
	return &directory{
		resource:      newResource(defaultRootDirMode),
		items:         make(map[string]dirEntry),
		filepathGlobs: make(map[string]*filePath),
	}
}

func newResource(mode os.FileMode) resource {
	return resource{
		mode: mode,
		uid:  currentUID(),
		gid:  currentGID(),
	}
}

func currentUID() uint32 {
	return normalizeID(os.Getuid())
}

func currentGID() uint32 {
	return normalizeID(os.Getgid())
}

func normalizeID(id int) uint32 {
	// ids will be -1 on windows
	if id < 0 {
		return 0
	}
	return uint32(id)
}

var anyFileContent = io.NopCloser(bytes.NewReader(nil))

// MatchAnyFileContent is a [PathOp] that updates a [Manifest] so that the file
// at path may contain any content.
func MatchAnyFileContent(path Path) error {
	if m, ok := path.(*filePath); ok {
		m.SetContent(anyFileContent)
	}
	return nil
}

// MatchContentIgnoreCarriageReturn is a [PathOp] that ignores cariage return
// discrepancies.
func MatchContentIgnoreCarriageReturn(path Path) error {
	if m, ok := path.(*filePath); ok {
		m.file.ignoreCariageReturn = true
	}
	return nil
}

const anyFile = "*"

// MatchExtraFiles is a [PathOp] that updates a [Manifest] to allow a directory
// to contain unspecified files.
func MatchExtraFiles(path Path) error {
	if m, ok := path.(*directoryPath); ok {
		return m.AddFile(anyFile)
	}
	return nil
}

// CompareResult is the result of comparison.
//
// See [gotest.tools/v3/assert/cmp.StringResult] for a convenient implementation of
// this interface.
type CompareResult interface {
	Success() bool
	FailureMessage() string
}

// MatchFileContent is a [PathOp] that updates a [Manifest] to use the provided
// function to determine if a file's content matches the expectation.
func MatchFileContent(f func([]byte) CompareResult) PathOp {
	return func(path Path) error {
		if m, ok := path.(*filePath); ok {
			m.file.compareContentFunc = f
		}
		return nil
	}
}

// MatchFilesWithGlob is a [PathOp] that updates a [Manifest] to match files using
// glob pattern, and check them using the ops.
func MatchFilesWithGlob(glob string, ops ...PathOp) PathOp {
	return func(path Path) error {
		if m, ok := path.(*directoryPath); ok {
			return m.AddGlobFiles(glob, ops...)
		}
		return nil
	}
}

// anyFileMode is represented by uint32_max
const anyFileMode os.FileMode = 4294967295

// MatchAnyFileMode is a [PathOp] that updates a [Manifest] so that the resource at path
// will match any file mode.
func MatchAnyFileMode(path Path) error {
	if m, ok := path.(manifestResource); ok {
		m.SetMode(anyFileMode)
	}
	return nil
}
