package getter

import (
	"bytes"
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	urlhelper "github.com/hashicorp/go-getter/helper/url"
)

var testHasGit bool

func init() {
	if _, err := exec.LookPath("git"); err == nil {
		testHasGit = true
	}
}

func TestGitGetter_impl(t *testing.T) {
	var _ Getter = new(GitGetter)
}

func TestGitGetter(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "basic")
	repo.commitFile("foo.txt", "hello")

	// With a dir that doesn't exist
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "foo.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_branch(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "branch")
	repo.git("checkout", "-b", "test-branch")
	repo.commitFile("branch.txt", "branch")

	q := repo.url.Query()
	q.Add("ref", "test-branch")
	repo.url.RawQuery = q.Encode()

	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "branch.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Get again should work
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "branch.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_commitID(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	// We're going to create different content on the main branch vs.
	// another branch here, so that below we can recognize if we
	// correctly cloned the commit actually requested (from the
	// "other branch"), not the one at HEAD.
	repo := testGitRepo(t, "commit_id")
	repo.git("checkout", "-b", "main-branch")
	repo.commitFile("wrong.txt", "Nope")
	repo.git("checkout", "-b", "other-branch")
	repo.commitFile("hello.txt", "Yep")
	commitID, err := repo.latestCommit()
	if err != nil {
		t.Fatal(err)
	}
	// Return to the main branch so that HEAD of this repository
	// will be that, rather than "test-branch".
	repo.git("checkout", "main-branch")

	q := repo.url.Query()
	q.Add("ref", commitID)
	repo.url.RawQuery = q.Encode()

	t.Logf("Getting %s", repo.url)
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "hello.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Get again should work
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "hello.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_remoteWithoutMaster(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "branch")
	repo.git("checkout", "-b", "test-branch")
	repo.commitFile("branch.txt", "branch")

	q := repo.url.Query()
	repo.url.RawQuery = q.Encode()

	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "branch.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	dst2 := tempDir(t)
	// Get again should work
	if err := g.Get(dst2, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "branch.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_shallowClone(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "upstream")
	repo.commitFile("upstream.txt", "0")
	repo.commitFile("upstream.txt", "1")

	// Specifiy a clone depth of 1
	q := repo.url.Query()
	q.Add("depth", "1")
	repo.url.RawQuery = q.Encode()

	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Assert rev-list count is '1'
	cmd := exec.Command("git", "rev-list", "HEAD", "--count")
	cmd.Dir = dst
	b, err := cmd.Output()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	out := strings.TrimSpace(string(b))
	if out != "1" {
		t.Fatalf("expected rev-list count to be '1' but got %v", out)
	}
}

func TestGitGetter_shallowCloneWithTag(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "upstream")
	repo.commitFile("v1.0.txt", "0")
	repo.git("tag", "v1.0")
	repo.commitFile("v1.1.txt", "1")

	// Specifiy a clone depth of 1 with a tag
	q := repo.url.Query()
	q.Add("ref", "v1.0")
	q.Add("depth", "1")
	repo.url.RawQuery = q.Encode()

	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Assert rev-list count is '1'
	cmd := exec.Command("git", "rev-list", "HEAD", "--count")
	cmd.Dir = dst
	b, err := cmd.Output()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	out := strings.TrimSpace(string(b))
	if out != "1" {
		t.Fatalf("expected rev-list count to be '1' but got %v", out)
	}

	// Verify the v1.0 file exists
	mainPath := filepath.Join(dst, "v1.0.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the v1.1 file does not exists
	mainPath = filepath.Join(dst, "v1.1.txt")
	if _, err := os.Stat(mainPath); err == nil {
		t.Fatalf("expected v1.1 file to not exist")
	}
}

func TestGitGetter_shallowCloneWithCommitID(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "upstream")
	repo.commitFile("v1.0.txt", "0")
	repo.git("tag", "v1.0")
	repo.commitFile("v1.1.txt", "1")

	commitID, err := repo.latestCommit()
	if err != nil {
		t.Fatal(err)
	}

	// Specify a clone depth of 1 with a naked commit ID
	// This is intentionally invalid: shallow clone always requires a named ref.
	q := repo.url.Query()
	q.Add("ref", commitID[:8])
	q.Add("depth", "1")
	repo.url.RawQuery = q.Encode()

	t.Logf("Getting %s", repo.url)
	err = g.Get(dst, repo.url)
	if err == nil {
		t.Fatalf("success; want error")
	}
	// We use a heuristic to generate an extra hint in the error message if
	// it looks like the user was trying to combine ref=COMMIT with depth.
	if got, want := err.Error(), "(note that setting 'depth' requires 'ref' to be a branch or tag name)"; !strings.Contains(got, want) {
		t.Errorf("missing error message hint\ngot: %s\nwant substring: %s", got, want)
	}
}

func TestGitGetter_branchUpdate(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	// First setup the state with a fresh branch
	repo := testGitRepo(t, "branch-update")
	repo.git("checkout", "-b", "test-branch")
	repo.commitFile("branch.txt", "branch")

	// Get the "test-branch" branch
	q := repo.url.Query()
	q.Add("ref", "test-branch")
	repo.url.RawQuery = q.Encode()
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "branch.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Commit an update to the branch
	repo.commitFile("branch-update.txt", "branch-update")

	// Get again should work
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "branch-update.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_tag(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "tag")
	repo.commitFile("tag.txt", "tag")
	repo.git("tag", "v1.0")

	q := repo.url.Query()
	q.Add("ref", "v1.0")
	repo.url.RawQuery = q.Encode()

	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath := filepath.Join(dst, "tag.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Get again should work
	if err := g.Get(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	mainPath = filepath.Join(dst, "tag.txt")
	if _, err := os.Stat(mainPath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_GetFile(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempTestFile(t)
	defer os.RemoveAll(filepath.Dir(dst))

	repo := testGitRepo(t, "file")
	repo.commitFile("file.txt", "hello")

	// Download the file
	repo.url.Path = filepath.Join(repo.url.Path, "file.txt")
	if err := g.GetFile(dst, repo.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Verify the main file exists
	if _, err := os.Stat(dst); err != nil {
		t.Fatalf("err: %s", err)
	}
	assertContents(t, dst, "hello")
}

func TestGitGetter_gitVersion(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}
	if runtime.GOOS == "windows" {
		t.Skip("skipping on windows since the test requires sh")
	}
	dir, err := ioutil.TempDir("", "go-getter")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(dir)

	script := filepath.Join(dir, "git")
	err = ioutil.WriteFile(
		script,
		[]byte("#!/bin/sh\necho \"git version 2.0 (Some Metadata Here)\n\""),
		0700)
	if err != nil {
		t.Fatal(err)
	}

	defer func(v string) {
		os.Setenv("PATH", v)
	}(os.Getenv("PATH"))

	os.Setenv("PATH", dir)

	// Asking for a higher version throws an error
	if err := checkGitVersion(context.Background(), "2.3"); err == nil {
		t.Fatal("expect git version error")
	}

	// Passes when version is satisfied
	if err := checkGitVersion(context.Background(), "1.9"); err != nil {
		t.Fatal(err)
	}
}

func TestGitGetter_sshKey(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	encodedKey := base64.StdEncoding.EncodeToString([]byte(testGitToken))

	// avoid getting locked by a github authenticity validation prompt
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	u, err := urlhelper.Parse("ssh://git@github.com/hashicorp/test-private-repo" +
		"?sshkey=" + encodedKey)
	if err != nil {
		t.Fatal(err)
	}

	if err := g.Get(dst, u); err != nil {
		t.Fatalf("err: %s", err)
	}

	readmePath := filepath.Join(dst, "README.md")
	if _, err := os.Stat(readmePath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_sshSCPStyle(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	encodedKey := base64.StdEncoding.EncodeToString([]byte(testGitToken))

	// avoid getting locked by a github authenticity validation prompt
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	// This test exercises the combination of the git detector and the
	// git getter, to make sure that together they make scp-style URLs work.
	client := &Client{
		Src: "git@github.com:hashicorp/test-private-repo?sshkey=" + encodedKey,
		Dst: dst,
		Pwd: ".",

		Mode: ClientModeDir,

		Detectors: []Detector{
			new(GitDetector),
		},
		Getters: map[string]Getter{
			"git": g,
		},
	}

	if err := client.Get(); err != nil {
		t.Fatalf("client.Get failed: %s", err)
	}

	readmePath := filepath.Join(dst, "README.md")
	if _, err := os.Stat(readmePath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_sshExplicitPort(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	encodedKey := base64.StdEncoding.EncodeToString([]byte(testGitToken))

	// avoid getting locked by a github authenticity validation prompt
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	// This test exercises the combination of the git detector and the
	// git getter, to make sure that together they make scp-style URLs work.
	client := &Client{
		Src: "git::ssh://git@github.com:22/hashicorp/test-private-repo?sshkey=" + encodedKey,
		Dst: dst,
		Pwd: ".",

		Mode: ClientModeDir,

		Detectors: []Detector{
			new(GitDetector),
		},
		Getters: map[string]Getter{
			"git": g,
		},
	}

	if err := client.Get(); err != nil {
		t.Fatalf("client.Get failed: %s", err)
	}

	readmePath := filepath.Join(dst, "README.md")
	if _, err := os.Stat(readmePath); err != nil {
		t.Fatalf("err: %s", err)
	}
}

func TestGitGetter_sshSCPStyleInvalidScheme(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	encodedKey := base64.StdEncoding.EncodeToString([]byte(testGitToken))

	// avoid getting locked by a github authenticity validation prompt
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	// This test exercises the combination of the git detector and the
	// git getter, to make sure that together they make scp-style URLs work.
	client := &Client{
		Src: "git::ssh://git@github.com:hashicorp/test-private-repo?sshkey=" + encodedKey,
		Dst: dst,
		Pwd: ".",

		Mode: ClientModeDir,

		Detectors: []Detector{
			new(GitDetector),
		},
		Getters: map[string]Getter{
			"git": g,
		},
	}

	err := client.Get()
	if err == nil {
		t.Fatalf("get succeeded; want error")
	}

	got := err.Error()
	want1, want2 := `invalid source string`, `invalid port number "hashicorp"`
	if !(strings.Contains(got, want1) || strings.Contains(got, want2)) {
		t.Fatalf("wrong error\ngot:  %s\nwant: %q or %q", got, want1, want2)
	}
}

func TestGitGetter_submodule(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	relpath := func(basepath, targpath string) string {
		relpath, err := filepath.Rel(basepath, targpath)
		if err != nil {
			t.Fatal(err)
		}
		return strings.Replace(relpath, `\`, `/`, -1)
		// on windows git still prefers relatives paths
		// containing `/` for submodules
	}

	// Set up the grandchild
	gc := testGitRepo(t, "grandchild")
	// Due to  CVE-2022-39253 (https://github.blog/2022-10-18-git-security-vulnerabilities-announced/#cve-2022-39253)
	// we are allowing file transport globally.
	gc.git("config", "--global", "protocol.file.allow", "always")
	gc.commitFile("grandchild.txt", "grandchild")

	// Set up the child
	c := testGitRepo(t, "child")
	c.commitFile("child.txt", "child")
	c.git("submodule", "add", "-f", relpath(c.dir, gc.dir))
	c.git("commit", "-m", "Add grandchild submodule")

	// Set up the parent
	p := testGitRepo(t, "parent")
	p.commitFile("parent.txt", "parent")
	p.git("submodule", "add", "-f", relpath(p.dir, c.dir))
	p.git("commit", "-m", "Add child submodule")

	// Clone the root repository
	if err := g.Get(dst, p.url); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Check that the files exist
	for _, path := range []string{
		filepath.Join(dst, "parent.txt"),
		filepath.Join(dst, "child", "child.txt"),
		filepath.Join(dst, "child", "grandchild", "grandchild.txt"),
	} {
		if _, err := os.Stat(path); err != nil {
			t.Fatalf("err: %s", err)
		}
	}
}

func TestGitGetter_setupGitEnv_sshKey(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("skipping on windows since the test requires sh")
	}

	cmd := exec.Command("/bin/sh", "-c", "echo $GIT_SSH_COMMAND")
	setupGitEnv(cmd, "/tmp/foo.pem")
	out, err := cmd.Output()
	if err != nil {
		t.Fatal(err)
	}

	actual := strings.TrimSpace(string(out))
	if actual != "ssh -i /tmp/foo.pem" {
		t.Fatalf("unexpected GIT_SSH_COMMAND: %q", actual)
	}
}

func TestGitGetter_setupGitEnvWithExisting_sshKey(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skipf("skipping on windows since the test requires sh")
		return
	}

	// start with an existing ssh command configuration
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	cmd := exec.Command("/bin/sh", "-c", "echo $GIT_SSH_COMMAND")
	setupGitEnv(cmd, "/tmp/foo.pem")
	out, err := cmd.Output()
	if err != nil {
		t.Fatal(err)
	}

	actual := strings.TrimSpace(string(out))
	if actual != "ssh -o StrictHostKeyChecking=no -o IdentitiesOnly=yes -i /tmp/foo.pem" {
		t.Fatalf("unexpected GIT_SSH_COMMAND: %q", actual)
	}
}

func TestGitGetter_setupGitEnvWithNoKeyFile(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skipf("skipping on windows since the test requires sh")
		return
	}

	// start with an existing ssh command configuration
	os.Setenv("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no")
	defer os.Setenv("GIT_SSH_COMMAND", "")

	cmd := exec.Command("/bin/sh", "-c", "echo $GIT_SSH_COMMAND")
	setupGitEnv(cmd, "")
	out, err := cmd.Output()
	if err != nil {
		t.Fatal(err)
	}

	actual := strings.TrimSpace(string(out))
	if actual != "ssh -o StrictHostKeyChecking=no" {
		t.Fatalf("unexpected GIT_SSH_COMMAND: %q", actual)
	}
}

func TestGitGetter_subdirectory_symlink(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	target, err := ioutil.TempFile("", "link-target")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(target.Name())

	repo := testGitRepo(t, "repo-with-symlink")
	innerDir := filepath.Join(repo.dir, "this-directory-contains-a-symlink")
	if err := os.Mkdir(innerDir, 0700); err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(innerDir, "this-is-a-symlink")
	if err := os.Symlink(target.Name(), path); err != nil {
		t.Fatal(err)
	}

	repo.git("add", path)
	repo.git("commit", "-m", "Adding "+path)

	u, err := url.Parse(fmt.Sprintf("git::%s//this-directory-contains-a-symlink", repo.url.String()))
	if err != nil {
		t.Fatal(err)
	}

	client := &Client{
		Src:             u.String(),
		Dst:             dst,
		Pwd:             ".",
		Mode:            ClientModeDir,
		DisableSymlinks: true,
		Detectors: []Detector{
			new(GitDetector),
		},
		Getters: map[string]Getter{
			"git": g,
		},
	}

	err = client.Get()

	if runtime.GOOS == "windows" {
		// Windows doesn't handle symlinks as one might expect with git.
		//
		// https://github.com/git-for-windows/git/wiki/Symbolic-Links
		filepath.Walk(dst, func(path string, info os.FileInfo, err error) error {
			if strings.Contains(path, "this-is-a-symlink") {
				if info.Mode()&os.ModeSymlink == os.ModeSymlink {
					// If you see this test fail in the future, you've probably enabled
					// symlinks within git on your Windows system. Our CI/CD system does
					// not do this, so this is this is the only way we can make this test
					// make any sense.
					t.Fatalf("windows git should not have cloned a symlink")
				}
			}
			return nil
		})
	} else {
		// We can rely on POSIX compliant systems running git to do the right thing.
		if err == nil {
			t.Fatalf("expected client get to fail")
		}
		if !errors.Is(err, ErrSymlinkCopy) {
			t.Fatalf("unexpected error: %v", err)
		}
	}

}

func TestGitGetter_subdirectory(t *testing.T) {
	if !testHasGit {
		t.Skip("git not found, skipping")
	}

	g := new(GitGetter)
	dst := tempDir(t)

	repo := testGitRepo(t, "empty-repo")
	u, err := url.Parse(fmt.Sprintf("git::%s//../../../../../../etc/passwd", repo.url.String()))
	if err != nil {
		t.Fatal(err)
	}

	client := &Client{
		Src: u.String(),
		Dst: dst,
		Pwd: ".",

		Mode: ClientModeDir,

		Detectors: []Detector{
			new(GitDetector),
		},
		Getters: map[string]Getter{
			"git": g,
		},
	}

	err = client.Get()
	if err == nil {
		t.Fatalf("expected client get to fail")
	}
	if !strings.Contains(err.Error(), "subdirectory component contain path traversal out of the repository") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestGitGetter_BadRemoteUrl(t *testing.T) {

	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	g := new(GitGetter)
	dst := tempDir(t)

	// try an option that exists
	badUrl := "--no-refs"

	u, err := url.Parse(badUrl)
	if err != nil {
		t.Fatal(err)
	}

	err = g.Get(dst, u)
	if err == nil {
		t.Fatalf("get succeeded; want error")
	}

	got := err.Error()
	want := `repository '--no-refs' does not exist`
	if !strings.Contains(got, want) {
		t.Fatalf("wrong error\ngot:  %s\nwant: %q", got, want)
	}
}

func TestGitGetter_BadGitConfig(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	ctx := context.Background()
	g := new(GitGetter)
	dst := tempDir(t)

	url, err := url.Parse("https://github.com/hashicorp/go-getter")
	if err != nil {
		t.Fatal(err)
	}

	_, err = os.Stat(dst)
	if err != nil && !os.IsNotExist(err) {
		t.Fatalf(err.Error())
	}
	if err == nil {
		// Update the repository containing the bad git config.
		// This should remove the bad git config file and initialize a new one.
		err = g.update(ctx, dst, testGitToken, url, "main", 1)
	} else {
		// Clone a repository with a git config file
		err = g.clone(ctx, dst, testGitToken, url, "main", 1)
		if err != nil {
			t.Fatalf(err.Error())
		}

		// Edit the git config file to simulate a bad git config
		gitConfigPath := filepath.Join(dst, ".git", "config")
		err = os.WriteFile(gitConfigPath, []byte("bad config"), 0600)
		if err != nil {
			t.Fatalf(err.Error())
		}

		// Update the repository containing the bad git config.
		// This should remove the bad git config file and initialize a new one.
		err = g.update(ctx, dst, testGitToken, url, "main", 1)
	}
	if err != nil {
		t.Fatalf(err.Error())
	}

	// Check if the .git/config file contains "bad config"
	gitConfigPath := filepath.Join(dst, ".git", "config")
	configBytes, err := os.ReadFile(gitConfigPath)
	if err != nil {
		t.Fatalf(err.Error())
	}
	if strings.Contains(string(configBytes), "bad config") {
		t.Fatalf("The .git/config file contains 'bad config'")
	}
}

func TestGitGetter_BadGitDirName(t *testing.T) {
	if !testHasGit {
		t.Log("git not found, skipping")
		t.Skip()
	}

	ctx := context.Background()
	g := new(GitGetter)
	dst := tempDir(t)

	url, err := url.Parse("https://github.com/hashicorp/go-getter")
	if err != nil {
		t.Fatal(err)
	}

	_, err = os.Stat(dst)
	if err != nil && !os.IsNotExist(err) {
		t.Fatalf(err.Error())
	}
	if err == nil {
		// Remove all variations of .git directories
		err = removeCaseInsensitiveGitDirectory(dst)
		if err != nil {
			t.Fatalf(err.Error())
		}
	} else {
		// Clone a repository with a git directory
		err = g.clone(ctx, dst, testGitToken, url, "main", 1)
		if err != nil {
			t.Fatalf(err.Error())
		}

		// Rename the .git directory to .GIT
		oldPath := filepath.Join(dst, ".git")
		newPath := filepath.Join(dst, ".GIT")
		err = os.Rename(oldPath, newPath)
		if err != nil {
			t.Fatalf(err.Error())
		}

		// Remove all variations of .git directories
		err = removeCaseInsensitiveGitDirectory(dst)
		if err != nil {
			t.Fatalf(err.Error())
		}
	}
	if err != nil {
		t.Fatalf(err.Error())
	}

	// Check if the .GIT directory exists
	if _, err := os.Stat(filepath.Join(dst, ".GIT")); !os.IsNotExist(err) {
		t.Fatalf(".GIT directory still exists")
	}

	// Check if the .git directory exists
	if _, err := os.Stat(filepath.Join(dst, ".git")); !os.IsNotExist(err) {
		t.Fatalf(".git directory still exists")
	}
}

// gitRepo is a helper struct which controls a single temp git repo.
type gitRepo struct {
	t   *testing.T
	url *url.URL
	dir string
}

// testGitRepo creates a new test git repository.
func testGitRepo(t *testing.T, name string) *gitRepo {
	dir, err := ioutil.TempDir("", "go-getter")
	if err != nil {
		t.Fatal(err)
	}
	dir = filepath.Join(dir, name)
	if err := os.Mkdir(dir, 0700); err != nil {
		t.Fatal(err)
	}

	r := &gitRepo{
		t:   t,
		dir: dir,
	}

	url, err := urlhelper.Parse("file://" + r.dir)
	if err != nil {
		t.Fatal(err)
	}
	r.url = url

	t.Logf("initializing git repo in %s", dir)
	r.git("init")
	r.git("config", "user.name", "go-getter")
	r.git("config", "user.email", "go-getter@hashicorp.com")

	return r
}

// git runs a git command against the repo.
func (r *gitRepo) git(args ...string) {
	cmd := exec.Command("git", args...)
	cmd.Dir = r.dir
	bfr := bytes.NewBuffer(nil)
	cmd.Stderr = bfr
	if err := cmd.Run(); err != nil {
		r.t.Fatal(err, bfr.String())
	}
}

// commitFile writes and commits a text file to the repo.
func (r *gitRepo) commitFile(file, content string) {
	path := filepath.Join(r.dir, file)
	if err := ioutil.WriteFile(path, []byte(content), 0600); err != nil {
		r.t.Fatal(err)
	}
	r.git("add", file)
	r.git("commit", "-m", "Adding "+file)
}

// latestCommit returns the full commit id of the latest commit on the current
// branch.
func (r *gitRepo) latestCommit() (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	cmd.Dir = r.dir
	rawOut, err := cmd.Output()
	if err != nil {
		return "", err
	}
	rawOut = bytes.TrimSpace(rawOut)
	return string(rawOut), nil
}

// This is a read-only deploy key for an empty test repository.
// Note: This is split over multiple lines to avoid being disabled by key
// scanners automatically.
var testGitToken = `-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA9cHsxCl3Jjgu9DHpwvmfFOl1XEdY+ShHDR/cMnzJ5ddk5/oV
Wy6EWatvyHZfRSZMwzv4PtKeUPm6iXjqWp4xdWU9khlPzozyj+U9Fq70TRVUW9E5
T1XdQVwJE421yffr4VMMwu60wBqjI1epapH2i2inYvw9Zl9X2MXq0+jTvFvDerbT
mDtfStDPljenELAIZtWVETSvbI46gALwbxbM2292ZUIL4D6jRz0aZMmyy/twYv8r
9WGJLwmYzU518Ie7zqKW/mCTdTrV0WRiDj0MeRaPgrGY9amuHE4r9iG/cJkwpKAO
Ccz0Hs6i89u9vZnTqZU9V7weJqRAQcMjXXR6yQIDAQABAoIBAQDBzICKnGxiTlHw
rd+6qqChnAy5jWYDbZjCJ8q8YZ3RS08+g/8NXZxvHftTqM0uOaq1FviHig3gq15H
hHvCpBc6jXDFYoKFzq6FfO/0kFkE5HoWweIgxwRow0xBCDJAJ+ryUEyy+Ay/pQHb
IAjwilRS0V+WdnVw4mTjBAhPvb4jPOo97Yfy3PYUyx2F3newkqXOZy+zx3G/ANoa
ncypfMGyy76sfCWKqw4J1gVkVQLwbB6gQkXUFGYwY9sRrxbG93kQw76Flc/E/s52
62j4v1IM0fq0t/St+Y/+s6Lkw` + `aqt3ft1nsqWcRaVDdqvMfkzgJGXlw0bGzJG5MEQ
AIBq3dHRAoGBAP8OeG/DKG2Z1VmSfzuz1pas1fbZ+F7venOBrjez3sKlb3Pyl2aH
mt2wjaTUi5v10VrHgYtOEdqyhQeUSYydWXIBKNMag0NLLrfFUKZK+57wrHWFdFjn
VgpsdkLSNTOZpC8gA5OaJ+36IcOPfGqyyP9wuuRoaYnVT1KEzqLa9FEFAoGBAPaq
pglwhil2rxjJE4zq0afQLNpAfi7Xqcrepij+xvJIcIj7nawxXuPxqRFxONE/h3yX
zkybO8wLdbHX9Iw/wc1j50Uf1Z5gHdLf7/hQJoWKpz1RnkWRy6CYON8v1tpVp0tb
OAajR/kZnzebq2mfa7pyy5zDCX++2kp/dcFwHf31AoGAE8oupBVTZLWj7TBFuP8q
LkS40U92Sv9v09iDCQVmylmFvUxcXPM2m+7f/qMTNgWrucxzC7kB/6MMWVszHbrz
vrnCTibnemgx9sZTjKOSxHFOIEw7i85fSa3Cu0qOIDPSnmlwfZpfcMKQrhjLAYhf
uhooFiLX1X78iZ2OXup4PHUCgYEAsmBrm83sp1V1gAYBBlnVbXakyNv0pCk/Vz61
iFXeRt1NzDGxLxGw3kQnED8BaIh5kQcyn8Fud7sdzJMv/LAqlT4Ww60mzNYTGyjo
H3jOsqm3ESfRvduWFreeAQBWbiOczGjV1i8D4EbAFfWT+tjXjchwKBf+6Yt5zn/o
Bw/uEHUCgYAFs+JPOR25oRyBs7ujrMo/OY1z/eXTVVgZxY+tYGe1FJqDeFyR7ytK
+JBB1MuDwQKGm2wSIXdCzTNoIx2B9zTseiPTwT8G7vqNFhXoIaTBp4P2xIQb45mJ
7GkTsMBHwpSMOXgX9Weq3v5xOJ2WxVtjENmd6qzxcYCO5lP15O17hA==
-----END RSA PRIVATE KEY-----`
