package getter

import (
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

var testHasHg bool

func init() {
	if _, err := exec.LookPath("hg"); err == nil {
		testHasHg = true
	}
}

func TestHgGetter_impl(t *testing.T) {
	var _ Getter = new(HgGetter)
}

func TestHgGetter(t *testing.T) {
	if !testHasHg {
		t.Log("hg not found, skipping")
		t.Skip()
	}

	g := new(HgGetter)
	dst := tempDir(t)

	// With a dir that doesn't exist
	if err := g.Get(dst, testModuleURL("basic-hg")); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "main.tf")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestHgGetter_branch(t *testing.T) {
	if !testHasHg {
		t.Log("hg not found, skipping")
		t.Skip()
	}

	g := new(HgGetter)
	dst := tempDir(t)

	url := testModuleURL("basic-hg")
	q := url.Query()
	q.Add("rev", "test-branch")
	url.RawQuery = q.Encode()

	if err := g.Get(dst, url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "main_branch.tf")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Get again should work
	if err := g.Get(dst, url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "main_branch.tf")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestHgGetter_GetFile(t *testing.T) {
	if !testHasHg {
		t.Log("hg not found, skipping")
		t.Skip()
	}

	g := new(HgGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	// Download
	if err := g.GetFile(dst, testModuleURL("basic-hg/foo.txt")); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	if _, err := os.Stat(dst); err != nil {
		t.Fatalf("err: %s", err)
	}
	assertContents(t, dst, "Hello\n")
}

func TestHgGetter_HgArgumentsNotAllowed(t *testing.T) {
	if !testHasHg {
		t.Log("hg not found, skipping")
		t.Skip()
	}

	g := new(HgGetter)

	// If arguments are allowed in the destination, this Get call will fail
	dst := "--config=alias.clone=!false"
	defer os.RemoveAll(dst)
	err := g.Get(dst, testModuleURL("basic-hg"))
	if err != nil {
		t.Fatalf("Expected no err, got: %s", err)
	}

	dst = tempDir(t)
	// Test arguments passed into the `rev` parameter
	// This clone call will fail regardless, but an exit code of 1 indicates
	// that the `false` command executed
	// We are expecting an hg parse error
	err = g.Get(dst, testModuleURL("basic-hg?rev=--config=alias.update=!false"))
	if err != nil {
		if !strings.Contains(err.Error(), "hg: parse error") {
			t.Fatalf("Expected no err, got: %s", err)
		}
	}

	dst = tempDir(t)
	// Test arguments passed in the repository URL
	// This Get call will fail regardless, but it should fail
	// because the repository can't be found.
	// Other failures indicate that hg interpretted the argument passed in the URL
	err = g.Get(dst, &url.URL{Path: "--config=alias.clone=false"})
	if err != nil {
		if !strings.Contains(err.Error(), "repository --config=alias.clone=false not found") {
			t.Fatalf("Expected no err, got: %s", err)
		}
	}

}
