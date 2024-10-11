package getter

import (
	"io/ioutil"
	"net/url"
	"os"
	"path/filepath"
	"testing"
)

// Note for external contributors: In order to run the GCS test suite, you will only be able to be run
// in GitHub Actions when you open a PR.

func TestGCSGetter_impl(t *testing.T) {
	var _ Getter = new(GCSGetter)
}

func TestGCSGetter(t *testing.T) {
	g := new(GCSGetter)
	dst := tempDir(t)

	// With a dir that doesn't exist
	err := g.Get(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "main.tf")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGCSGetter_subdir(t *testing.T) {
	g := new(GCSGetter)
	dst := tempDir(t)

	// With a dir that doesn't exist
	err := g.Get(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/subfolder"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the sub file exists
	subPath := filepath.Join(dst, "sub.tf")
	if _, err := os.Stat(subPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGCSGetter_GetFile(t *testing.T) {
	g := new(GCSGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	// Download
	err := g.GetFile(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/main.tf"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	if _, err := os.Stat(dst); err != nil {
		t.Fatalf("err: %s", err)
	}
	assertContents(t, dst, "# Main\n")
}

func TestGCSGetter_GetGenerationFile(t *testing.T) {
	g := new(GCSGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	// Download Previous Version
	err := g.GetFile(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/versioned.txt#1664282135302009"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify contents are valid for this generation
	content, err := ioutil.ReadFile(dst)
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	if string(content) != "old\n" {
		t.Fatalf("expected file contents from generation to be `old` but got `%s`", content)
	}

	// Download Current Version
	err = g.GetFile(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/versioned.txt#1664282167903672"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify contents are valid for this generation
	content, err = ioutil.ReadFile(dst)
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	if string(content) != "new\n" {
		t.Fatalf("expected file contents from generation to be `new` but got `%s`", content)
	}

}

func TestGCSGetter_GetFile_notfound(t *testing.T) {
	g := new(GCSGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	// Download
	err := g.GetFile(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/404.tf"))
	if err == nil {
		t.Fatalf("expected error, got none")
	}
}

func TestGCSGetter_ClientMode_dir(t *testing.T) {
	g := new(GCSGetter)

	// Check client mode on a key prefix with only a single key.
	mode, err := g.ClientMode(
		testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/subfolder"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	if mode != ClientModeDir {
		t.Fatal("expect ClientModeDir")
	}
}

func TestGCSGetter_ClientMode_file(t *testing.T) {
	g := new(GCSGetter)

	// Check client mode on a key prefix which contains sub-keys.
	mode, err := g.ClientMode(
		testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/subfolder/sub.tf"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	if mode != ClientModeFile {
		t.Fatal("expect ClientModeFile")
	}
}

func TestGCSGetter_ClientMode_notfound(t *testing.T) {
	g := new(GCSGetter)

	// Check the client mode when a non-existent key is looked up. This does not
	// return an error, but rather should just return the file mode.
	mode, err := g.ClientMode(
		testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/foobar"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	if mode != ClientModeFile {
		t.Fatal("expect ClientModeFile")
	}
}

func TestGCSGetter_Url(t *testing.T) {
	var gcstests = []struct {
		name   string
		url    string
		bucket string
		path   string
	}{
		{
			name:   "test1",
			url:    "gcs::https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/foo/null.zip",
			bucket: "hc-go-getter-test",
			path:   "go-getter/foo/null.zip",
		},
	}

	for i, pt := range gcstests {
		t.Run(pt.name, func(t *testing.T) {
			g := new(GCSGetter)
			forced, src := getForcedGetter(pt.url)
			u, err := url.Parse(src)

			if err != nil {
				t.Errorf("test %d: unexpected error: %s", i, err)
			}
			if forced != "gcs" {
				t.Fatalf("expected forced protocol to be gcs")
			}

			bucket, path, _, err := g.parseURL(u)

			if err != nil {
				t.Fatalf("err: %s", err)
			}

			if bucket != pt.bucket {
				t.Fatalf("expected %s, got %s", pt.bucket, bucket)
			}
			if path != pt.path {
				t.Fatalf("expected %s, got %s", pt.path, path)
			}
		})
	}
}

func TestGCSGetter_GetFile_OAuthAccessToken(t *testing.T) {
	if os.Getenv("GOOGLE_OAUTH_ACCESS_TOKEN") == "" {
		t.Skip("Skipping; set GOOGLE_OAUTH_ACCESS_TOKEN to run")
	}
	g := new(GCSGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	// Download
	err := g.GetFile(
		dst, testURL("https://www.googleapis.com/storage/v1/hc-go-getter-test/go-getter/folder/main.tf"))
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	if _, err := os.Stat(dst); err != nil {
		t.Fatalf("err: %s", err)
	}
	assertContents(t, dst, "# Main\n")
}
