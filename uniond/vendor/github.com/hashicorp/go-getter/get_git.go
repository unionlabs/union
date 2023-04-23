package getter

import (
	"bytes"
	"context"
	"encoding/base64"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"time"

	urlhelper "github.com/hashicorp/go-getter/helper/url"
	safetemp "github.com/hashicorp/go-safetemp"
	version "github.com/hashicorp/go-version"
)

// GitGetter is a Getter implementation that will download a module from
// a git repository.
type GitGetter struct {
	getter

	// Timeout sets a deadline which all git CLI operations should
	// complete within. Zero value means no timeout.
	Timeout time.Duration
}

var defaultBranchRegexp = regexp.MustCompile(`\s->\sorigin/(.*)`)
var lsRemoteSymRefRegexp = regexp.MustCompile(`ref: refs/heads/([^\s]+).*`)

func (g *GitGetter) ClientMode(_ *url.URL) (ClientMode, error) {
	return ClientModeDir, nil
}

func (g *GitGetter) Get(dst string, u *url.URL) error {
	ctx := g.Context()

	if g.Timeout > 0 {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, g.Timeout)
		defer cancel()
	}

	if _, err := exec.LookPath("git"); err != nil {
		return fmt.Errorf("git must be available and on the PATH")
	}

	// The port number must be parseable as an integer. If not, the user
	// was probably trying to use a scp-style address, in which case the
	// ssh:// prefix must be removed to indicate that.
	//
	// This is not necessary in versions of Go which have patched
	// CVE-2019-14809 (e.g. Go 1.12.8+)
	if portStr := u.Port(); portStr != "" {
		if _, err := strconv.ParseUint(portStr, 10, 16); err != nil {
			return fmt.Errorf("invalid port number %q; if using the \"scp-like\" git address scheme where a colon introduces the path instead, remove the ssh:// portion and use just the git:: prefix", portStr)
		}
	}

	// Extract some query parameters we use
	var ref, sshKey string
	depth := 0 // 0 means "don't use shallow clone"
	q := u.Query()
	if len(q) > 0 {
		ref = q.Get("ref")
		q.Del("ref")

		sshKey = q.Get("sshkey")
		q.Del("sshkey")

		if n, err := strconv.Atoi(q.Get("depth")); err == nil {
			depth = n
		}
		q.Del("depth")

		// Copy the URL
		var newU url.URL = *u
		u = &newU
		u.RawQuery = q.Encode()
	}

	var sshKeyFile string
	if sshKey != "" {
		// Check that the git version is sufficiently new.
		if err := checkGitVersion(ctx, "2.3"); err != nil {
			return fmt.Errorf("Error using ssh key: %v", err)
		}

		// We have an SSH key - decode it.
		raw, err := base64.StdEncoding.DecodeString(sshKey)
		if err != nil {
			return err
		}

		// Create a temp file for the key and ensure it is removed.
		fh, err := ioutil.TempFile("", "go-getter")
		if err != nil {
			return err
		}
		sshKeyFile = fh.Name()
		defer os.Remove(sshKeyFile)

		// Set the permissions prior to writing the key material.
		if err := os.Chmod(sshKeyFile, 0600); err != nil {
			return err
		}

		// Write the raw key into the temp file.
		_, err = fh.Write(raw)
		fh.Close()
		if err != nil {
			return err
		}
	}

	// Clone or update the repository
	_, err := os.Stat(dst)
	if err != nil && !os.IsNotExist(err) {
		return err
	}
	if err == nil {
		err = g.update(ctx, dst, sshKeyFile, ref, depth)
	} else {
		err = g.clone(ctx, dst, sshKeyFile, u, ref, depth)
	}
	if err != nil {
		return err
	}

	// Next: check out the proper tag/branch if it is specified, and checkout
	if ref != "" {
		if err := g.checkout(ctx, dst, ref); err != nil {
			return err
		}
	}

	// Lastly, download any/all submodules.
	return g.fetchSubmodules(ctx, dst, sshKeyFile, depth)
}

// GetFile for Git doesn't support updating at this time. It will download
// the file every time.
func (g *GitGetter) GetFile(dst string, u *url.URL) error {
	td, tdcloser, err := safetemp.Dir("", "getter")
	if err != nil {
		return err
	}
	defer tdcloser.Close()

	// Get the filename, and strip the filename from the URL so we can
	// just get the repository directly.
	filename := filepath.Base(u.Path)
	u.Path = filepath.Dir(u.Path)

	// Get the full repository
	if err := g.Get(td, u); err != nil {
		return err
	}

	// Copy the single file
	u, err = urlhelper.Parse(fmtFileURL(filepath.Join(td, filename)))
	if err != nil {
		return err
	}

	fg := &FileGetter{Copy: true}
	return fg.GetFile(dst, u)
}

func (g *GitGetter) checkout(ctx context.Context, dst string, ref string) error {
	cmd := exec.CommandContext(ctx, "git", "checkout", ref)
	cmd.Dir = dst
	return getRunCommand(cmd)
}

// gitCommitIDRegex is a pattern intended to match strings that seem
// "likely to be" git commit IDs, rather than named refs. This cannot be
// an exact decision because it's valid to name a branch or tag after a series
// of hexadecimal digits too.
//
// We require at least 7 digits here because that's the smallest size git
// itself will typically generate, and so it'll reduce the risk of false
// positives on short branch names that happen to also be "hex words".
var gitCommitIDRegex = regexp.MustCompile("^[0-9a-fA-F]{7,40}$")

func (g *GitGetter) clone(ctx context.Context, dst, sshKeyFile string, u *url.URL, ref string, depth int) error {
	args := []string{"clone"}

	originalRef := ref // we handle an unspecified ref differently than explicitly selecting the default branch below
	if ref == "" {
		ref = findRemoteDefaultBranch(ctx, u)
	}
	if depth > 0 {
		args = append(args, "--depth", strconv.Itoa(depth))
		args = append(args, "--branch", ref)
	}
	args = append(args, u.String(), dst)

	cmd := exec.CommandContext(ctx, "git", args...)
	setupGitEnv(cmd, sshKeyFile)
	err := getRunCommand(cmd)
	if err != nil {
		if depth > 0 && originalRef != "" {
			// If we're creating a shallow clone then the given ref must be
			// a named ref (branch or tag) rather than a commit directly.
			// We can't accurately recognize the resulting error here without
			// hard-coding assumptions about git's human-readable output, but
			// we can at least try a heuristic.
			if gitCommitIDRegex.MatchString(originalRef) {
				return fmt.Errorf("%w (note that setting 'depth' requires 'ref' to be a branch or tag name)", err)
			}
		}
		return err
	}

	if depth < 1 && originalRef != "" {
		// If we didn't add --depth and --branch above then we will now be
		// on the remote repository's default branch, rather than the selected
		// ref, so we'll need to fix that before we return.
		return g.checkout(ctx, dst, originalRef)
	}
	return nil
}

func (g *GitGetter) update(ctx context.Context, dst, sshKeyFile, ref string, depth int) error {
	// Determine if we're a branch. If we're NOT a branch, then we just
	// switch to master prior to checking out
	cmd := exec.CommandContext(ctx, "git", "show-ref", "-q", "--verify", "refs/heads/"+ref)
	cmd.Dir = dst

	if getRunCommand(cmd) != nil {
		// Not a branch, switch to default branch. This will also catch
		// non-existent branches, in which case we want to switch to default
		// and then checkout the proper branch later.
		ref = findDefaultBranch(ctx, dst)
	}

	// We have to be on a branch to pull
	if err := g.checkout(ctx, dst, ref); err != nil {
		return err
	}

	if depth > 0 {
		cmd = exec.CommandContext(ctx, "git", "pull", "--depth", strconv.Itoa(depth), "--ff-only")
	} else {
		cmd = exec.CommandContext(ctx, "git", "pull", "--ff-only")
	}

	cmd.Dir = dst
	setupGitEnv(cmd, sshKeyFile)
	return getRunCommand(cmd)
}

// fetchSubmodules downloads any configured submodules recursively.
func (g *GitGetter) fetchSubmodules(ctx context.Context, dst, sshKeyFile string, depth int) error {
	args := []string{"submodule", "update", "--init", "--recursive"}
	if depth > 0 {
		args = append(args, "--depth", strconv.Itoa(depth))
	}
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = dst
	setupGitEnv(cmd, sshKeyFile)
	return getRunCommand(cmd)
}

// findDefaultBranch checks the repo's origin remote for its default branch
// (generally "master"). "master" is returned if an origin default branch
// can't be determined.
func findDefaultBranch(ctx context.Context, dst string) string {
	var stdoutbuf bytes.Buffer
	cmd := exec.CommandContext(ctx, "git", "branch", "-r", "--points-at", "refs/remotes/origin/HEAD")
	cmd.Dir = dst
	cmd.Stdout = &stdoutbuf
	err := cmd.Run()
	matches := defaultBranchRegexp.FindStringSubmatch(stdoutbuf.String())
	if err != nil || matches == nil {
		return "master"
	}
	return matches[len(matches)-1]
}

// findRemoteDefaultBranch checks the remote repo's HEAD symref to return the remote repo's
// default branch. "master" is returned if no HEAD symref exists.
func findRemoteDefaultBranch(ctx context.Context, u *url.URL) string {
	var stdoutbuf bytes.Buffer
	cmd := exec.CommandContext(ctx, "git", "ls-remote", "--symref", u.String(), "HEAD")
	cmd.Stdout = &stdoutbuf
	err := cmd.Run()
	matches := lsRemoteSymRefRegexp.FindStringSubmatch(stdoutbuf.String())
	if err != nil || matches == nil {
		return "master"
	}
	return matches[len(matches)-1]
}

// setupGitEnv sets up the environment for the given command. This is used to
// pass configuration data to git and ssh and enables advanced cloning methods.
func setupGitEnv(cmd *exec.Cmd, sshKeyFile string) {
	const gitSSHCommand = "GIT_SSH_COMMAND="
	var sshCmd []string

	// If we have an existing GIT_SSH_COMMAND, we need to append our options.
	// We will also remove our old entry to make sure the behavior is the same
	// with versions of Go < 1.9.
	env := os.Environ()
	for i, v := range env {
		if strings.HasPrefix(v, gitSSHCommand) && len(v) > len(gitSSHCommand) {
			sshCmd = []string{v}

			env[i], env[len(env)-1] = env[len(env)-1], env[i]
			env = env[:len(env)-1]
			break
		}
	}

	if len(sshCmd) == 0 {
		sshCmd = []string{gitSSHCommand + "ssh"}
	}

	if sshKeyFile != "" {
		// We have an SSH key temp file configured, tell ssh about this.
		if runtime.GOOS == "windows" {
			sshKeyFile = strings.Replace(sshKeyFile, `\`, `/`, -1)
		}
		sshCmd = append(sshCmd, "-i", sshKeyFile)
	}

	env = append(env, strings.Join(sshCmd, " "))
	cmd.Env = env
}

// checkGitVersion is used to check the version of git installed on the system
// against a known minimum version. Returns an error if the installed version
// is older than the given minimum.
func checkGitVersion(ctx context.Context, min string) error {
	want, err := version.NewVersion(min)
	if err != nil {
		return err
	}

	out, err := exec.CommandContext(ctx, "git", "version").Output()
	if err != nil {
		return err
	}

	fields := strings.Fields(string(out))
	if len(fields) < 3 {
		return fmt.Errorf("Unexpected 'git version' output: %q", string(out))
	}
	v := fields[2]
	if runtime.GOOS == "windows" && strings.Contains(v, ".windows.") {
		// on windows, git version will return for example:
		// git version 2.20.1.windows.1
		// Which does not follow the semantic versionning specs
		// https://semver.org. We remove that part in order for
		// go-version to not error.
		v = v[:strings.Index(v, ".windows.")]
	}

	have, err := version.NewVersion(v)
	if err != nil {
		return err
	}

	if have.LessThan(want) {
		return fmt.Errorf("Required git version = %s, have %s", want, have)
	}

	return nil
}
