package fs

import (
	"bytes"
	"io"
	"os"
	"runtime"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"gotest.tools/v3/assert"
)

func TestManifestFromDir(t *testing.T) {
	var defaultFileMode os.FileMode = 0644
	var subDirMode = 0755 | os.ModeDir
	var jFileMode os.FileMode = 0600
	if runtime.GOOS == "windows" {
		defaultFileMode = 0666
		subDirMode = 0777 | os.ModeDir
		jFileMode = 0666
	}

	var userOps []PathOp
	var expectedUserResource = newResource(defaultFileMode)
	if os.Geteuid() == 0 {
		userOps = append(userOps, AsUser(1001, 1002))
		expectedUserResource = resource{mode: defaultFileMode, uid: 1001, gid: 1002}
	}

	srcDir := NewDir(t, t.Name(),
		WithFile("j", "content j", WithMode(0600)),
		WithDir("s",
			WithFile("k", "content k")),
		WithSymlink("f", "j"),
		WithFile("x", "content x", userOps...))
	defer srcDir.Remove()

	expected := Manifest{
		root: &directory{
			resource: newResource(defaultRootDirMode),
			items: map[string]dirEntry{
				"j": &file{
					resource: newResource(jFileMode),
					content:  readCloser("content j"),
				},
				"s": &directory{
					resource: newResource(subDirMode),
					items: map[string]dirEntry{
						"k": &file{
							resource: newResource(defaultFileMode),
							content:  readCloser("content k"),
						},
					},
					filepathGlobs: map[string]*filePath{},
				},
				"f": &symlink{
					resource: newResource(defaultSymlinkMode),
					target:   srcDir.Join("j"),
				},
				"x": &file{
					resource: expectedUserResource,
					content:  readCloser("content x"),
				},
			},
			filepathGlobs: map[string]*filePath{},
		},
	}
	actual := ManifestFromDir(t, srcDir.Path())
	assert.DeepEqual(t, actual, expected, cmpManifest)
	actual.root.items["j"].(*file).content.Close()
	actual.root.items["x"].(*file).content.Close()
	actual.root.items["s"].(*directory).items["k"].(*file).content.Close()
}

func TestSymlinks(t *testing.T) {
	rootDirectory := NewDir(t, "root",
		WithFile("foo.txt", "foo"),
		WithSymlink("foo.link", "foo.txt"))
	defer rootDirectory.Remove()
	expected := Expected(t,
		WithFile("foo.txt", "foo"),
		WithSymlink("foo.link", rootDirectory.Join("foo.txt")))
	assert.Assert(t, Equal(rootDirectory.Path(), expected))
}

var cmpManifest = cmp.Options{
	cmp.AllowUnexported(Manifest{}, resource{}, file{}, symlink{}, directory{}),
	cmp.Comparer(func(x, y io.ReadCloser) bool {
		if x == nil || y == nil {
			return x == y
		}
		xContent, err := io.ReadAll(x)
		if err != nil {
			return false
		}

		yContent, err := io.ReadAll(y)
		if err != nil {
			return false
		}
		return bytes.Equal(xContent, yContent)
	}),
}

func readCloser(s string) io.ReadCloser {
	return io.NopCloser(strings.NewReader(s))
}
